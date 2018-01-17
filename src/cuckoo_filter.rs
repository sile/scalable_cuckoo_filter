#![allow(dead_code)] // TODO: remove
use std::hash::{BuildHasher, Hash, Hasher};
use rand;

use bucket::Buckets;

#[derive(Debug)]
pub struct CuckooFilter {
    buckets: Buckets,
    max_kicks: usize,
}
impl CuckooFilter {
    pub fn new(
        fingerprint_bitwidth: usize,
        entries_per_bucket: usize,
        number_of_items_hint: usize,
        max_kicks: usize,
    ) -> Self {
        let number_of_buckets = number_of_items_hint / entries_per_bucket;
        let buckets = Buckets::new(fingerprint_bitwidth, entries_per_bucket, number_of_buckets);
        CuckooFilter { buckets, max_kicks }
    }
    pub fn try_insert<T: Hash, H: BuildHasher>(&mut self, item: &T, hasher: &H) -> bool {
        let (fingerprint, i0, i1) = self.calculate_fingerprint_and_indices(item, hasher);
        if self.buckets.contains(i0, fingerprint) || self.buckets.contains(i1, fingerprint) {
            true
        } else {
            self.try_insert_fingerprint(i0, i1, fingerprint, hasher)
        }
    }
    pub fn contains<T: Hash, H: BuildHasher>(&self, item: &T, hasher: &H) -> bool {
        let (fingerprint, i0, i1) = self.calculate_fingerprint_and_indices(item, hasher);
        self.buckets.contains(i0, fingerprint) || self.buckets.contains(i1, fingerprint)
    }
    pub fn bits(&self) -> u64 {
        self.buckets.bits()
    }
    fn calculate_fingerprint_and_indices<T: Hash, H: BuildHasher>(
        &self,
        item: &T,
        hasher: &H,
    ) -> (u64, usize, usize) {
        let fingerprint = hash(&(0, item), hasher) & self.buckets.fingerprint_mask();
        let i0 = hash(&(1, item), hasher);
        let i1 = i0 ^ hash(&fingerprint, hasher);
        (fingerprint, i0 as usize, i1 as usize)
    }
    fn try_insert_fingerprint<H: BuildHasher>(
        &mut self,
        i0: usize,
        i1: usize,
        mut fingerprint: u64,
        hasher: &H,
    ) -> bool {
        if self.buckets.try_insert(i0, fingerprint) {
            true
        } else if self.buckets.try_insert(i0, fingerprint) {
            true
        } else {
            let mut i = if rand::random::<bool>() { i0 } else { i1 };
            for _ in 0..self.max_kicks {
                fingerprint = self.buckets.random_swap(i, fingerprint);
                i = i ^ hash(&fingerprint, hasher) as usize;
                if self.buckets.try_insert(i, fingerprint) {
                    return true;
                }
            }
            false
        }
    }
}

fn hash<T: Hash, H: BuildHasher>(item: &T, hasher: &H) -> u64 {
    let mut hasher = hasher.build_hasher();
    item.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let mut filter = CuckooFilter::new(9, 4, 100, 32);
    }
}
