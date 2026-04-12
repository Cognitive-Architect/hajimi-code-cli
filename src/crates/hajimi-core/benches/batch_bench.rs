//! BatchedStream performance benchmarks
//! DEBT-W03-B05: Throughput measurement vs ChannelStream

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use futures::StreamExt;
use hajimi_core::streaming::{BatchedStream, BatchConfig, ChannelStream, StreamChunk};
use tokio::runtime::Runtime;

/// Benchmark batched throughput vs raw channel stream
fn batched_throughput(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime");
    let mut group = c.benchmark_group("stream_throughput");
    
    for msg_count in [100, 500, 1000].iter() {
        // Baseline: ChannelStream (no batching)
        group.bench_with_input(
            BenchmarkId::new("channel_stream", msg_count),
            msg_count,
            |b, &n| {
                b.to_async(&rt).iter(|| async {
                    let (stream, tx) = ChannelStream::new(n);
                    let msgs: Vec<String> = (0..n).map(|i| format!("msg-{}", i)).collect();
                    
                    tokio::spawn(async move {
                        for msg in msgs {
                            tx.send(StreamChunk::Output(msg)).await.ok();
                        }
                    });
                    
                    let count = stream.count().await;
                    black_box(count)
                });
            },
        );
        
        // Optimized: BatchedStream
        group.bench_with_input(
            BenchmarkId::new("batched_stream", msg_count),
            msg_count,
            |b, &n| {
                b.to_async(&rt).iter(|| async {
                    let (inner, tx) = ChannelStream::new(n);
                    let stream = BatchedStream::new(inner, BatchConfig {
                        batch_size: 10,
                        flush_interval_ms: 50,
                        compression: false,
                    });
                    let msgs: Vec<String> = (0..n).map(|i| format!("msg-{}", i)).collect();
                    
                    tokio::spawn(async move {
                        for msg in msgs {
                            tx.send(StreamChunk::Output(msg)).await.ok();
                        }
                    });
                    
                    let count = stream.count().await;
                    black_box(count)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark different batch sizes
fn batch_size_comparison(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create Tokio runtime");
    let mut group = c.benchmark_group("batch_size");
    
    for size in [5, 10, 20, 50].iter() {
        group.bench_with_input(BenchmarkId::new("size", size), size, |b, &batch_size| {
            b.to_async(&rt).iter(|| async {
                let (inner, tx) = ChannelStream::new(1000);
                let stream = BatchedStream::new(inner, BatchConfig {
                    batch_size,
                    flush_interval_ms: 100,
                    compression: false,
                });
                
                tokio::spawn(async move {
                    for i in 0..1000 {
                        tx.send(StreamChunk::Output(format!("data-{}", i))).await.ok();
                    }
                });
                
                black_box(stream.count().await)
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, batched_throughput, batch_size_comparison);
criterion_main!(benches);
