use std::hash::Hash;
use std::marker::PhantomData;
use rand::{self, Rng, ThreadRng};

use {DefaultHasher, Hasher};
use cuckoo_filter::CuckooFilter;

/// Builder for `ScalableCuckooFilter`.
#[derive(Debug)]
pub struct ScalableCuckooFilterBuilder<H = DefaultHasher, R = ThreadRng> {
    initial_capacity: usize,
    false_positive_probability: f64,
    entries_per_bucket: usize,
    max_kicks: usize,
    hasher: H,
    rng: R,
}
impl ScalableCuckooFilterBuilder<DefaultHasher, ThreadRng> {
    /// Makes a new `ScalableCuckooFilterBuilder` instance.
    pub fn new() -> Self {
        ScalableCuckooFilterBuilder {
            initial_capacity: 100_000,
            false_positive_probability: 0.001,
            entries_per_bucket: 4,
            max_kicks: 512,
            hasher: DefaultHasher,
            rng: rand::thread_rng(),
        }
    }
}
impl<H: Hasher, R: Rng> ScalableCuckooFilterBuilder<H, R> {
    /// Sets the initial capacity (i.e., the number of estimated maximum items) of this filter.
    ///
    /// The default value is `100_000`.
    pub fn initial_capacity(mut self, capacity: usize) -> Self {
        self.initial_capacity = capacity;
        self
    }

    /// Sets the expected upper bound of the false positive probability of this filter.
    ///
    /// The default value is `0.001`.
    ///
    /// # Panics
    ///
    /// This method panics if `probability` is not a non-negative number smaller than or equal to `1.0`.
    pub fn false_positive_probability(mut self, probability: f64) -> Self {
        assert!(0.0 < probability && probability <= 1.0);
        self.false_positive_probability = probability;
        self
    }

    /// Sets the number of entries per bucket of this filter.
    ///
    /// The default value is `4`.
    pub fn entries_per_bucket(mut self, n: usize) -> Self {
        self.entries_per_bucket = n;
        self
    }

    /// Sets the maximum number of relocations in an insertion.
    ///
    /// If this limit exceeded, the filter will be expanded.
    ///
    /// The default value is `512`.
    pub fn max_kicks(mut self, kicks: usize) -> Self {
        self.max_kicks = kicks;
        self
    }

    /// Sets the hasher of this filter.
    ///
    /// The default value if `DefaultHasher`.
    pub fn hasher<T: Hasher>(self, hasher: T) -> ScalableCuckooFilterBuilder<T, R> {
        ScalableCuckooFilterBuilder {
            initial_capacity: self.initial_capacity,
            false_positive_probability: self.false_positive_probability,
            entries_per_bucket: self.entries_per_bucket,
            max_kicks: self.max_kicks,
            hasher,
            rng: self.rng,
        }
    }

    /// Sets the random number generator of this filter.
    ///
    /// The default value is `rand::thread_rng()`.
    pub fn rng<T: Rng>(self, rng: T) -> ScalableCuckooFilterBuilder<H, T> {
        ScalableCuckooFilterBuilder {
            initial_capacity: self.initial_capacity,
            false_positive_probability: self.false_positive_probability,
            entries_per_bucket: self.entries_per_bucket,
            max_kicks: self.max_kicks,
            hasher: self.hasher,
            rng,
        }
    }

    /// Builds a `ScalableCuckooFilter` instance.
    pub fn finish<T: Hash + ?Sized>(self) -> ScalableCuckooFilter<T, H, R> {
        let initial_probability = self.false_positive_probability / 2.0;
        let fingerprint_bitwidth = ((1.0 / initial_probability).log2()
            + ((2 * self.entries_per_bucket) as f64).log2())
            .ceil() as usize;
        let filter = CuckooFilter::new(
            fingerprint_bitwidth,
            self.entries_per_bucket,
            self.initial_capacity,
            self.max_kicks,
        );
        ScalableCuckooFilter {
            hasher: self.hasher,
            rng: self.rng,
            false_positive_probability: self.false_positive_probability,
            capacity: self.initial_capacity,
            filters: vec![filter],
            item_count: 0,
            _item: PhantomData,
        }
    }
}

