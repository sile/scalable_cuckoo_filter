use rand::Rng;

use bits::Bits;

#[derive(Debug)]
pub struct Buckets {
    fingerprint_bitwidth: usize, // fingerprint length in bits
    entries_per_bucket: usize,   // number of entries per bucket
    bucket_bitwidth: usize,
    bucket_index_bitwidth: usize,
    bits: Bits,
}
impl Buckets {
    pub fn new(
        fingerprint_bitwidth: usize,
        entries_per_bucket: usize,
        number_of_buckets_hint: usize,
    ) -> Self {
        let bucket_index_bitwidth = (0..)
            .find(|i| number_of_buckets_hint <= (1usize << i))
            .expect("Never fails");
        let bucket_bitwidth = fingerprint_bitwidth * entries_per_bucket;
        let bits = Bits::new(bucket_bitwidth << bucket_index_bitwidth);
        Buckets {
            fingerprint_bitwidth,
            entries_per_bucket,
            bucket_bitwidth,
            bucket_index_bitwidth,
            bits,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        1 << self.bucket_index_bitwidth
    }

    #[inline]
    pub fn bits(&self) -> u64 {
        self.bits.len() as u64
    }

    #[inline]
    pub fn index_mask(&self) -> usize {
        (1 << self.bucket_index_bitwidth) - 1
    }

    #[inline]
    pub fn fingerprint_mask(&self) -> u64 {
        (1 << self.fingerprint_bitwidth) - 1
    }

    #[inline]
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

    #[inline]
    pub fn try_insert(&mut self, bucket_index: usize, fingerprint: u64) -> bool {
        debug_assert_ne!(fingerprint, 0);
        for i in 0..self.entries_per_bucket {
            let f = self.get_fingerprint(bucket_index, i);
            if f == 0 {
                self.set_fingerprint(bucket_index, i, fingerprint);
                return true;
            }
            debug_assert_ne!(f, fingerprint);
        }
        false
    }

    #[inline]
    pub fn random_swap<R: Rng>(
        &mut self,
        rng: &mut R,
        bucket_index: usize,
        fingerprint: u64,
    ) -> u64 {
        let i = rng.gen_range(0, self.entries_per_bucket);
        let f = self.get_fingerprint(bucket_index, i);
        self.set_fingerprint(bucket_index, i, fingerprint);

        debug_assert_ne!(fingerprint, 0);
        debug_assert_eq!(fingerprint, self.get_fingerprint(bucket_index, i));
        debug_assert_ne!(f, fingerprint);
        debug_assert_ne!(f, 0);
        f
    }

    #[inline]
    fn set_fingerprint(&mut self, bucket_index: usize, entry_index: usize, fingerprint: u64) {
        let offset = self.bucket_bitwidth * bucket_index + self.fingerprint_bitwidth * entry_index;
        self.bits
            .set_uint(offset, self.fingerprint_bitwidth, fingerprint);
    }

    #[inline]
    fn get_fingerprint(&self, bucket_index: usize, entry_index: usize) -> u64 {
        let offset = self.bucket_bitwidth * bucket_index + self.fingerprint_bitwidth * entry_index;
        self.bits.get_uint(offset, self.fingerprint_bitwidth)
    }
}

#[cfg(test)]
mod test {
    use rand;
    use super::*;

    #[test]
    fn it_works() {
        let mut buckets = Buckets::new(8, 4, 1000);
        assert_eq!(buckets.len(), 1024);
        assert_eq!(buckets.bits(), 1024 * 8 * 4);

        for i in 0..4 {
            assert!(!buckets.contains(333, 100 + i));
            assert!(buckets.try_insert(333, 100 + i));
            assert!(buckets.contains(333, 100 + i));
        }
        assert!(!buckets.try_insert(333, 104)); // full

        let old = buckets.random_swap(&mut rand::thread_rng(), 333, 104);
        assert!(buckets.contains(333, 104));
        assert!(!buckets.contains(333, old));
    }
}
