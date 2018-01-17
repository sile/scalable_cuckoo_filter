// TODO: Add a flag for semi-sorting optimization

#[derive(Debug)]
pub struct Buckets {
    fingerprint_bitwidth: usize, // fingerprint length in bits
    entries_per_bucket: usize,   // number of entries per bucket
    buckets: usize,              // number of buckets
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
            buckets,
            bytes,
        }
    }
    pub fn bits(&self) -> u64 {
        self.bytes.len() as u64 * 8
    }
    pub fn get(&self, index: usize) -> Bucket {
        Bucket {
            buckets: self,
            index,
        }
    }
    pub fn fingerprint_mask(&self) -> u64 {
        (1 << self.fingerprint_bitwidth) - 1
    }
}

#[derive(Debug)]
pub struct Bucket<'a> {
    buckets: &'a Buckets,
    index: usize,
}
impl<'a> Bucket<'a> {
    pub fn random_swap(&mut self, fingerprint: u64) -> u64 {
        unimplemented!()
    }
    pub fn contains(&self, fingerprint: u64) -> bool {
        unimplemented!()
    }
    pub fn try_insert(&mut self, fingerprint: u64) -> bool {
        unimplemented!()
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
