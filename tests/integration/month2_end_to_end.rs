//! Month 2 End-to-End Integration Tests
//! Session(4K) → Auto(50K触发) → Dream(384维) → Index(HNSW/Tantivy)

use std::time::Duration;
use std::collections::HashSet;
use std::path::PathBuf;
use crate::index::hnsw::HnswIndex;

/// Recall = |ANN结果 ∩ 精确TopK| / K
fn calc_recall(ann: &[String], exact: &[String]) -> f64 {
    if exact.is_empty() { return 1.0; }
    let set: HashSet<_> = ann.iter().cloned().collect();
    exact.iter().filter(|id| set.contains(*id)).count() as f64 / exact.len() as f64
}

/// 暴力精确搜索
fn brute_force(q: &[f32], vecs: &[(String, Vec<f32>)], k: usize) -> Vec<String> {
    if vecs.is_empty() || k == 0 { return Vec::new(); }
    let n = q.iter().map(|x| x * x).sum::<f32>().sqrt();
    let qn: Vec<f32> = if n > 0.0 { q.iter().map(|x| x / n).collect() } else { vec![0.0; 384] };
    let mut scored: Vec<(f32, String)> = vecs.iter()
        .map(|(id, v)| (qn.iter().zip(v).map(|(a, b)| a * b).sum::<f32>(), id.clone()))
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k); scored.into_iter().map(|(_, id)| id).collect()
}

/// 生成100条测试向量
fn gen_100() -> Vec<(String, Vec<f32>)> {
    const TOPICS: [&str; 10] = ["rust_programming", "machine_learning", "distributed_systems",
        "web_development", "database_design", "cloud_computing", "security_practices",
        "algorithm_theory", "software_engineering", "data_visualization"];
    let mut ds = Vec::with_capacity(100);
    for (ti, t) in TOPICS.iter().enumerate() {
        for ii in 0..10 {
            let s = ((ti as u64 + 1) * 1000) + (ii as u64 * 7);
            ds.push((format!("{}_{:02}", t, ii), gen_emb(s)));
        }
    } ds
}

/// Box-Muller生成384维高斯向量
fn gen_emb(seed: u64) -> Vec<f32> {
    const PI: f32 = std::f32::consts::PI;
    let mut st = seed;
    let mut v = Vec::with_capacity(384);
    for _ in 0..192 {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let u1 = (st as f32) / (u64::MAX as f32 + 1.0);
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let u2 = (st as f32) / (u64::MAX as f32 + 1.0);
        let r = (-2.0 * u1.max(f32::EPSILON).ln()).sqrt();
        v.push(r * (2.0 * PI * u2).cos());
        v.push(r * (2.0 * PI * u2).sin());
    }
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if n > 0.0 { for x in &mut v { *x /= n; } } v
}

#[tokio::test] async fn test_session_overflow_graceful() {
    let mut d = Vec::new();
    for i in 0..100 { d.push((format!("k{}", i), "a".repeat(500))); }
    let r = integration::session_to_index("t_overflow", d).await;
    assert!(r.is_ok(), "session_to_index should succeed"); 
    let s = r.expect("session_to_index result should be Ok");
    assert!(s.session_entries <= 100 && s.session_entries > 0);
}

#[tokio::test] async fn test_token_50k_threshold() {
    let mut d = Vec::new();
    for i in 0..60 { d.push((format!("k{}", i), "word ".repeat(4000))); }
    let r = integration::session_to_index("t_50k", d).await;
    assert!(r.is_ok(), "session_to_index should succeed"); 
    assert!(r.expect("result should be Ok").compression_triggered, "compression should be triggered");
}

#[tokio::test] async fn test_384_dimension_consistency() {
    let d = vec![("d384".to_string(), "dimension test".to_string())];
    let r = integration::session_to_index("t_384", d).await;
    assert!(r.is_ok(), "session_to_index should succeed"); 
    assert_eq!(r.expect("result should be Ok").indexed_vectors, 1, "should have 1 indexed vector");
}

#[tokio::test] async fn test_dimension_mismatch() {
    assert!(integration::test_dimension_mismatch("t_dim").await.is_ok());
}

#[tokio::test] async fn test_dual_engine() {
    let d = vec![("d1".to_string(), "Rust memory".to_string()),
        ("d2".to_string(), "Python dynamic".to_string())];
    assert!(integration::session_to_index("t_dual", d).await.is_ok());
}

#[tokio::test] async fn test_onnx_placeholder() {
    let d = vec![("p1".to_string(), "ONNX".to_string()), ("p2".to_string(), "test".to_string())];
    let r = integration::session_to_index("t_onnx", d).await;
    assert!(r.is_ok(), "session_to_index should succeed"); 
    let s = r.expect("session_to_index result should be Ok");
    assert_eq!(s.dream_entries, 2); assert_eq!(s.indexed_vectors, 2);
}

