use rand::Rng;
use std::cmp;
use std::hash::Hasher;
use std::mem;

use crate::buckets::Buckets;

#[derive(Debug)]
pub struct CuckooFilter {
    buckets: Buckets,
    max_kicks: usize,
    exceptional_items: ExceptionalItems,
    item_count: usize,
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
            exceptional_items: ExceptionalItems::new(),
            item_count: 0,
        }
    }

    #[inline]
    pub fn bits(&self) -> u64 {
        self.buckets.bits() + self.exceptional_items.bits()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.item_count
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buckets.entries() + self.exceptional_items.len()
    }

    #[inline]
    pub fn is_nearly_full(&self) -> bool {
        self.exceptional_items.contains_kicked_out_entries()
    }

    #[inline]
    pub fn contains<H: Hasher + Clone>(&self, hasher: &H, item_hash: u64) -> bool {
        let fingerprint = self.buckets.fingerprint(item_hash);
        let i0 = self.buckets.index(item_hash);
        let i1 = self
            .buckets
            .index(i0 as u64 ^ crate::hash(hasher, &fingerprint));
        self.contains_fingerprint(i0, i1, fingerprint)
    }

    #[inline]
    pub fn insert<H: Hasher + Clone, R: Rng>(&mut self, hasher: &H, rng: &mut R, item_hash: u64) {
        let fingerprint = self.buckets.fingerprint(item_hash);
        let i0 = self.buckets.index(item_hash);
        self.insert_fingerprint(hasher, rng, i0, fingerprint);
    }

    #[inline]
    pub fn shrink_to_fit<H: Hasher + Clone, R: Rng>(&mut self, hasher: &H, rng: &mut R) {
        let entries_per_bucket = self.buckets.entries_per_bucket();
        let shrunk_buckets_len = Buckets::required_number_of_buckets(
            (self.item_count + entries_per_bucket - 1) / entries_per_bucket,
        );
        if shrunk_buckets_len < self.buckets.len() {
            let mut shrunk_filter = CuckooFilter::new(
                self.buckets.fingerprint_bitwidth(),
                self.buckets.entries_per_bucket(),
                self.item_count,
                self.max_kicks,
            );
            for (i, fingerprint) in self.buckets.iter() {
                let shrunk_i = shrunk_filter.buckets.index(i as u64);
                shrunk_filter.insert_fingerprint(hasher, rng, shrunk_i, fingerprint);
            }
            *self = shrunk_filter;
        }
        self.exceptional_items.shrink_to_fit();
    }

    #[inline]
    fn contains_fingerprint(&self, i0: usize, i1: usize, fingerprint: u64) -> bool {
        if self.exceptional_items.contains(i0, i1, fingerprint) {
            true
        } else if fingerprint == 0 {
            false
        } else {
            self.buckets.contains(i0, fingerprint) || self.buckets.contains(i1, fingerprint)
        }
    }

    #[inline]
    fn insert_fingerprint<H: Hasher + Clone, R: Rng>(
        &mut self,
        hasher: &H,
        rng: &mut R,
        i0: usize,
        fingerprint: u64,
    ) {
        let i1 = self
            .buckets
            .index(i0 as u64 ^ crate::hash(hasher, &fingerprint));
        if self.contains_fingerprint(i0, i1, fingerprint) {
            return;
        }
        self.item_count += 1;

        if fingerprint == 0 {
            self.exceptional_items.insert(i0, i1, 0);
            return;
        }
        if self.buckets.try_insert(i0, fingerprint) || self.buckets.try_insert(i1, fingerprint) {
            return;
        }

        let mut fingerprint = fingerprint;
        let mut i = if rng.gen::<bool>() { i0 } else { i1 };
        let mut prev_i = i;
        for _ in 0..self.max_kicks {
            fingerprint = self.buckets.random_swap(rng, i, fingerprint);
            prev_i = i;
            i = self
                .buckets
                .index(i as u64 ^ crate::hash(hasher, &fingerprint));
            if self.buckets.try_insert(i, fingerprint) {
                return;
            }
        }
        self.exceptional_items.insert(prev_i, i, fingerprint);
    }
}

#[derive(Debug)]
struct ExceptionalItems(Vec<(u64, usize)>);
impl ExceptionalItems {
    fn new() -> Self {
        ExceptionalItems(Vec::new())
    }

    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    fn bits(&self) -> u64 {
        (mem::size_of::<(u64, usize)>() * self.0.capacity()) as u64 * 8
    }

    #[inline]
    fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    #[inline]
    fn contains_kicked_out_entries(&self) -> bool {
        self.0
            .last()
            .map_or(false, |&(fingerprint, _)| fingerprint != 0)
    }

    #[inline]
    fn contains(&self, i0: usize, i1: usize, fingerprint: u64) -> bool {
        let item = (fingerprint, cmp::min(i0, i1));
        self.0.binary_search(&item).is_ok()
    }

    #[inline]
    fn insert(&mut self, i0: usize, i1: usize, fingerprint: u64) {
        let item = (fingerprint, cmp::min(i0, i1));
        for i in 0..self.0.len() {
            debug_assert_ne!(self.0[i], item);
            if item < self.0[i] {
                self.0.insert(i, item);
                return;
            }
        }
        self.0.push(item);
    }
}
