use bloomfilter::Bloom;
use parking_lot::Mutex;

pub struct BloomStore {
    filter: Mutex<Bloom<String>>,
}

impl BloomStore {
    pub fn new() -> Self {
        let items_count = 1_000_000;
        let fp_rate = 0.01;
        let filter = Bloom::new_for_fp_rate(items_count, fp_rate);
        BloomStore {
            filter: Mutex::new(filter),
        }
    }

    pub fn add(&self, key: &str) {
        let mut filter = self.filter.lock();
        filter.set(&key.to_string());
    }

    pub fn contains(&self, key: &str) -> bool {
        let filter = self.filter.lock();
        filter.check(&key.to_string())
    }

    pub fn memory_usage(&self) -> usize {
        let filter = self.filter.lock();
        (filter.number_of_bits() / 8) as usize
    }
}

impl Default for BloomStore {
    fn default() -> Self {
        Self::new()
    }
}