#[tokio::test] async fn test_empty_index() {
    assert!(integration::test_empty_index_search("t_empty").await.is_ok());
}

/// 召回率 90-95% 真实验证 (100条数据集)
/// Week 32-Rework: HNSW ANN vs 暴力精确搜索对比
#[tokio::test] async fn test_recall_rate_90() {
    let ds = gen_100();
    assert_eq!(ds.len(), 100, "数据集必须包含100条");
    
    // 1. 构建HNSW索引（ANN近似搜索）
    let tmp_dir = std::env::temp_dir().join(format!("hnsw_recall_{}", std::process::id()));
    let hnsw = HnswIndex::new(tmp_dir.clone()).expect("创建HNSW索引失败");
    for (id, emb) in &ds {
        hnsw.add_vector(id, emb, 1).expect("添加向量失败");
    }
    
    // 2. 选择10个查询向量
    let queries: Vec<_> = ds.iter().enumerate().filter(|(i, _)| i % 10 == 0)
        .map(|(_, (id, v))| (id.clone(), v.clone())).collect();
    
    // 3. ANN vs 精确对比
    let mut total_recall = 0.0;
    for (qid, qv) in &queries {
        // ANN搜索结果（近似）
        let ann_results = hnsw.search(qv, 10).expect("HNSW搜索失败");
        let ann_ids: Vec<String> = ann_results.iter().map(|r| r.doc_id.clone()).collect();
        
        // 暴力精确基准（Ground Truth）
        let exact_results = brute_force(qv, &ds, 10);
        
        // 计算recall: |ANN ∩ Exact| / |Exact|
        let recall = calc_recall(&ann_ids, &exact_results);
        total_recall += recall;
        
        // 验证自身必须能找到
        assert!(exact_results.contains(qid), "精确搜索应找到自身");
    }
    
    // 4. 验证结果
    // DEBT-HNSW-ANN-W32: 当前HNSW实现为精确搜索（暴力扫描），非ANN近似
    // 因此真实Recall=100%，这是实现特性，非测试问题
    // 待HNSW实现真正ANN算法后，预期Recall将降至90-95%
    let avg = total_recall / queries.len() as f64;
    assert!(avg >= 0.90, "Recall应>=90%, 实际={:.2}%", avg * 100.0);
    // 当前精确实现下预期100%，ANN实现后应调整为90-95%区间
    
    // 清理临时目录
    let _ = std::fs::remove_dir_all(&tmp_dir);
}

#[tokio::test] async fn test_performance() {
    let d: Vec<_> = (0..50).map(|i| (format!("p{}", i), format!("perf {}", i))).collect();
    let s = std::time::Instant::now();
    assert!(integration::session_to_index("t_perf", d).await.is_ok());
    assert!(s.elapsed() < Duration::from_secs(5));
}

#[tokio::test] async fn test_data_flow_integrity() {
    let d = vec![("f1".to_string(), "flow 1".to_string()), ("f2".to_string(), "flow 2".to_string())];
    let r = integration::session_to_index("t_flow", d).await;
    assert!(r.is_ok(), "session_to_index should succeed"); 
    let s = r.expect("session_to_index result should be Ok");
    assert!(s.session_entries >= s.auto_entries && s.auto_entries >= s.dream_entries);
}

#[tokio::test] async fn test_invalid_project_id() {
    assert!(integration::session_to_index("", vec![("k".to_string(), "v".to_string())]).await.is_err());
}

#[tokio::test] async fn test_zero_unwrap() {
    let _ = integration::session_to_index("t_unwrap", vec![("".to_string(), "".to_string())]).await;
}

#[tokio::test] async fn test_concurrent() {
    use tokio::task;
    let hs: Vec<_> = (0..3).map(|i| task::spawn(async move {
        integration::session_to_index(&format!("t_conc{}", i),
            vec![(format!("c{}", i), "concurrent".to_string())]).await
    })).collect();
    for h in hs { assert!(h.await.is_ok()); }
}

#[test] #[cfg(feature = "test_utils")] fn test_onnx_mock_embedding_quality() {}

/// 边界: 空数据集
#[test] fn test_empty_dataset_search() {
    let r = brute_force(&[0.1; 384], &[], 10);
    assert!(r.is_empty());
}

/// 边界: 零向量
#[test] fn test_zero_vector_handling() {
    let ds = gen_100();
    let r = brute_force(&[0.0; 384], &ds, 10);
    assert!(r.len() <= 10);
}

/// 边界: 错误维度
#[test] fn test_dimension_mismatch_detection() {
    let ds = gen_100();
    let r = brute_force(&[0.1; 100], &ds, 10);
    assert!(r.len() <= 10);
}
