use std::hash::Hasher;
use rand::Rng;

use hash;
use buckets::Buckets;

#[derive(Debug)]
pub struct CuckooFilter {
    buckets: Buckets,
    max_kicks: usize,
    has_zero_fingerprint: bool,
    kicked_fingerprint: Option<u64>,
}
impl CuckooFilter {
    pub fn new(
        fingerprint_bitwidth: usize,
        entries_per_bucket: usize,
        number_of_items_hint: usize,
        max_kicks: usize,
    ) -> Self {
        let number_of_buckets_hint =
            (number_of_items_hint + entries_per_bucket - 1) / entries_per_bucket;
        let buckets = Buckets::new(
            fingerprint_bitwidth,
            entries_per_bucket,
            number_of_buckets_hint,
        );
        CuckooFilter {
            buckets,
            max_kicks,
            has_zero_fingerprint: false,
            kicked_fingerprint: None,
        }
    }

    #[inline]
    pub fn bits(&self) -> u64 {
        self.buckets.bits()
    }

    #[inline]
    pub fn entries(&self) -> usize {
        self.buckets.entries()
    }

    #[inline]
    pub fn contains<H: Hasher + Clone>(&self, hasher: &H, item_hash: u64) -> bool {
        let fingerprint = self.buckets.fingerprint(item_hash);
        if fingerprint == 0 {
            return self.has_zero_fingerprint;
        }
        if Some(fingerprint) == self.kicked_fingerprint {
            return true;
        }

        let i0 = self.buckets.index(item_hash);
        let i1 = self.buckets.index(i0 as u64 ^ hash(hasher, &fingerprint));
        self.buckets.contains(i0, fingerprint) || self.buckets.contains(i1, fingerprint)
    }

    #[inline]
    pub fn try_insert<H: Hasher + Clone, R: Rng>(
        &mut self,
        hasher: &H,
        rng: &mut R,
        item_hash: u64,
    ) -> bool {
        let fingerprint = self.buckets.fingerprint(item_hash);
        if fingerprint == 0 {
            self.has_zero_fingerprint = true;
            return true;
        }
        if Some(fingerprint) == self.kicked_fingerprint {
            return true;
        }

        let i0 = self.buckets.index(item_hash);
        let i1 = self.buckets.index(i0 as u64 ^ hash(hasher, &fingerprint));
        if self.buckets.contains(i0, fingerprint) || self.buckets.contains(i1, fingerprint) {
            true
        } else if self.kicked_fingerprint.is_some() {
            false
        } else {
            self.insert_fingerprint(hasher, rng, i0, i1, fingerprint);
            true
        }
    }

    #[inline]
    fn insert_fingerprint<H: Hasher + Clone, R: Rng>(
        &mut self,
        hasher: &H,
        rng: &mut R,
        i0: usize,
        i1: usize,
        mut fingerprint: u64,
    ) {
        if self.buckets.try_insert(i0, fingerprint) || self.buckets.try_insert(i1, fingerprint) {
            return;
        }

        let mut i = if rng.gen::<bool>() { i0 } else { i1 };
        for _ in 0..self.max_kicks {
            fingerprint = self.buckets.random_swap(rng, i, fingerprint);
            i = self.buckets.index(i as u64 ^ hash(hasher, &fingerprint));
            if self.buckets.try_insert(i, fingerprint) {
                return;
            }
        }
        self.kicked_fingerprint = Some(fingerprint);
    }
}
