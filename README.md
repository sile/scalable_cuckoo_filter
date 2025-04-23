scalable_cuckoo_filter
======================

[![scalable_cuckoo_filter](https://img.shields.io/crates/v/scalable_cuckoo_filter.svg)](https://crates.io/crates/scalable_cuckoo_filter)
[![Documentation](https://docs.rs/scalable_cuckoo_filter/badge.svg)](https://docs.rs/scalable_cuckoo_filter)
[![Actions Status](https://github.com/sile/scalable_cuckoo_filter/workflows/CI/badge.svg)](https://github.com/sile/scalable_cuckoo_filter/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A variant of [Cuckoo Filter][cuckoo filter] whose size automatically scales as necessary.

[Documentation](https://docs.rs/scalable_cuckoo_filter)

Examples
--------

Basic usage:

```rust
use scalable_cuckoo_filter::ScalableCuckooFilter;

let mut filter = ScalableCuckooFilter::new(100, 0.001);
assert!(!filter.contains("foo"));
filter.insert("foo");
assert!(filter.contains("foo"));
```

Filter grows automatically:

```rust
use scalable_cuckoo_filter::ScalableCuckooFilter;

let mut filter = ScalableCuckooFilter::new(100, 0.001);
assert_eq!(filter.capacity(), 128);

for i in 0..1000 {
    filter.insert(&i);
}
assert_eq!(filter.capacity(), 1923);
```

Filter shrinking:

```rust
use scalable_cuckoo_filter::ScalableCuckooFilter;

let mut filter = ScalableCuckooFilter::new(1000, 0.001);
for i in 0..100 {
    filter.insert(&i);
}
assert_eq!(filter.capacity(), 1024);
assert_eq!(filter.bits(), 14336);

filter.shrink_to_fit();
for i in 0..100 {
    assert!(filter.contains(&i));
}
assert_eq!(filter.capacity(), 128);
assert_eq!(filter.bits(), 1792);
```

References
----------

- [Cuckoo Filter: Practically Better Than Bloom][cuckoo filter]
- [Scalable Bloom Filters][scalable bloom filters]

[cuckoo filter]: https://www.cs.cmu.edu/~dga/papers/cuckoo-conext2014.pdf
[scalable bloom filters]: http://haslab.uminho.pt/cbm/files/dbloom.pdf
