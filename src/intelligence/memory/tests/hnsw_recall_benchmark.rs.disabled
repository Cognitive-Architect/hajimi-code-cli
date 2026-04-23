//! HNSW Recall and Performance Benchmark Tests
//! Week 37 Deliverable: Recall@10≥90% + 10K P99<10ms Validation
#![deny(unsafe_code)]

use memory::hnsw::{HnswIndex, EMBEDDING_DIM, Neighbor};
use rand::{SeedableRng, rngs::StdRng, Rng};
use std::time::Instant;
use std::collections::HashSet;

const TEST_SEED: u64 = 42;
const N_10K: usize = 10000;

/// Generate SIFT-like normalized 384-dim vector
fn generate_sift_vector(rng: &mut StdRng) -> [f32; EMBEDDING_DIM] {
    let mut v = [0.0_f32; EMBEDDING_DIM];
    for i in 0..EMBEDDING_DIM { v[i] = rng.gen_range(0.0..255.0); }
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 { for x in &mut v { *x /= norm; } }
    v
}

/// Ground truth brute force search - 遍历所有节点，精确距离，返回最近 k 个
fn brute_force_search(conn: &rusqlite::Connection, query: [f32; EMBEDDING_DIM], k: usize) -> Vec<String> {
    let mut all: Vec<(String, f32)> = Vec::new();
    let mut stmt = conn.prepare("SELECT id, vector_json FROM hnsw_nodes WHERE level = 0").unwrap();
    let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))).unwrap();
    for row in rows {
        let (id, vj) = row.unwrap();
        let v: Vec<f32> = serde_json::from_str(&vj).unwrap();
        if v.len() != EMBEDDING_DIM { continue; }
        let mut arr = [0.0_f32; EMBEDDING_DIM];
        arr.copy_from_slice(&v);
        let dist = euclidean_distance(query, arr);
        all.push((id, dist));
    }
    drop(stmt);
    all.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    all.into_iter().take(k).map(|(id, _)| id).collect()
}

fn euclidean_distance(a: [f32; EMBEDDING_DIM], b: [f32; EMBEDDING_DIM]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f32>().sqrt()
}

/// Calculate Recall@k
fn calculate_recall(ann_results: &[String], ground_truth: &[String]) -> f64 {
    if ground_truth.is_empty() { return 0.0; }
    let ann_set: HashSet<_> = ann_results.iter().collect();
    let gt_set: HashSet<_> = ground_truth.iter().collect();
    ann_set.intersection(&gt_set).count() as f64 / ground_truth.len() as f64
}

/// Generate performance report
fn generate_report(recall_10: f64, recall_50: f64, p99_ms: f64, tps: f64) -> String {
    format!("## Performance Report\n- Recall@10: {:.2}%\n- Recall@50: {:.2}%\n- P99 Latency: {:.2}ms\n- Insert TPS: {:.2}\n",
        recall_10 * 100.0, recall_50 * 100.0, p99_ms, tps)
}

/// Test Recall@10 >= 0.90 (地狱红线: Recall@10 < 90% → Week 37失败)
#[test]
fn test_recall_at_10() {
    let mut rng = StdRng::seed_from_u64(TEST_SEED);
    let temp = tempfile::NamedTempFile::new().expect("temp file");
    let db_path = temp.path().to_str().unwrap();
    let mut index = HnswIndex::new(db_path).expect("index");
    const N: usize = 500;
    for i in 0..N {
        index.insert_with_levels(&format!("n{}", i), generate_sift_vector(&mut rng)).expect("insert");
    }
    let conn = rusqlite::Connection::open(db_path).unwrap();
    let mut total_recall = 0.0;
    for _ in 0..50 {
        let query = generate_sift_vector(&mut rng);
        let ann_ids: Vec<String> = index.search_ann_with_ef(query, 10, 64).expect("ann").iter().map(|n| n.id.clone()).collect();
        let gt = brute_force_search(&conn, query, 10);
        total_recall += calculate_recall(&ann_ids, &gt);
    }
    let avg_recall = total_recall / 50.0;
    println!("Recall@10 = {:.2}%", avg_recall * 100.0);
    assert!(avg_recall >= 0.90, "Recall@10 must be >= 90%, got {:.2}%", avg_recall * 100.0);
}

