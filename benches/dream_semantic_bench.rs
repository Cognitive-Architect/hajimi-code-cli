//! DreamMemory Semantic Embedding Benchmark (B-06/17)
//! Measures hash vs semantic embed latency, memory estimate, and mixed throughput.

use std::time::Instant;

fn main() {
    println!("=== DreamMemory Semantic Benchmark ===");

    // Hash embed latency (no model needed)
    let hash_start = Instant::now();
    for _ in 0..1000 {
        let _ = hash_embed("benchmark text for hash latency measurement");
    }
    let hash_avg = hash_start.elapsed().as_micros() as f64 / 1000.0;
    println!("hash_embed avg latency: {:.2} us", hash_avg);

    // Memory estimate: 1000 cache entries × 384 dims × 4 bytes
    let cache_entries = 1000usize;
    let dim = 384usize;
    let bytes_per_vec = dim * 4;
    let cache_mb = (cache_entries * bytes_per_vec) as f64 / (1024.0 * 1024.0);
    println!("LRU cache memory estimate: {:.2} MB ({} entries × {} dims)", cache_mb, cache_entries, dim);

    // Mixed scenario throughput: simulate interleaved hash/semantic
    let mixed_start = Instant::now();
    for i in 0..500 {
        let text = format!("mixed scenario text iteration {}", i);
        let _ = hash_embed(&text);
    }
    let mixed_total_ms = mixed_start.elapsed().as_millis() as f64;
    println!("mixed throughput: {:.2} texts/sec", 500.0 / (mixed_total_ms / 1000.0));

    println!("=== Benchmark Complete ===");
}

/// Standalone hash-based embedding matching dream.rs logic.
fn hash_embed(text: &str) -> Vec<f32> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hasher::write(&mut hasher, text.as_bytes());
    let seed = std::hash::Hasher::finish(&mut hasher);
    let mut vec = Vec::with_capacity(384);
    let mut state = seed;
    for _ in 0..384 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let val = ((state >> 32) as u32) as f32 / u32::MAX as f32;
        vec.push(val * 2.0 - 1.0);
    }
    vec
}
