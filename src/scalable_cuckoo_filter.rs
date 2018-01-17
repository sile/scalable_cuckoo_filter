use std::hash::Hash;
use std::marker::PhantomData;

use {DefaultHasher, Hasher};
use cuckoo_filter::CuckooFilter;

// TODO: builder

#[derive(Debug)]
pub struct ScalableCuckooFilter<T: ?Sized, H = DefaultHasher> {
    hasher: H,
    filters: Vec<CuckooFilter>,
    false_positive_probability: f64,
    capacity: usize,
    _item: PhantomData<T>,
}
impl<T: Hash + ?Sized> ScalableCuckooFilter<T> {
    pub fn new(initial_capacity: usize, false_positive_probability: f64) -> Self {
        assert!(false_positive_probability > 0.0);
        assert!(false_positive_probability <= 1.0);

        let initial_probability = false_positive_probability / 2.0;
        let max_kicks = 512;
        let entries_per_bucket = 4;
        let fingerprint_bitwidth = ((1.0 / initial_probability).log2()
            + ((2 * entries_per_bucket) as f64).log2())
            .ceil() as usize;
        let filter = CuckooFilter::new(
            fingerprint_bitwidth,
            entries_per_bucket,
            initial_capacity,
            max_kicks,
        );
        ScalableCuckooFilter {
            hasher: DefaultHasher,
            filters: vec![filter],
            false_positive_probability,
            capacity: initial_capacity,
            _item: PhantomData,
        }
    }
}
impl<T: Hash + ?Sized, H: Hasher> ScalableCuckooFilter<T, H> {
    pub fn insert(&mut self, item: &T) {
        let item_hash = self.hasher.hash(item);
        let fingerprint = self.hasher.fingerprint(item);
        let last = self.filters.len() - 1;
        for (i, filter) in self.filters.iter_mut().enumerate() {
            if i == last {
                if filter.try_insert(&self.hasher, item_hash, fingerprint) {
                    return;
                } else {
                    break;
                }
            } else {
                if filter.contains(&self.hasher, item_hash, fingerprint) {
                    return;
                }
            }
        }

        self.capacity *= 2;
        let probability =
            self.false_positive_probability / 2f64.powi(self.filters.len() as i32 + 1);
        let max_kicks = 512;
        let entries_per_bucket = 4;
        let fingerprint_bitwidth =
            ((1.0 / probability).log2() + ((2 * entries_per_bucket) as f64).log2()).ceil() as usize;
        let mut filter = CuckooFilter::new(
            fingerprint_bitwidth,
            entries_per_bucket,
            self.capacity,
            max_kicks,
        );
        assert!(filter.try_insert(&self.hasher, item_hash, fingerprint));
        self.filters.push(filter);
    }
    pub fn contains(&self, item: &T) -> bool {
        let item_hash = self.hasher.hash(item);
        let fingerprint = self.hasher.fingerprint(item);
        self.filters
            .iter()
            .any(|f| f.contains(&self.hasher, item_hash, fingerprint))
    }
    pub fn bits(&self) -> u64 {
        self.filters.iter().map(|f| f.bits()).sum()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let mut filter = ScalableCuckooFilter::new(1000, 0.001);
        assert_eq!(filter.bits(), 14_000);

        assert!(!filter.contains("foo"));
        filter.insert("foo");
        assert!(filter.contains("foo"));
    }
}
