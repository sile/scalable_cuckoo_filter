use std::hash::{Hash, Hasher as StdHasher};
use siphasher::sip::SipHasher13;

/// Hash function for cuckoo filters.
pub trait Hasher {
    /// Calculates the hash value of `item`.
    fn hash<T: Hash + ?Sized>(&self, item: &T) -> u64;

    /// Calculates the fingerprint of `item`.
    fn fingerprint<T: Hash + ?Sized>(&self, item: &T) -> u64;
}

/// The default implementation of `Hasher` trait.
#[derive(Debug)]
pub struct DefaultHasher;
impl Hasher for DefaultHasher {
    fn hash<T: Hash + ?Sized>(&self, item: &T) -> u64 {
        let mut hasher = SipHasher13::new();
        (0, item).hash(&mut hasher);
        hasher.finish()
    }
    fn fingerprint<T: Hash + ?Sized>(&self, item: &T) -> u64 {
        let mut hasher = SipHasher13::new();
        (1, item).hash(&mut hasher);
        hasher.finish()
    }
}
