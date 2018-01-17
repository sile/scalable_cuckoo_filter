extern crate rand;
extern crate siphasher;

#[derive(Debug)]
pub struct ScalableCuckooFilter {}

mod bits;
mod bucket;
mod cuckoo_filter;
