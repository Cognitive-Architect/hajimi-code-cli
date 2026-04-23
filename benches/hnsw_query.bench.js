/**
 * HNSW Vector Query Benchmark - WASM Performance Validation
 * Target: 3.0x acceleration (Fallback: 2.5x)
 * Build: wasm-pack build --release --target nodejs
 */

const { HnswIndex } = require('../src/wasm/pkg/hajimi_wasm');
const fs = require('fs');
const path = require('path');

// Configuration
const VECTOR_DIM = 384;
const NUM_VECTORS = 10000;
const NUM_QUERIES = 1000;
const TOP_K = 10;
const WARMUP_ITERATIONS = 100;

/**
 * Generate random vector
 */
function generateVector(dim) {
    return Array.from({ length: dim }, () => Math.random() * 2 - 1);
}

/**
 * Calculate cosine similarity
 */
function cosineSimilarity(a, b) {
    let dot = 0, normA = 0, normB = 0;
    for (let i = 0; i < a.length; i++) {
        dot += a[i] * b[i];
        normA += a[i] * a[i];
        normB += b[i] * b[i];
    }
    return dot / (Math.sqrt(normA) * Math.sqrt(normB));
}

/**
 * Brute force search (baseline)
 */
function bruteForceSearch(vectors, query, k) {
    const scores = vectors.map((vec, idx) => ({
        idx,
        score: cosineSimilarity(vec, query)
    }));
    scores.sort((a, b) => b.score - a.score);
    return scores.slice(0, k);
}

/**
 * Run benchmark with warmup
 */
function runBenchmark(name, fn, iterations) {
    // Warmup - exclude JIT compilation time
    for (let i = 0; i < WARMUP_ITERATIONS; i++) {
        fn();
    }
    
    // Actual benchmark
    const times = [];
    for (let i = 0; i < iterations; i++) {
        const start = performance.now();
        fn();
        const end = performance.now();
        times.push(end - start);
    }
    
    const avg = times.reduce((a, b) => a + b, 0) / times.length;
    const min = Math.min(...times);
    const max = Math.max(...times);
    const p95 = times.sort((a, b) => a - b)[Math.floor(times.length * 0.95)];
    const p99 = times.sort((a, b) => a - b)[Math.floor(times.length * 0.99)];
    const stddev = Math.sqrt(times.reduce((sq, n) => sq + Math.pow(n - avg, 2), 0) / times.length);
    
    return { name, avg, min, max, p95, p99, stddev };
}

/**
 * Main benchmark
 */
async function main() {
    console.log('HNSW WASM Query Benchmark');
    console.log('=========================');
    console.log(`Vector dim: ${VECTOR_DIM}`);
    console.log(`Index size: ${NUM_VECTORS} vectors`);
    console.log(`Queries: ${NUM_QUERIES}`);
    console.log(`Top-K: ${TOP_K}`);
    console.log('Target acceleration: 3.0x (Fallback: 2.5x)');
    console.log('');

    // Generate test data
    console.log('Generating test data...');
    const vectors = Array.from({ length: NUM_VECTORS }, () => generateVector(VECTOR_DIM));
    const queries = Array.from({ length: NUM_QUERIES }, () => generateVector(VECTOR_DIM));

    // Build HNSW index
    console.log('Building HNSW index (SIMD + BF16 optimized)...');
    const index = new HnswIndex();
    for (let i = 0; i < vectors.length; i++) {
        index.add_vector(vectors[i]);
    }
    console.log(`Index built: ${index.size()} vectors`);
    console.log('');

    // Benchmark HNSW search
    console.log('Benchmarking HNSW (optimized)...');
    const hnswResult = runBenchmark('HNSW Search', () => {
        const query = queries[Math.floor(Math.random() * queries.length)];
        index.search(query, TOP_K);
    }, NUM_QUERIES);

    // Benchmark brute force (baseline)
    console.log('Benchmarking Brute Force (baseline 1.0x)...');
    const bfIterations = Math.min(NUM_QUERIES, 100); // Fewer for slow baseline
    const bfResult = runBenchmark('Brute Force', () => {
        const query = queries[Math.floor(Math.random() * queries.length)];
        bruteForceSearch(vectors, query, TOP_K);
    }, bfIterations);

    // Calculate acceleration
    const acceleration = bfResult.avg / hnswResult.avg;

    // Results
    console.log('\nResults:');
    console.log('--------');
    console.log(`HNSW Average latency: ${hnswResult.avg.toFixed(3)} ms`);
    console.log(`HNSW P95 latency: ${hnswResult.p95.toFixed(3)} ms`);
    console.log(`HNSW P99 latency: ${hnswResult.p99.toFixed(3)} ms`);
    console.log(`HNSW StdDev: ${hnswResult.stddev.toFixed(3)} ms`);
    console.log('');
    console.log(`Brute Force Average latency: ${bfResult.avg.toFixed(3)} ms`);
    console.log('');
    console.log(`Acceleration ratio: ${acceleration.toFixed(2)}x`);
    console.log(`Target: 3.0x (Fallback: 2.5x)`);
    console.log('');

    // Performance check
    if (acceleration >= 3.0) {
        console.log('✅ PASS: Target 3.0x achieved!');
    } else if (acceleration >= 2.5) {
        console.log('⚠️ FALLBACK: 2.5x achieved (below 3.0x target)');
        console.log('   DEBT-HNSW-PERF-001: Performance debt declared');
    } else {
        console.log('❌ FAIL: Below 2.5x fallback threshold');
        process.exit(1);
    }

    // Save report
    const report = {
        timestamp: new Date().toISOString(),
        config: {
            vector_dim: VECTOR_DIM,
            num_vectors: NUM_VECTORS,
            num_queries: NUM_QUERIES,
            top_k: TOP_K,
            warmup_iterations: WARMUP_ITERATIONS
        },
        hnsw: hnswResult,
        brute_force: bfResult,
        acceleration: acceleration,
        target: 3.0,
        fallback: 2.5,
        pass: acceleration >= 2.5
    };

    const reportPath = path.join(__dirname, '../docs/perf/HNSW-WASM-BENCHMARK-001.json');
    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
    console.log(`\nReport saved: ${reportPath}`);
    
    return acceleration;
}

// Run benchmark
main().then(accel => {
    process.exit(accel >= 2.5 ? 0 : 1);
}).catch(err => {
    console.error('Benchmark failed:', err);
    process.exit(1);
});
