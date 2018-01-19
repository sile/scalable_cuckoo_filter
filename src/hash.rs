use std::hash::{Hash, Hasher as StdHasher};
use siphasher::sip::SipHasher13;

/// Hash function for cuckoo filters.
pub trait Hasher {
    /// Calculates the hash value of `item`.
    fn hash<T: Hash + ?Sized>(&self, item: &T) -> u64;

    /// Calculates the fingerprint of `item`.
    ///
    /// This should return a hash value that differed from the result of `Hasher::hash` method.
    fn fingerprint<T: Hash + ?Sized>(&self, item: &T) -> u64;
}

// Arbitrary prefix to make the result of `fingerprint()` different from the result of `hash()`.
const FINGERPRINT_PREFIX: u8 = b'F';

/// The default implementation of `Hasher` trait.
#[derive(Debug)]
pub struct DefaultHasher;
impl Hasher for DefaultHasher {
    fn hash<T: Hash + ?Sized>(&self, item: &T) -> u64 {
        let mut hasher = SipHasher13::new();
        item.hash(&mut hasher);
        hasher.finish()
    }
    fn fingerprint<T: Hash + ?Sized>(&self, item: &T) -> u64 {
        let mut hasher = SipHasher13::new();
        (FINGERPRINT_PREFIX, item).hash(&mut hasher);
        hasher.finish()
    }
}
