//! Simple 64-bit SimHash for sharding (byte-hash based).
//! Unified from duplicates in tantivy_index.rs and adr_index.rs (B-08).
//! Zero behavior change - exact same algorithm preserved for regression safety.
//! NUM_SHARDS = 16 for consistent sharding across knowledge graph and search index.
//!
//! Used for deterministic shard routing based on content ID or text key.
//! Part of foundation layer per ADR-008.

pub const NUM_SHARDS: usize = 16;

/// Computes a simple 64-bit hash for text (SimHash approximation using prime multiplier).
/// Exact match to previous duplicate implementations for zero behavior change.
pub fn simhash64(text: &str) -> u64 {
    let mut hash: u64 = 0;
    for (i, byte) in text.bytes().enumerate() {
        hash = hash.wrapping_add((byte as u64).wrapping_mul(0x9e3779b97f4a7c15 + i as u64));
    }
    hash
}

/// Determines shard ID using simhash64 modulo NUM_SHARDS.
pub fn get_shard_id(text: &str) -> usize {
    (simhash64(text) % NUM_SHARDS as u64) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simhash64_consistency() {
        assert_eq!(simhash64("test"), simhash64("test"));
        assert_ne!(simhash64("a"), simhash64("b"));
    }

    #[test]
    fn test_get_shard_id_range() {
        for i in 0..100 {
            let id = get_shard_id(&format!("doc_{}", i));
            assert!(id < NUM_SHARDS);
        }
    }

    #[test]
    fn test_shard_distribution() {
        let mut counts = vec![0; NUM_SHARDS];
        for i in 0..1000 {
            counts[get_shard_id(&format!("key_{}", i))] += 1;
        }
        for &count in &counts {
            assert!(count > 0, "Some shards empty - bad hash");
        }
    }
}
