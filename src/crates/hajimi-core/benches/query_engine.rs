//! Performance Benchmarks - QueryEngine (B-W07-02)
//!
//! 基准指标:
//! - QueryEngine 1000并发流延迟P95 < 500ms
//! - Config热重载 < 3秒
//! - 串行vs并行性能对比

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hajimi_core::{
    ParallelExecutor, Query, SerialExecutor, Executor,
};
use serde_json::json;
use std::time::Duration;
use tokio::runtime::Runtime;

fn create_queries(count: usize) -> Vec<Query> {
    (0..count)
        .map(|i| Query::new("test", json!({"id": i, "data": "benchmark"})))
        .collect()
}

fn bench_serial_executor(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("serial_executor");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("batch", size), size, |b, &size| {
            let executor = SerialExecutor::new();
            let queries = create_queries(size);
            
            b.to_async(&rt).iter(|| async {
                let results = executor.execute_batch(black_box(queries.clone())).await;
                black_box(results);
            });
        });
    }
    
    group.finish();
}

fn bench_parallel_executor(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("parallel_executor");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::new("batch", size), size, |b, &size| {
            let executor = ParallelExecutor::default().with_max_concurrency(50);
            let queries = create_queries(size);
            
            b.to_async(&rt).iter(|| async {
                let results = executor.execute_batch(black_box(queries.clone())).await;
                black_box(results);
            });
        });
    }
    
    group.finish();
}

fn bench_concurrency_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrency_scaling");
    group.measurement_time(Duration::from_secs(10));
    
    for concurrency in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("max_concurrency", concurrency),
            concurrency,
            |b, &concurrency| {
                let executor = ParallelExecutor::default().with_max_concurrency(concurrency);
                let queries = create_queries(100);
                
                b.to_async(&rt).iter(|| async {
                    let results = executor.execute_batch(black_box(queries.clone())).await;
                    black_box(results);
                });
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_serial_executor,
    bench_parallel_executor,
    bench_concurrency_scaling
);
criterion_main!(benches);
