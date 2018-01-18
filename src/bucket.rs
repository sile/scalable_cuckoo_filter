use rand::Rng;

use bits::Bits;

#[derive(Debug)]
pub struct Buckets {
    fingerprint_bitwidth: usize, // fingerprint length in bits
    entries_per_bucket: usize,   // number of entries per bucket
    bucket_bitwidth: usize,
    buckets: usize,
    bits: Bits,
}
impl Buckets {
    pub fn new(fingerprint_bitwidth: usize, entries_per_bucket: usize, buckets: usize) -> Self {
        let bucket_bitwidth = fingerprint_bitwidth * entries_per_bucket;
        let buckets_bitwidth = bucket_bitwidth * buckets;
        let bits = Bits::new(buckets_bitwidth);
        Buckets {
            fingerprint_bitwidth,
            entries_per_bucket,
            bucket_bitwidth,
            buckets,
            bits,
        }
    }
    pub fn len(&self) -> usize {
        self.buckets
    }
    pub fn bits(&self) -> u64 {
        self.bits.capacity() as u64
    }

    pub fn fingerprint_mask(&self) -> u64 {
        (1 << self.fingerprint_bitwidth) - 1
    }

    pub fn contains(&self, bucket_index: usize, fingerprint: u64) -> bool {
        debug_assert_ne!(fingerprint, 0);
        for i in 0..self.entries_per_bucket {
            let f = self.get_fingerprint(bucket_index, i);
            if f == fingerprint {
                return true;
            } else if f == 0 {
                break;
            }
        }
        false
    }

    pub fn try_insert(&mut self, bucket_index: usize, fingerprint: u64) -> bool {
        debug_assert_ne!(fingerprint, 0);
        for i in 0..self.entries_per_bucket {
            let f = self.get_fingerprint(bucket_index, i);
            if f == 0 {
                self.set_fingerprint(bucket_index, i, fingerprint);
                return true;
            }
        }
        false
    }

    pub fn random_swap<R: Rng>(
        &mut self,
        rng: &mut R,
        bucket_index: usize,
        fingerprint: u64,
    ) -> u64 {
        debug_assert_ne!(fingerprint, 0);
        let i = rng.gen_range(0, self.entries_per_bucket);
        let f = self.get_fingerprint(bucket_index, i);
        self.set_fingerprint(bucket_index, i, fingerprint);
        debug_assert_eq!(fingerprint, self.get_fingerprint(bucket_index, i));
        debug_assert_ne!(f, fingerprint);
        debug_assert_ne!(f, 0);
        f
    }

    fn set_fingerprint(&mut self, bucket_index: usize, entry_index: usize, fingerprint: u64) {
        let offset = self.bucket_bitwidth * bucket_index + self.fingerprint_bitwidth * entry_index;
        self.bits
            .set_uint(offset, self.fingerprint_bitwidth, fingerprint);
    }
    fn get_fingerprint(&self, bucket_index: usize, entry_index: usize) -> u64 {
        let offset = self.bucket_bitwidth * bucket_index + self.fingerprint_bitwidth * entry_index;
        self.bits.get_uint(offset, self.fingerprint_bitwidth)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let buckets = Buckets::new(8, 4, 1000);
        assert_eq!(buckets.bits(), 32000);
    }
}
