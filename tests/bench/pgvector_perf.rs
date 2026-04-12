//! pgvector Performance Benchmark - Week 40 Final Validation
//! Validates: Recall@10 >= 0.95, P99 < 10ms, TPS > 100
#![deny(unsafe_code)]

use hajimi::db::pg_pool::create_pool;
use hajimi::index::pgvector::PgVectorIndex;
use rand::{SeedableRng, rngs::StdRng, Rng};
use std::time::Instant;

const TEST_SEED: u64 = 42;
const DIM: usize = 384;
const N_10K: usize = 10000;
const K: usize = 10;

#[derive(Debug)]
enum BenchError {
    Db(String),
    Validation(String),
}

impl std::fmt::Display for BenchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BenchError::Db(msg) => write!(f, "DB error: {}", msg),
            BenchError::Validation(msg) => write!(f, "Validation: {}", msg),
        }
    }
}

impl std::error::Error for BenchError {}
impl From<sqlx::Error> for BenchError {
    fn from(e: sqlx::Error) -> Self { BenchError::Db(e.to_string()) }
}

/// Generate normalized 384-dim test vector
fn generate_vector(rng: &mut StdRng) -> [f32; 384] {
    let mut v = [0.0_f32; DIM];
    for i in 0..DIM { v[i] = rng.gen_range(-1.0..1.0); }
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 { for x in &mut v { *x /= norm; } }
    v
}

/// Ground truth: brute force linear scan
async fn ground_truth_search(store: &PgVectorIndex, query: [f32; 384], k: usize) -> Result<Vec<(String, f32)>, BenchError> {
    store.exact_search(query, k).await.map_err(BenchError::from)
}

/// Compute Recall@k
fn compute_recall(pg_results: &[(String, f32)], gt_results: &[(String, f32)], k: usize) -> f64 {
    let pg_set: std::collections::HashSet<_> = pg_results.iter().take(k).map(|(id, _)| id.clone()).collect();
    let gt_set: std::collections::HashSet<_> = gt_results.iter().take(k).map(|(id, _)| id.clone()).collect();
    let intersection = pg_set.intersection(&gt_set).count();
    intersection as f64 / k as f64
}

/// Validate recall >= 0.95
fn validate_recall(recall: f64) -> Result<(), BenchError> {
    if recall >= 0.95 { Ok(()) } else { Err(BenchError::Validation(format!("Recall@10 must be >= 0.95, got {:.2}%", recall * 100.0))) }
}

/// Validate P99 < 10ms
fn validate_p99(p99_ms: f64) -> Result<(), BenchError> {
    if p99_ms < 10.0 { Ok(()) } else { Err(BenchError::Validation(format!("P99 must be < 10ms, got {:.2}ms", p99_ms))) }
}

/// Validate TPS > 100
fn validate_tps(tps: f64) -> Result<(), BenchError> {
    if tps > 100.0 { Ok(()) } else { Err(BenchError::Validation(format!("TPS must be > 100, got {:.2}", tps))) }
}

#[tokio::test]
async fn benchmark_recall() -> Result<(), BenchError> {
    let pool = create_pool().await?;
    let store = PgVectorIndex::new(pool).await?;
    store.create_hnsw_index().await?;

    let mut rng = StdRng::seed_from_u64(TEST_SEED);
    for i in 0..N_10K {
        let v = generate_vector(&mut rng);
        store.insert_vector(&format!("vec{}", i), v).await?;
    }

    let mut total_recall = 0.0;
    for i in 0..100 {
        let query = generate_vector(&mut StdRng::seed_from_u64(TEST_SEED + i as u64));
        let pg_results = store.search_vectors(query, K).await?;
        let gt_results = ground_truth_search(&store, query, K).await?;
        total_recall += compute_recall(&pg_results, &gt_results, K);
    }

    let avg_recall = total_recall / 100.0;
    println!("Recall@10 = {:.2}%", avg_recall * 100.0);
    assert!(avg_recall >= 0.95, "Recall@10 must be >= 0.95, got {:.2}%", avg_recall * 100.0);
    validate_recall(avg_recall)?;
    Ok(())
}

#[tokio::test]
async fn benchmark_latency() -> Result<(), BenchError> {
    let pool = create_pool().await?;
    let store = PgVectorIndex::new(pool).await?;
    let mut rng = StdRng::seed_from_u64(TEST_SEED);

    for i in 0..N_10K {
        let v = generate_vector(&mut rng);
        store.insert_vector(&format!("lat{}", i), v).await?;
    }

    let mut latencies = Vec::with_capacity(1000);
    for _ in 0..1000 {
        let query = generate_vector(&mut rng);
        let start = Instant::now();
        store.search_vectors(query, K).await?;
        latencies.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p99 = latencies[(latencies.len() as f64 * 0.99) as usize];
    println!("P99 Latency = {:.2}ms", p99);
    assert!(p99 < 10.0, "P99 must be < 10ms, got {:.2}ms", p99);
    validate_p99(p99)?;
    Ok(())
}

#[tokio::test]
async fn benchmark_throughput() -> Result<(), BenchError> {
    let pool = create_pool().await?;
    let store = PgVectorIndex::new(pool).await?;
    let mut rng = StdRng::seed_from_u64(TEST_SEED);

    let start = Instant::now();
    for i in 0..1000 {
        let v = generate_vector(&mut rng);
        store.insert_vector(&format!("tps{}", i), v).await?;
    }

    let tps = 1000.0 / start.elapsed().as_secs_f64();
    println!("Insert TPS = {:.2}", tps);
    assert!(tps > 100.0, "TPS must be > 100, got {:.2}", tps);
    validate_tps(tps)?;
    Ok(())
}
