use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use siphasher::sip::SipHasher13;
use rand::{self, Rng, ThreadRng};

use hash;
use cuckoo_filter::CuckooFilter;

/// Builder for `ScalableCuckooFilter`.
#[derive(Debug)]
pub struct ScalableCuckooFilterBuilder<H = SipHasher13, R = ThreadRng> {
    initial_capacity: usize,
    false_positive_probability: f64,
    entries_per_bucket: usize,
    max_kicks: usize,
    hasher: H,
    rng: R,
}
impl ScalableCuckooFilterBuilder<SipHasher13, ThreadRng> {
    /// Makes a new `ScalableCuckooFilterBuilder` instance.
    pub fn new() -> Self {
        ScalableCuckooFilterBuilder {
            initial_capacity: 100_000,
            false_positive_probability: 0.001,
            entries_per_bucket: 4,
            max_kicks: 512,
            hasher: SipHasher13::new(),
            rng: rand::thread_rng(),
        }
    }
}
impl<H: Hasher + Clone, R: Rng> ScalableCuckooFilterBuilder<H, R> {
    /// Sets the initial capacity (i.e., the number of estimated maximum items) of this filter.
    ///
    /// The default value is `100_000`.
    pub fn initial_capacity(mut self, capacity_hint: usize) -> Self {
        self.initial_capacity = capacity_hint;
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
    /// The default value if `SipHasher13::new()`.
    pub fn hasher<T: Hasher + Clone>(self, hasher: T) -> ScalableCuckooFilterBuilder<T, R> {
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
        let mut filter = ScalableCuckooFilter {
            hasher: self.hasher,
            rng: self.rng,
            initial_capacity: self.initial_capacity,
            false_positive_probability: self.false_positive_probability,
            entries_per_bucket: self.entries_per_bucket,
            max_kicks: self.max_kicks,
            filters: Vec::new(),
            _item: PhantomData,
        };
        filter.grow();
        filter
    }
}
impl Default for ScalableCuckooFilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Scalable Cuckoo Filter.
#[derive(Debug)]
pub struct ScalableCuckooFilter<T: ?Sized, H = SipHasher13, R = ThreadRng> {
    hasher: H,
    filters: Vec<CuckooFilter>,
    initial_capacity: usize,
    false_positive_probability: f64,
    entries_per_bucket: usize,
    max_kicks: usize,
    rng: R,
    _item: PhantomData<T>,
}
impl<T: Hash + ?Sized> ScalableCuckooFilter<T> {
    /// Makes a new `ScalableCuckooFilter` instance.
    ///
    /// This is equivalent to the following expression:
    ///
    /// ```
    /// # use scalable_cuckoo_filter::{ScalableCuckooFilter, ScalableCuckooFilterBuilder};
    /// # let initial_capacity = 10;
    /// # let false_positive_probability = 0.1;
    /// # let _: ScalableCuckooFilter<()> =
    /// ScalableCuckooFilterBuilder::new()
    ///     .initial_capacity(initial_capacity)
    ///     .false_positive_probability(false_positive_probability)
    ///     .finish()
    /// # ;
    /// ```
    pub fn new(initial_capacity_hint: usize, false_positive_probability: f64) -> Self {
        ScalableCuckooFilterBuilder::new()
            .initial_capacity(initial_capacity_hint)
            .false_positive_probability(false_positive_probability)
            .finish()
    }
}
impl<T: Hash + ?Sized, H: Hasher + Clone, R: Rng> ScalableCuckooFilter<T, H, R> {
    /// Returns the approximate number of items inserted in this filter.
    pub fn len(&self) -> usize {
        self.filters.iter().map(|f| f.len()).sum()
    }

    /// Returns `true` if this filter contains no items, otherwise `false`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the capacity (i.e., the upper bound of acceptable items count) of this filter.
    ///
    /// "capacity" is upper bound of the number of items can be inserted into the filter without resizing.
    pub fn capacity(&self) -> usize {
        self.filters.iter().map(|f| f.capacity()).sum()
    }

    /// Returns the number of bits being used for representing this filter.
    pub fn bits(&self) -> u64 {
        self.filters.iter().map(|f| f.bits()).sum()
    }

    /// Returns `true` if this filter may contain `item`, otherwise `false`.
    pub fn contains(&self, item: &T) -> bool {
        let item_hash = hash(&self.hasher, item);
        self.filters
            .iter()
            .any(|f| f.contains(&self.hasher, item_hash))
    }

    /// Inserts `item` into this filter.
    ///
    /// If the current filter becomes full, it will be expanded automatically.
    pub fn insert(&mut self, item: &T) {
        let item_hash = hash(&self.hasher, item);
        let last = self.filters.len() - 1;
        for filter in self.filters.iter().take(last) {
            if filter.contains(&self.hasher, item_hash) {
                return;
            }
        }

        self.filters[last].insert(&self.hasher, &mut self.rng, item_hash);
        if self.filters[last].is_nearly_full() {
            self.grow();
        }
    }

    /// Shrinks the capacity of this filter as much as possible.
    pub fn shrink_to_fit(&mut self) {
        for f in &mut self.filters {
            f.shrink_to_fit(&self.hasher, &mut self.rng);
        }
    }

    fn grow(&mut self) {
        let capacity = self.initial_capacity * 2usize.pow(self.filters.len() as u32);
        let probability =
            self.false_positive_probability / 2f64.powi(self.filters.len() as i32 + 1);
        let fingerprint_bitwidth = ((1.0 / probability).log2()
            + ((2 * self.entries_per_bucket) as f64).log2())
            .ceil() as usize;
        let filter = CuckooFilter::new(
            fingerprint_bitwidth,
            self.entries_per_bucket,
            capacity,
            self.max_kicks,
        );
        self.filters.push(filter);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let mut filter = ScalableCuckooFilter::new(1000, 0.001);
        assert!(filter.is_empty());
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
        assert_eq!(filter.len(), 10_000);
    }

    #[test]
    fn shrink_to_fit_works() {
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
    }
}