/// Test Recall@50 >= 0.95
#[test]
fn test_recall_at_50() {
    let mut rng = StdRng::seed_from_u64(TEST_SEED);
    let temp = tempfile::NamedTempFile::new().expect("temp file");
    let db_path = temp.path().to_str().unwrap();
    let mut index = HnswIndex::new(db_path).expect("index");
    for i in 0..1000 {
        index.insert_with_levels(&format!("n{}", i), generate_sift_vector(&mut rng)).expect("insert");
    }
    let conn = rusqlite::Connection::open(db_path).unwrap();
    let mut total_recall = 0.0;
    for _ in 0..30 {
        let query = generate_sift_vector(&mut rng);
        let ann_ids: Vec<String> = index.search_ann_with_ef(query, 50, 128).expect("ann").iter().map(|n| n.id.clone()).collect();
        let gt = brute_force_search(&conn, query, 50);
        total_recall += calculate_recall(&ann_ids, &gt);
    }
    let avg_recall = total_recall / 30.0;
    println!("Recall@50 = {:.2}%", avg_recall * 100.0);
    assert!(avg_recall >= 0.95, "Recall@50 must be >= 95%, got {:.2}%", avg_recall * 100.0);
}

/// Test P99 latency < 10ms for 10K nodes (地狱红线: P99 >= 10ms → 性能不达标)
#[test]
fn test_latency_p99_10k() {
    let mut rng = StdRng::seed_from_u64(TEST_SEED);
    let temp = tempfile::NamedTempFile::new().expect("temp file");
    let mut index = HnswIndex::new(temp.path().to_str().unwrap()).expect("index");
    for i in 0..N_10K {
        index.insert_with_levels(&format!("n{}", i), generate_sift_vector(&mut rng)).expect("insert");
    }
    let mut latencies: Vec<std::time::Duration> = Vec::with_capacity(100);
    for _ in 0..100 {
        let query = generate_sift_vector(&mut rng);
        let start = Instant::now();
        let _ = index.search_ann_with_ef(query, 10, 64).expect("search");
        latencies.push(start.elapsed());
    }
    latencies.sort();
    let p99_ms = latencies[99 * latencies.len() / 100].as_secs_f64() * 1000.0;
    println!("P99 Latency (10K): {:.2}ms", p99_ms);
    assert!(p99_ms < 10.0, "P99 latency must be < 10ms, got {:.2}ms", p99_ms);
}

/// Test insert throughput - 目标：相比Week 36提升>3x (地狱红线: <3x → DEBT-PERF-INSERT-W36未清偿)
#[test]
fn test_insert_throughput() {
    let mut rng = StdRng::seed_from_u64(TEST_SEED);
    let temp = tempfile::NamedTempFile::new().expect("temp file");
    let mut index = HnswIndex::new(temp.path().to_str().unwrap()).expect("index");
    const BASELINE_TPS_W36: f64 = 50.0;
    let start = Instant::now();
    for i in 0..N_10K {
        index.insert_with_levels(&format!("n{}", i), generate_sift_vector(&mut rng)).expect("insert");
    }
    let tps = N_10K as f64 / start.elapsed().as_secs_f64();
    let speedup = tps / BASELINE_TPS_W36;
    println!("Insert TPS: {:.1}, Speedup: {:.2}x", tps, speedup);
    assert!(speedup >= 3.0, "Insert throughput must improve >3x, got {:.2}x", speedup);
}

/// 综合性能报告测试
#[test]
fn test_performance_report() {
    let mut rng = StdRng::seed_from_u64(TEST_SEED);
    let temp = tempfile::NamedTempFile::new().expect("temp file");
    let db_path = temp.path().to_str().unwrap();
    let mut index = HnswIndex::new(db_path).expect("index");
    
    // Insert 10K
    let insert_start = Instant::now();
    for i in 0..N_10K {
        index.insert_with_levels(&format!("n{}", i), generate_sift_vector(&mut rng)).expect("insert");
    }
    let tps = N_10K as f64 / insert_start.elapsed().as_secs_f64();
    
    let conn = rusqlite::Connection::open(db_path).unwrap();
    
    // Recall@10
    let mut r10 = 0.0;
    for _ in 0..50 {
        let q = generate_sift_vector(&mut rng);
        let ann: Vec<String> = index.search_ann_with_ef(q, 10, 64).unwrap().iter().map(|n| n.id.clone()).collect();
        r10 += calculate_recall(&ann, &brute_force_search(&conn, q, 10));
    }
    
    // Recall@50
    let mut r50 = 0.0;
    for _ in 0..20 {
        let q = generate_sift_vector(&mut rng);
        let ann: Vec<String> = index.search_ann_with_ef(q, 50, 128).unwrap().iter().map(|n| n.id.clone()).collect();
        r50 += calculate_recall(&ann, &brute_force_search(&conn, q, 50));
    }
    
    // P99 latency
    let mut latencies: Vec<_> = (0..100).map(|_| {
        let q = generate_sift_vector(&mut rng);
        let s = Instant::now();
        let _ = index.search_ann_with_ef(q, 10, 64);
        s.elapsed()
    }).collect();
    latencies.sort();
    let p99_ms = latencies[99].as_secs_f64() * 1000.0;
    
    println!("{}", generate_report(r10 / 50.0, r50 / 20.0, p99_ms, tps));
}