/// Scalable Cuckoo Filter.
#[derive(Debug)]
pub struct ScalableCuckooFilter<T: ?Sized, H = DefaultHasher, R = ThreadRng> {
    hasher: H,
    filters: Vec<CuckooFilter>,
    false_positive_probability: f64,
    capacity: usize,
    rng: R,
    item_count: usize,
    _item: PhantomData<T>,
}
impl<T: Hash + ?Sized> ScalableCuckooFilter<T> {
    /// Makes a new `ScalableCuckooFilter` instance.
    ///
    /// This is equivalent to the following expression:
    ///
    /// ```
    /// # use scalable_cuckoo_filter::ScalableCuckooFilterBuilder;
    /// # let initial_capacity = 10;
    /// # let false_positive_probability = 0.1;
    /// ScalableCuckooFilterBuilder::new()
    ///     .initial_capacity(initial_capacity)
    ///     .false_positive_probability(false_positive_probability)
    ///     .finish()
    /// # ;
    /// ```
    pub fn new(initial_capacity: usize, false_positive_probability: f64) -> Self {
        ScalableCuckooFilterBuilder::new()
            .initial_capacity(initial_capacity)
            .false_positive_probability(false_positive_probability)
            .finish()
    }
}
impl<T: Hash + ?Sized, H: Hasher, R: Rng> ScalableCuckooFilter<T, H, R> {
    /// Returns the number of bits being used by this filter.
    pub fn bits(&self) -> u64 {
        self.filters.iter().map(|f| f.bits()).sum()
    }

    /// Returns the approximate number of items inserted in this filter.
    pub fn len(&self) -> usize {
        self.item_count
    }

    /// Returns the capacity (i.e., the upper bound of acceptable items count) of this filter.
    ///
    /// "capacity" is upper bound of the number of items can be inserted into the filter without resizing.
    pub fn capacity(&self) -> usize {
        self.filters.iter().map(|f| f.entries()).sum()
    }

    /// Returns `true` if this filter may contain `item`, otherwise `false`.
    pub fn contains(&self, item: &T) -> bool {
        let item_hash = self.hasher.hash(item);
        let fingerprint = self.hasher.fingerprint(item);
        self.filters
            .iter()
            .any(|f| f.contains(&self.hasher, item_hash, fingerprint))
    }

    /// Inserts `item` into this filter.
    ///
    /// If the current filter becomes full, it will be expanded automatically.
    pub fn insert(&mut self, item: &T) {
        let item_hash = self.hasher.hash(item);
        let fingerprint = self.hasher.fingerprint(item);
        let last = self.filters.len() - 1;
        for (i, filter) in self.filters.iter_mut().enumerate() {
            if i == last {
                if filter.try_insert(&self.hasher, &mut self.rng, item_hash, fingerprint) {
                    return;
                } else {
                    break;
                }
            } else {
                if filter.contains(&self.hasher, item_hash, fingerprint) {
                    return;
                }
            }
        }

        self.capacity *= 2;
        let probability =
            self.false_positive_probability / 2f64.powi(self.filters.len() as i32 + 1);
        let max_kicks = 512;
        let entries_per_bucket = 4;
        let fingerprint_bitwidth =
            ((1.0 / probability).log2() + ((2 * entries_per_bucket) as f64).log2()).ceil() as usize;
        let mut filter = CuckooFilter::new(
            fingerprint_bitwidth,
            entries_per_bucket,
            self.capacity,
            max_kicks,
        );
        assert!(filter.try_insert(&self.hasher, &mut self.rng, item_hash, fingerprint));
        self.filters.push(filter);
    }
    pub fn contains(&self, item: &T) -> bool {
        let item_hash = self.hasher.hash(item);
        let fingerprint = self.hasher.fingerprint(item);
        self.filters
            .iter()
            .any(|f| f.contains(&self.hasher, item_hash, fingerprint))
    }
    pub fn bits(&self) -> u64 {
        self.filters.iter().map(|f| f.bits()).sum()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let mut filter = ScalableCuckooFilter::new(1000, 0.001);
        assert_eq!(filter.bits(), 14_336);

        assert!(!filter.contains("foo"));
        filter.insert("foo");
        assert!(filter.contains("foo"));
    }

    #[test]
    fn insert_works() {
        use rand::{SeedableRng, StdRng};

        let rng: StdRng = SeedableRng::from_seed(&[1, 2, 3, 4][..]);
        let mut filter = ScalableCuckooFilterBuilder::new()
            .initial_capacity(100)
            .false_positive_probability(0.00001)
            .rng(rng)
            .finish();
        for i in 0..10_000 {
            assert!(!filter.contains(&i));
            filter.insert(&i);
            assert!(filter.contains(&i));
        }
    }
}
