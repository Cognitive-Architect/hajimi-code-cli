//! Batch distance computation and layer caching for HNSW
#![deny(unsafe_code)]

use std::collections::HashMap;

/// Euclidean distance between two 384-dim vectors
pub fn euclidean_distance(a: &[f32; 384], b: &[f32; 384]) -> f32 {
    let mut sum = 0.0_f32;
    for i in 0..384 {
        let d = a[i] - b[i];
        sum += d * d;
    }
    sum.sqrt()
}

/// Batch compute distances from query to multiple candidates
/// Returns vector of (distance, candidate_id) sorted by distance
pub fn batch_compute_distances<F>(
    query: &[f32; 384],
    candidate_ids: &[String],
    vector_lookup: &mut F,
) -> Vec<(f32, String)>
where
    F: FnMut(&str) -> Option<[f32; 384]>,
{
    let mut results: Vec<(f32, String)> = candidate_ids
        .iter()
        .filter_map(|id| {
            vector_lookup(id).map(|vec| {
                let dist = euclidean_distance(query, &vec);
                (dist, id.clone())
            })
        })
        .collect();

    // Sort by distance ascending
    results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    results
}

/// Simple LRU cache for hot layer nodes
pub struct LayerCache {
    cache: HashMap<String, ([f32; 384], usize)>,
    access_order: Vec<String>,
    capacity: usize,
    hits: usize,
    misses: usize,
    access_counter: usize,
}

impl LayerCache {
    /// Create a new LayerCache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(capacity),
            access_order: Vec::with_capacity(capacity),
            capacity,
            hits: 0,
            misses: 0,
            access_counter: 0,
        }
    }

    /// Get a vector from cache by ID
    pub fn get(&mut self, id: &str) -> Option<[f32; 384]> {
        if let Some(&(vec, _)) = self.cache.get(id) {
            self.hits += 1;
            // Update access time
            self.access_counter += 1;
            self.cache
                .insert(id.to_string(), (vec, self.access_counter));
            Some(vec)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Put a vector into cache
    pub fn put(&mut self, id: String, vector: [f32; 384]) {
        if self.cache.contains_key(&id) {
            // Update existing entry
            self.access_counter += 1;
            self.cache.insert(id.clone(), (vector, self.access_counter));
        } else {
            // Evict oldest if at capacity
            if self.cache.len() >= self.capacity && !self.access_order.is_empty() {
                // Find oldest entry
                let oldest_id = self
                    .access_order
                    .iter()
                    .find(|key| self.cache.contains_key(*key))
                    .cloned();
                if let Some(oldest) = oldest_id {
                    self.cache.remove(&oldest);
                }
            }
            self.access_counter += 1;
            self.cache.insert(id.clone(), (vector, self.access_counter));
            self.access_order.push(id);
        }
    }

    /// Get cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Get hits and misses stats
    pub fn stats(&self) -> (usize, usize) {
        (self.hits, self.misses)
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.hits = 0;
        self.misses = 0;
        self.access_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_distance() {
        let a = [0.0_f32; 384];
        let mut b = [0.0_f32; 384];
        b[0] = 3.0;
        b[1] = 4.0;
        let dist = euclidean_distance(&a, &b);
        assert!((dist - 5.0).abs() < 0.001, "Expected ~5.0, got {}", dist);
    }

    #[test]
    fn test_batch_compute_distances() {
        let query = [1.0_f32; 384];
        let candidates = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let mut lookup = |id: &str| -> Option<[f32; 384]> {
            match id {
                "a" => Some([0.0_f32; 384]),
                "b" => Some([2.0_f32; 384]),
                "c" => None,
                _ => None,
            }
        };

        let results = batch_compute_distances(&query, &candidates, &mut lookup);
        assert_eq!(results.len(), 2);
        // Distance to "a" (all zeros) should be smaller than to "b" (all 2s)
        assert_eq!(results[0].1, "a");
        assert_eq!(results[1].1, "b");
    }

    #[test]
    fn test_layer_cache() {
        let mut cache = LayerCache::new(2);

        // Initial insertions (no hits or misses yet)
        cache.put("a".to_string(), [1.0_f32; 384]);
        cache.put("b".to_string(), [2.0_f32; 384]);

        // These gets should count as hits
        assert!(cache.get("a").is_some());
        assert!(cache.get("b").is_some());

        // This get should count as a miss (not in cache)
        assert!(cache.get("nonexistent").is_none());

        // Add third item, should evict oldest
        cache.put("c".to_string(), [3.0_f32; 384]);

        // Cache hit rate should be tracked
        let (hits, misses) = cache.stats();
        assert!(hits > 0, "Expected hits > 0, got {}", hits);
        assert!(misses > 0, "Expected misses > 0, got {}", misses);

        let rate = cache.hit_rate();
        assert!(rate >= 0.0 && rate <= 1.0);
    }

    #[test]
    fn test_zero_unsafe() {
        // This test ensures no unsafe code is used
        // The #![deny(unsafe_code)] directive enforces this at compile time
        let _cache = LayerCache::new(10);
    }
}
