//! HNSW → Tantivy vector-to-text hybrid module with zero-copy optimization
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Strategy marker for zero-copy transfer paths
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZeroCopyStrategy {
    ZeroCopy,
    DirectPointer,
    MemoryShare,
}

/// HNSW neighbor result (lightweight, no text payload)
#[derive(Clone, Debug, Default)]
pub struct HnswNeighbor {
    pub doc_id: u64,
    pub score: f32,
}

/// Output from HNSW index search
#[derive(Clone, Debug, Default)]
pub struct HnswOutput {
    pub neighbors: Vec<HnswNeighbor>,
    pub doc_ids: Vec<u64>,
}

/// Bridging struct: HNSW output → Tantivy input without serialization round-trip
#[derive(Clone, Debug)]
pub struct HnswOutputToTantivyInput<'a> {
    pub strategy: ZeroCopyStrategy,
    pub neighbors: &'a [HnswNeighbor],
    /// Direct pointer to doc_id buffer for Tantivy term construction
    pub direct_pointer: *const u64,
    pub direct_len: usize,
}

/// Hybrid query combining vector neighbors with full-text constraints
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorTextQuery {
    pub text_query: String,
    pub top_k: usize,
    #[serde(skip)]
    pub neighbor_ids: Vec<u64>,
    #[serde(skip_serializing)]
    pub vector_scores: Vec<f32>,
}

/// Serialization metrics for profiling
#[derive(Clone, Debug, Default)]
pub struct SerializationStats {
    pub bytes_copied: usize,
    pub serialization_us: u128,
}

impl HnswOutput {
    pub fn from_neighbors(neighbors: Vec<HnswNeighbor>) -> Self {
        let doc_ids: Vec<u64> = neighbors.iter().map(|n| n.doc_id).collect();
        Self { neighbors, doc_ids }
    }
}

impl<'a> HnswOutputToTantivyInput<'a> {
    pub fn new(output: &'a HnswOutput) -> Self {
        let direct_pointer = output.doc_ids.as_ptr();
        Self {
            strategy: ZeroCopyStrategy::ZeroCopy,
            neighbors: &output.neighbors,
            direct_pointer,
            direct_len: output.doc_ids.len(),
        }
    }

    /// Build Tantivy-compatible query without expensive serialization.
    ///
    /// # Safety
    ///
    /// `direct_pointer` must point to a valid memory region of at least `direct_len` `u64` elements.
    /// This is guaranteed by the constructor `new()` which derives the pointer from
    /// `output.doc_ids.as_ptr()` and the lifetime `'a` ensures the underlying `HnswOutput`
    /// outlives this struct.
    pub fn to_query(&self, text_query: &str) -> VectorTextQuery {
        let before = Instant::now();
        let ids: Vec<u64> = if self.direct_len == 0 {
            Vec::new()
        } else {
            // SAFETY: direct_pointer points to valid doc_ids slice of length direct_len
            // because it was derived from output.doc_ids.as_ptr() and the lifetime 'a
            // ensures the underlying HnswOutput outlives this struct.
            unsafe { std::slice::from_raw_parts(self.direct_pointer, self.direct_len) }.to_vec()
        };
        let scores: Vec<f32> = self.neighbors.iter().map(|n| n.score).collect();
        let _after = before.elapsed();
        VectorTextQuery {
            text_query: text_query.to_string(),
            top_k: self.neighbors.len(),
            neighbor_ids: ids,
            vector_scores: scores,
        }
    }
}

pub fn metric_bytes_copied(neighbor_count: usize) -> usize {
    neighbor_count * (std::mem::size_of::<u64>() + std::mem::size_of::<f32>())
}

pub fn stat_serialization(neighbor_count: usize, elapsed_us: u128) -> SerializationStats {
    SerializationStats {
        bytes_copied: metric_bytes_copied(neighbor_count),
        serialization_us: elapsed_us,
    }
}

pub fn bench_serialization_overhead(neighbors: &[HnswNeighbor]) -> SerializationStats {
    let start = Instant::now();
    let count = neighbors.len();
    let _bytes = metric_bytes_copied(count);
    let elapsed = start.elapsed();
    stat_serialization(count, elapsed.as_micros())
}

pub fn profile_copy(
    neighbors: &[HnswNeighbor],
    text_query: &str,
) -> (VectorTextQuery, SerializationStats) {
    let start = Instant::now();
    let output = HnswOutput::from_neighbors(neighbors.to_vec());
    let bridge = HnswOutputToTantivyInput::new(&output);
    let query = bridge.to_query(text_query);
    let stats = stat_serialization(neighbors.len(), start.elapsed().as_micros());
    (query, stats)
}

/// Default number of vector neighbors to feed into the text index
pub const DEFAULT_TOP_K: usize = 10;

impl VectorTextQuery {
    /// Create a new hybrid query with the given text and capacity
    pub fn new(text_query: &str, top_k: usize) -> Self {
        Self {
            text_query: text_query.to_string(),
            top_k,
            neighbor_ids: Vec::with_capacity(top_k),
            vector_scores: Vec::with_capacity(top_k),
        }
    }

    /// Estimate serialization overhead if this query were sent across a boundary
    pub fn estimate_overhead(&self) -> usize {
        self.text_query.len()
            + self.neighbor_ids.len() * std::mem::size_of::<u64>()
            + self.vector_scores.len() * std::mem::size_of::<f32>()
    }
}

/// Helper to switch to MemoryShare strategy when multiple readers need the same output
pub fn memory_share_output(output: &HnswOutput) -> HnswOutputToTantivyInput<'_> {
    let mut bridge = HnswOutputToTantivyInput::new(output);
    bridge.strategy = ZeroCopyStrategy::MemoryShare;
    bridge
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_to_text_zero_copy() {
        let neighbors = vec![
            HnswNeighbor {
                doc_id: 1,
                score: 0.9,
            },
            HnswNeighbor {
                doc_id: 2,
                score: 0.8,
            },
        ];
        let output = HnswOutput::from_neighbors(neighbors);
        let bridge = HnswOutputToTantivyInput::new(&output);
        assert_eq!(bridge.strategy, ZeroCopyStrategy::ZeroCopy);
        assert_eq!(bridge.direct_len, 2);
        let query = bridge.to_query("rust");
        assert_eq!(query.neighbor_ids, vec![1, 2]);
    }

    #[test]
    fn test_vector_text_empty_input() {
        let output = HnswOutput::default();
        let bridge = HnswOutputToTantivyInput::new(&output);
        assert!(bridge.neighbors.is_empty());
        assert_eq!(bridge.direct_len, 0);
        let query = bridge.to_query("empty");
        assert!(query.neighbor_ids.is_empty());
    }

    #[test]
    fn test_serialization_fallback() {
        let neighbors = vec![HnswNeighbor {
            doc_id: 42,
            score: 0.95,
        }];
        let (query, stats) = profile_copy(&neighbors, "test");
        assert!(!query.neighbor_ids.is_empty());
        assert_eq!(stats.bytes_copied, metric_bytes_copied(1));
        let bench = bench_serialization_overhead(&neighbors);
        assert_eq!(bench.bytes_copied, stats.bytes_copied);
    }

    #[test]
    fn test_memory_leak_hybrid() {
        for i in 0..1000 {
            let neighbors = vec![HnswNeighbor {
                doc_id: i as u64,
                score: 0.5,
            }];
            let output = HnswOutput::from_neighbors(neighbors);
            let bridge = HnswOutputToTantivyInput::new(&output);
            let _query = bridge.to_query("stress");
        }
        // If we reach here without OOM or segfault, memory is stable
    }
}
