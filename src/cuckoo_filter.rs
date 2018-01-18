use rand::Rng;

use Hasher;
use bucket::Buckets;

#[derive(Debug)]
pub struct CuckooFilter {
    buckets: Buckets,
    max_kicks: usize,
    has_zero_fingerprint: bool,
}
impl CuckooFilter {
    pub fn new(
        fingerprint_bitwidth: usize,
        entries_per_bucket: usize,
        number_of_items_hint: usize,
        max_kicks: usize,
    ) -> Self {
        let mut number_of_buckets = number_of_items_hint / entries_per_bucket;
        for i in 0.. {
            if number_of_buckets <= (1 << i) {
                number_of_buckets = 1 << i;
                break;
            }
        }
        let buckets = Buckets::new(fingerprint_bitwidth, entries_per_bucket, number_of_buckets);
        CuckooFilter {
            buckets,
            max_kicks,
            has_zero_fingerprint: false,
        }
    }
    pub fn try_insert<H: Hasher, R: Rng>(
        &mut self,
        hasher: &H,
        rng: &mut R,
        item_hash: u64,
        fingerprint: u64,
    ) -> bool {
        let fingerprint = fingerprint & self.buckets.fingerprint_mask();
        if fingerprint == 0 {
            self.has_zero_fingerprint = true;
            return true;
        }

        let i0 = item_hash as usize % self.buckets.len();
        let i1 = (i0 ^ hasher.hash(&fingerprint) as usize) % self.buckets.len();
        debug_assert_eq!(
            i0,
            (i1 ^ hasher.hash(&fingerprint) as usize) % self.buckets.len()
        );
        if self.buckets.contains(i0, fingerprint) || self.buckets.contains(i1, fingerprint) {
            true
        } else {
            self.try_insert_fingerprint(hasher, rng, i0, i1, fingerprint)
        }
    }
    pub fn contains<H: Hasher>(&self, hasher: &H, item_hash: u64, fingerprint: u64) -> bool {
        let fingerprint = fingerprint & self.buckets.fingerprint_mask();
        if fingerprint == 0 {
            return self.has_zero_fingerprint;
        }

        let i0 = item_hash as usize % self.buckets.len();
        let i1 = (i0 ^ hasher.hash(&fingerprint) as usize) % self.buckets.len();
        self.buckets.contains(i0, fingerprint) || self.buckets.contains(i1, fingerprint)
    }
    pub fn bits(&self) -> u64 {
        self.buckets.bits()
    }
    fn try_insert_fingerprint<H: Hasher, R: Rng>(
        &mut self,
        hasher: &H,
        rng: &mut R,
        i0: usize,
        i1: usize,
        mut fingerprint: u64,
    ) -> bool {
        if self.buckets.try_insert(i0, fingerprint) {
            true
        } else if self.buckets.try_insert(i1, fingerprint) {
            true
        } else {
            let mut i = if rng.gen::<bool>() { i0 } else { i1 };
            for _ in 0..self.max_kicks {
                fingerprint = self.buckets.random_swap(rng, i, fingerprint);
                i = (i ^ hasher.hash(&fingerprint) as usize) % self.buckets.len();
                if self.buckets.try_insert(i, fingerprint) {
                    return true;
                }
            }
            false
        }
    }
}
