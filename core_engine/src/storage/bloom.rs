//! Bloom Filter Store - High-Performance Probabilistic Cache
//! 
//! Provides O(1) lookups to quickly determine if a short code
//! definitely does NOT exist, avoiding unnecessary database queries.
//! 
//! Configuration:
//! - Capacity: 1,000,000 items
//! - False Positive Rate: 1%
//! - Memory Usage: ~1.2 MB

use bloomfilter::Bloom;
use parking_lot::Mutex;

/// Thread-safe Bloom Filter store for short code existence checks.
/// 
/// # Performance Characteristics
/// - Insert: O(k) where k = number of hash functions
/// - Lookup: O(k) where k = number of hash functions
/// - Memory: ~1.2 MB for 1M items at 1% FPR
pub struct BloomStore {
    filter: Mutex<Bloom<str>>,
}

impl BloomStore {
    /// Creates a new BloomStore configured for high-traffic URL shortening.
    /// 
    /// # Configuration
    /// - Capacity: 1,000,000 items (handles ~1M unique short codes)
    /// - False Positive Rate: 1% (acceptable trade-off for performance)
    pub fn new() -> Self {
        // Calculate optimal parameters:
        // - items_count: 1,000,000
        // - fp_rate: 0.01 (1%)
        let items_count = 1_000_000;
        let fp_rate = 0.01;
        
        let filter = Bloom::new_for_fp_rate(items_count, fp_rate);
        
        BloomStore {
            filter: Mutex::new(filter),
        }
    }

    /// Adds a short code to the Bloom Filter.
    /// 
    /// After this call, `contains()` will always return `true` for this key.
    /// This operation is thread-safe.
    pub fn add(&self, key: &str) {
        let mut filter = self.filter.lock();
        filter.set(key);
    }

    /// Checks if a short code might exist in the store.
    /// 
    /// # Returns
    /// - `false`: The key definitely does NOT exist (guaranteed)
    /// - `true`: The key might exist (with 1% false positive rate)
    /// 
    /// This operation is thread-safe and non-blocking.
    pub fn contains(&self, key: &str) -> bool {
        let filter = self.filter.lock();
        filter.check(key)
    }

    /// Returns the current memory usage estimation in bytes.
    pub fn memory_usage(&self) -> usize {
        // Approximate: bitmap_size + overhead
        let filter = self.filter.lock();
        filter.number_of_bits() / 8
    }
}

impl Default for BloomStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_add_and_contains() {
        let store = BloomStore::new();
        
        // Key not added yet
        assert!(!store.contains("abc123"));
        
        // Add key
        store.add("abc123");
        
        // Key should now be found
        assert!(store.contains("abc123"));
    }

    #[test]
    fn test_bloom_no_false_negatives() {
        let store = BloomStore::new();
        
        // Add multiple keys
        for i in 0..1000 {
            store.add(&format!("key{}", i));
        }
        
        // All keys must be found (no false negatives)
        for i in 0..1000 {
            assert!(store.contains(&format!("key{}", i)));
        }
    }
}
