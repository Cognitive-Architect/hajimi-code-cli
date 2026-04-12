//! pgvector Recall and Performance Benchmark Tests
#![deny(unsafe_code)]

use hajimi::db::pg_pool::create_pool;
use hajimi::index::pgvector::PgVectorIndex;
use rand::{SeedableRng, rngs::StdRng, Rng};
use std::time::Instant;

const TEST_SEED: u64 = 42;
const N_10K: usize = 10000;

/// Generate SIFT-like normalized 384-dim vector
fn generate_sift_vector(rng: &mut StdRng) -> [f32; 384] {
    let mut v = [0.0_f32; 384];
    for i in 0..384 { v[i] = rng.gen_range(0.0..255.0); }
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 { for x in &mut v { *x /= norm; } }
    v
}

/// Compute Recall@k with ground truth
fn compute_recall_at_k(pg_results: &[(String, f32)], ground_truth: &[(String, f32)], k: usize) -> f64 {
    let pg_set: std::collections::HashSet<_> = pg_results.iter().take(k).map(|(id, _)| id.clone()).collect();
    let gt_set: std::collections::HashSet<_> = ground_truth.iter().take(k).map(|(id, _)| id.clone()).collect();
    pg_set.intersection(&gt_set).count() as f64 / gt_set.len() as f64
}

/// Generate performance report
fn generate_report(recall_10: f64, p99_ms: f64, tps: f64) -> String {
    format!("## pgvector Performance Report\n- Recall@10: {:.2}% (>=95%)\n- P99 Latency: {:.2}ms (<10ms)\n- Insert TPS: {:.2} (>100)\n", 
        recall_10 * 100.0, p99_ms, tps)
}

#[tokio::test]
async fn test_pgvector_recall_at_10() {
    let pool = create_pool().await.expect("pool");
    let index = PgVectorIndex::new(pool).await.expect("index");
    index.create_hnsw_index().await.expect("hnsw");
    let mut rng = StdRng::seed_from_u64(TEST_SEED);
    for i in 0..N_10K {
        index.insert_vector(&format!("vec{}", i), generate_sift_vector(&mut rng)).await.expect("insert");
    }
    let mut total_recall = 0.0;
    for i in 0..100 {
        let query = generate_sift_vector(&mut StdRng::seed_from_u64(TEST_SEED + i as u64));
        let pg_results = index.search_vectors(query, 10).await.expect("search");
        let ground_truth = index.exact_search(query, 10).await.expect("exact");
        total_recall += compute_recall_at_k(&pg_results, &ground_truth, 10);
    }
    let avg_recall = total_recall / 100.0;
    println!("Recall@10 = {:.2}%", avg_recall * 100.0);
    assert!(avg_recall >= 0.95, "Recall@10 must be >= 0.95, got {:.2}%", avg_recall * 100.0);
}

