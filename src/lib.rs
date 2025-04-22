//! A variant of [Cuckoo Filter][cuckoo filter] whose size automatically scales as necessary.
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```
//! use scalable_cuckoo_filter::ScalableCuckooFilter;
//!
//! let mut filter = ScalableCuckooFilter::<str>::new(100, 0.001);
//! assert!(!filter.contains("foo"));
//! filter.insert("foo");
//! assert!(filter.contains("foo"));
//! ```
//!
//! Filter grows automatically:
//!
//! ```
//! use scalable_cuckoo_filter::ScalableCuckooFilter;
//!
//! let mut filter = ScalableCuckooFilter::<usize>::new(100, 0.001);
//! assert_eq!(filter.capacity(), 128);
//!
//! for i in 0..1000 {
//!     filter.insert(&i);
//! }
//! assert_eq!(filter.capacity(), 1923);
//! ```
//!
//! Filter shrinking:
//!
//! ```
//! use scalable_cuckoo_filter::ScalableCuckooFilter;
//!
//! let mut filter = ScalableCuckooFilter::<usize>::new(1000, 0.001);
//! for i in 0..100 {
//!     filter.insert(&i);
//! }
//! assert_eq!(filter.capacity(), 1024);
//! assert_eq!(filter.bits(), 14336);
//!
//! filter.shrink_to_fit();
//! for i in 0..100 {
//!     assert!(filter.contains(&i));
//! }
//! assert_eq!(filter.capacity(), 128);
//! assert_eq!(filter.bits(), 1792);
//! ```
//!
//! # References
//!
//! - [Cuckoo Filter: Practically Better Than Bloom][cuckoo filter]
//! - [Scalable Bloom Filters][scalable bloom filters]
//!
//! [cuckoo filter]: https://www.cs.cmu.edu/~dga/papers/cuckoo-conext2014.pdf
//! [scalable bloom filters]: http://haslab.uminho.pt/cbm/files/dbloom.pdf
#![warn(missing_docs)]

pub use crate::scalable_cuckoo_filter::{
    DefaultHasher, DefaultRng, ScalableCuckooFilter, ScalableCuckooFilterBuilder,
};

mod bits;
mod buckets;
mod cuckoo_filter;
mod scalable_cuckoo_filter;

#[inline]
fn hash<T: ?Sized + std::hash::Hash, H: std::hash::Hasher + Clone>(hasher: &H, item: &T) -> u64 {
    let mut hasher = hasher.clone();
    item.hash(&mut hasher);
    hasher.finish()
}
