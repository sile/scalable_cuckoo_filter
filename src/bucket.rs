use rand;

#[derive(Debug)]
pub struct Buckets {
    fingerprint_bitwidth: usize, // fingerprint length in bits
    entries_per_bucket: usize,   // number of entries per bucket
    bucket_bitwidth: usize,
    bytes: Vec<u8>,
}
impl Buckets {
    pub fn new(fingerprint_bitwidth: usize, entries_per_bucket: usize, buckets: usize) -> Self {
        let bucket_bitwidth = 1 + fingerprint_bitwidth * entries_per_bucket;
        let buckets_bitwidth = bucket_bitwidth * buckets;
        let bytes = vec![0; (buckets_bitwidth + 7) / 8];
        Buckets {
            fingerprint_bitwidth,
            entries_per_bucket,
            bucket_bitwidth,
            bytes,
        }
    }
    pub fn bits(&self) -> u64 {
        self.bytes.len() as u64 * 8
    }

    pub fn fingerprint_mask(&self) -> u64 {
        (1 << self.fingerprint_bitwidth) - 1
    }

    pub fn contains(&self, bucket_index: usize, fingerprint: u64) -> bool {
        if fingerprint == 0 {
            return self.contains_zero_fingerprint(bucket_index);
        }
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
        if fingerprint == 0 {
            self.insert_zero_fingerprint(bucket_index);
            return true;
        }
        for i in 0..self.entries_per_bucket {
            let f = self.get_fingerprint(bucket_index, i);
            if f == 0 {
                self.set_fingerprint(bucket_index, i, fingerprint);
                return true;
            }
        }
        false
    }

    pub fn random_swap(&mut self, bucket_index: usize, fingerprint: u64) -> u64 {
        let i = rand::random::<usize>() % self.entries_per_bucket;
        let f = self.get_fingerprint(bucket_index, i);
        debug_assert_ne!(fingerprint, 0);
        debug_assert_ne!(f, 0);
        self.set_fingerprint(bucket_index, i, fingerprint);
        f
    }

    fn set_fingerprint(&mut self, bucket_index: usize, entry_index: usize, mut fingerprint: u64) {
        let bit_offset =
            self.bucket_bitwidth * bucket_index + 1 + entry_index * self.fingerprint_bitwidth;
        let base = bit_offset / 8;
        let mut offset = bit_offset % 8;

        let mut remaining_bits = self.fingerprint_bitwidth;
        for b in &mut self.bytes[base..] {
            *b ^= ((*b >> offset) & ((1 << remaining_bits) - 1)) << offset;
            *b |= (fingerprint << offset) as u8;
            if remaining_bits <= 8 - offset {
                break;
            }
            remaining_bits -= 8 - offset;
            fingerprint >>= 8 - offset;
            offset = 0;
        }
    }
    fn get_fingerprint(&self, bucket_index: usize, entry_index: usize) -> u64 {
        let bit_offset =
            self.bucket_bitwidth * bucket_index + 1 + entry_index * self.fingerprint_bitwidth;
        let base = bit_offset / 8;
        let mut offset = bit_offset % 8;

        let mut f = 0;
        let mut filled_bits = 0;
        for &b in &self.bytes[base..] {
            f |= (u64::from(b) >> offset) << filled_bits;
            filled_bits += 8 - offset;
            if filled_bits >= self.fingerprint_bitwidth {
                break;
            }
            offset = 0;
        }
        f & self.fingerprint_mask()
    }
    fn insert_zero_fingerprint(&mut self, bucket_index: usize) {
        let bit_offset = self.bucket_bitwidth * bucket_index;
        let base = bit_offset / 8;
        let offset = bit_offset % 8;
        self.bytes[base] |= 1 << offset;
    }
    fn contains_zero_fingerprint(&self, bucket_index: usize) -> bool {
        let bit_offset = self.bucket_bitwidth * bucket_index;
        let base = bit_offset / 8;
        let offset = bit_offset % 8;
        ((self.bytes[base] >> offset) & 1) != 0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let buckets = Buckets::new(8, 4, 1000);
        assert_eq!(buckets.bits(), 33000);
    }
}