#[tokio::test]
async fn test_pgvector_p99_latency() {
    let pool = create_pool().await.expect("pool");
    let index = PgVectorIndex::new(pool).await.expect("index");
    let mut rng = StdRng::seed_from_u64(TEST_SEED);
    for i in 0..N_10K {
        index.insert_vector(&format!("lat{}", i), generate_sift_vector(&mut rng)).await.expect("insert");
    }
    let mut latencies = Vec::new();
    for _ in 0..1000 {
        let start = Instant::now();
        index.search_vectors(generate_sift_vector(&mut rng), 10).await.expect("search");
        latencies.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    latencies.sort_by(|a, b| a.partial_cmp(b).expect("data: latency comparison should be valid"));
    let p99 = latencies[(latencies.len() as f64 * 0.99) as usize];
    println!("P99 Latency = {:.2}ms", p99);
    assert!(p99 < 10.0, "P99 must be < 10ms, got {:.2}ms", p99);
}

#[tokio::test]
async fn test_pgvector_throughput() {
    let pool = create_pool().await.expect("pool");
    let index = PgVectorIndex::new(pool).await.expect("index");
    let start = Instant::now();
    let mut rng = StdRng::seed_from_u64(TEST_SEED);
    for i in 0..1000 {
        index.insert_vector(&format!("tps{}", i), generate_sift_vector(&mut rng)).await.expect("insert");
    }
    let tps = 1000.0 / start.elapsed().as_secs_f64();
    println!("Insert TPS = {:.2}", tps);
    assert!(tps > 100.0, "TPS must be > 100, got {:.2}", tps);
}

/// Comparison: pgvector HNSW vs built-in HNSW
#[tokio::test]
async fn test_pgvector_vs_hnsw_comparison() {
    use hajimi::index::hnsw::HnswIndex;
    let pool = create_pool().await.expect("pool");
    let pg_index = PgVectorIndex::new(pool).await.expect("pg index");
    let hnsw_index = HnswIndex::new(std::env::temp_dir().join("hnsw_cmp")).expect("hnsw index");
    let mut rng = StdRng::seed_from_u64(TEST_SEED);
    for i in 0..1000 {
        let v = generate_sift_vector(&mut rng);
        pg_index.insert_vector(&format!("cmp{}", i), v).await.expect("pg insert");
        hnsw_index.add_vector(&format!("cmp{}", i), &v, 1).expect("hnsw insert");
    }
    let (mut pg_lat, mut hnsw_lat) = (Vec::new(), Vec::new());
    for _ in 0..100 {
        let query = generate_sift_vector(&mut rng);
        let pg_s = Instant::now();
        let _ = pg_index.search_vectors(query, 10).await.expect("pg search");
        pg_lat.push(pg_s.elapsed().as_secs_f64() * 1000.0);
        let hnsw_s = Instant::now();
        let _ = hnsw_index.search(&query, 10).expect("hnsw search");
        hnsw_lat.push(hnsw_s.elapsed().as_secs_f64() * 1000.0);
    }
    pg_lat.sort_by(|a, b| a.partial_cmp(b).expect("data: pg latency comparison should be valid"));
    hnsw_lat.sort_by(|a, b| a.partial_cmp(b).expect("data: hnsw latency comparison should be valid"));
    println!("pgvector P99: {:.2}ms, HNSW P99: {:.2}ms, Speedup: {:.2}x", 
        pg_lat[99], hnsw_lat[99], pg_lat[99] / hnsw_lat[99]);
}

/// Comprehensive performance report
#[tokio::test]
async fn test_pgvector_performance_report() {
    let pool = create_pool().await.expect("pool");
    let index = PgVectorIndex::new(pool).await.expect("index");
    index.create_hnsw_index().await.expect("hnsw");
    let mut rng = StdRng::seed_from_u64(TEST_SEED);
    let insert_start = Instant::now();
    for i in 0..N_10K {
        index.insert_vector(&format!("rpt{}", i), generate_sift_vector(&mut rng)).await.expect("insert");
    }
    let tps = N_10K as f64 / insert_start.elapsed().as_secs_f64();
    let mut total_recall = 0.0;
    for i in 0..100 {
        let query = generate_sift_vector(&mut StdRng::seed_from_u64(TEST_SEED + i as u64));
        total_recall += compute_recall_at_k(&index.search_vectors(query, 10).await.expect("search"), 
            &index.exact_search(query, 10).await.expect("exact"), 10);
    }
    let recall_10 = total_recall / 100.0;
    let mut latencies = Vec::new();
    for _ in 0..1000 {
        let start = Instant::now();
        index.search_vectors(generate_sift_vector(&mut rng), 10).await.expect("search");
        latencies.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    latencies.sort_by(|a, b| a.partial_cmp(b).expect("data: latency comparison should be valid"));
    let p99_ms = latencies[(latencies.len() as f64 * 0.99) as usize];
    println!("{}", generate_report(recall_10, p99_ms, tps));
    assert!(recall_10 >= 0.95 && p99_ms < 10.0 && tps > 100.0);
}
