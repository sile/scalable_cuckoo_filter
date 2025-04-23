use rand::Rng;

use crate::bits::Bits;

#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
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
        let bucket_index_bitwidth =
            number_of_buckets_hint.next_power_of_two().trailing_zeros() as usize;
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
    pub fn required_number_of_buckets(number_of_buckets_hint: usize) -> usize {
        number_of_buckets_hint.next_power_of_two()
    }

    #[inline]
    pub fn len(&self) -> usize {
        1 << self.bucket_index_bitwidth
    }

    #[inline]
    pub fn entries(&self) -> usize {
        self.len() * self.entries_per_bucket
    }

    #[inline]
    pub fn bits(&self) -> u64 {
        self.bits.len() as u64
    }

    #[inline]
    pub fn index(&self, hash: u64) -> usize {
        (hash & ((1 << self.bucket_index_bitwidth) - 1)) as usize
    }

    #[inline]
    pub fn fingerprint(&self, hash: u64) -> u64 {
        hash >> (64 - self.fingerprint_bitwidth)
    }

    #[inline]
    pub fn entries_per_bucket(&self) -> usize {
        self.entries_per_bucket
    }

    #[inline]
    pub fn fingerprint_bitwidth(&self) -> usize {
        self.fingerprint_bitwidth
    }

    #[inline]
    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }

    #[inline]
    pub fn contains(&self, bucket_index: usize, fingerprint: u64) -> bool {
        debug_assert_ne!(fingerprint, 0);
        for i in 0..self.entries_per_bucket {
            let f = self.get_fingerprint(bucket_index, i);
            if f == fingerprint {
                return true;
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
        let i = rng.random_range(0..self.entries_per_bucket);
        let f = self.get_fingerprint(bucket_index, i);
        self.set_fingerprint(bucket_index, i, fingerprint);

        debug_assert_ne!(fingerprint, 0);
        debug_assert_eq!(fingerprint, self.get_fingerprint(bucket_index, i));
        debug_assert_ne!(f, 0);
        f
    }

    #[inline]
    pub fn remove_fingerprint(&mut self, bucket_index: usize, fingerprint: u64) -> bool {
        debug_assert_ne!(fingerprint, 0);
        for i in 0..self.entries_per_bucket {
            let f = self.get_fingerprint(bucket_index, i);
            if f == fingerprint {
                self.set_fingerprint(bucket_index, i, 0);
                return true;
            }
        }
        false
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

#[derive(Debug)]
pub struct Iter<'a> {
    buckets: &'a Buckets,
    bucket_i: usize,
    entry_i: usize,
}
impl<'a> Iter<'a> {
    fn new(buckets: &'a Buckets) -> Self {
        Iter {
            buckets,
            bucket_i: 0,
            entry_i: 0,
        }
    }
}
impl Iterator for Iter<'_> {
    type Item = (usize, u64);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.bucket_i == self.buckets.len() {
                return None;
            } else if self.entry_i == self.buckets.entries_per_bucket {
                self.bucket_i += 1;
                self.entry_i = 0;
            } else {
                let f = self.buckets.get_fingerprint(self.bucket_i, self.entry_i);
                if f == 0 {
                    self.bucket_i += 1;
                    self.entry_i = 0;
                } else {
                    self.entry_i += 1;
                    return Some((self.bucket_i, f));
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
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

        let old = buckets.random_swap(&mut rand::rng(), 333, 104);
        assert!(buckets.contains(333, 104));
        assert!(!buckets.contains(333, old));
    }
}
