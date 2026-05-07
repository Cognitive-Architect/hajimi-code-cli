//! HNSW Performance Benchmark Runner (B-14/17)
//!
//! This runner executes the HNSW benchmark suite via `cargo test`.
//! Actual benchmark logic lives in `src/intelligence/memory/src/dream.rs`.
//!
//! ## Target metrics
//! - latency: HNSW search < 5ms @ 10K vectors (vs linear scan baseline)
//! - recall: top-K accuracy against ground-truth self-query
//! - memory: estimated footprint < 200MB
//! - params: max_nb_connection tuning (M=8, 16, 32 comparison)
//!
//! ## Usage
//!   cargo test -p memory --features hnsw-index -- bench_hnsw --nocapture
//!   # Or compile and run this runner directly:
//!   rustc --edition 2021 benches/hnsw_bench.rs -o hnsw_bench_runner && ./hnsw_bench_runner

use std::process::{Command, Stdio};

fn main() {
    println!("=== HNSW Benchmark Runner (B-14/17) ===");
    println!("Target: latency <5ms, memory <200MB, recall >=0.90 @ 10K vectors");

    let status = Command::new("cargo")
        .args(&["test", "-p", "memory", "--features", "hnsw-index", "--", "bench_hnsw", "--nocapture"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(s) if s.success() => println!("\n=== All HNSW benchmarks passed ==="),
        Ok(s) => {
            eprintln!("\n=== HNSW benchmark failed with exit code: {:?} ===", s.code());
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("\n=== Failed to spawn cargo test: {} ===", e);
            std::process::exit(1);
        }
    }
}
