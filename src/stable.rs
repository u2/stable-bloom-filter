use crate::buckets::Buckets;
use crate::fnv::FnvHasher;
use crate::Filter;
use crate::{optimal_k, optimal_stable_p};
use rand::{thread_rng, Rng};
use std::hash::Hasher;

pub struct StableBloomFilter {
    /// filter data
    cells: Buckets,
    /// hash function (kernel for all k functions)
    hash: FnvHasher,
    /// number of cells
    m: usize,
    /// number of cells to decrement
    p: usize,
    /// number of hash functions
    k: usize,
    /// cell max value
    max: u8,
    /// buffer used to cache indices
    index_buffer: Vec<usize>,
}

impl StableBloomFilter {
    /// Creates a new Stable Bloom Filter with m cells and d
    /// bits allocated per cell optimized for the target false-positive rate. Use
    /// default if you don't want to calculate d.
    pub fn new(m: usize, d: u8, fp_rate: f64) -> Self {
        let mut k = optimal_k(fp_rate) / 2;
        if k > m {
            k = m;
        } else if k == 0 {
            k = 1;
        }

        let cells = Buckets::new(m, d);

        StableBloomFilter {
            hash: FnvHasher::default(),
            m,
            k,
            p: optimal_stable_p(m, k, d, fp_rate),
            max: cells.max_bucket_value(),
            cells,
            index_buffer: vec![0; k],
        }
    }

    /// Creates a new Stable Bloom Filter with m 1-bit
    /// cells and which is optimized for cases where there is no prior knowledge of
    /// the input data stream while maintaining an upper bound using the provided
    /// rate of false positives.
    pub fn new_default(m: usize, fp_rate: f64) -> Self {
        Self::new(m, 1, fp_rate)
    }

    /// NewUnstableBloomFilter creates a new special case of Stable Bloom Filter
    /// which is a traditional Bloom filter with m bits and an optimal number of
    /// hash functions for the target false-positive rate. Unlike the stable
    /// variant, data is not evicted and a cell contains a maximum of 1 hash value.
    pub fn new_unstable(m: usize, fp_rate: f64) -> Self {
        let cells = Buckets::new(m, 1);
        let k = optimal_k(fp_rate);

        StableBloomFilter {
            hash: FnvHasher::default(),
            m,
            k,
            p: 0,
            max: cells.max_bucket_value(),
            cells,
            index_buffer: vec![0; k],
        }
    }

    /// Returns the number of cells in the Stable Bloom Filter.
    pub fn cells(&self) -> usize {
        self.m
    }

    /// Returns the number of hash functions.
    pub fn k(&self) -> usize {
        self.k
    }

    /// Returns the number of cells decremented on every add.
    pub fn p(&self) -> usize {
        self.p
    }

    pub fn max(&self) -> u8 {
        self.max
    }

    /// Returns the limit of the expected fraction of zeros in the
    /// Stable Bloom Filter when the number of iterations goes to infinity. When
    /// this limit is reached, the Stable Bloom Filter is considered stable.
    pub fn stable_point(&self) -> f64 {
        let sub_denom = (self.p as f64) * ((1.0 / (self.k as f64)) - (1.0 / (self.m as f64)));
        let denom = 1.0 + 1.0 / sub_denom;
        let base = 1.0 / denom;

        base.powf(f64::from(self.max))
    }

    /// Returns the upper bound on false positives when the filter
    /// has become stable.
    pub fn false_positive_rate(&self) -> f64 {
        (1.0 - self.stable_point()).powf(self.k as f64)
    }

    pub fn hash_kernel(&self, data: &[u8]) -> (u32, u32) {
        let mut hasher = self.hash.clone();
        hasher.write(data);
        let hash: u64 = hasher.finish();
        let lower = hash as u32;
        let upper = (hash >> 32) as u32;
        (lower, upper)
    }

    /// Restores the Stable Bloom Filter to its original state. It returns the
    /// filter to allow for chaining.
    pub fn reset(&mut self) -> &Self {
        self.cells.reset();
        self
    }

    /// Will decrement a random cell and (p-1) adjacent cells by 1. This
    /// is faster than generating p random numbers. Although the processes of
    /// picking the p cells are not independent, each cell has a probability of p/m
    /// for being picked at each iteration, which means the properties still hold.
    pub fn decrement(&mut self) {
        let mut rng = thread_rng();
        let r: usize = rng.gen_range(0, self.m);

        for i in 0..(self.p) {
            let idx = (r + i) % self.m;
            self.cells.decrease(idx, 1);
        }
    }
}

impl Filter for StableBloomFilter {
    /// Will test for membership of the data and returns true if it is a
    /// member, false if not. This is a probabilistic test, meaning there is a
    /// non-zero probability of false positives and false negatives.
    fn test(&self, data: &[u8]) -> bool {
        let (lower, upper) = self.hash_kernel(data);
        for i in 0..(self.k) {
            if self
                .cells
                .get((lower as usize + upper as usize * i) % self.m)
                == 0
            {
                return false;
            }
        }
        true
    }

    /// Will add the data to the Stable Bloom Filter. It returns the filter to
    /// allow for chaining.
    fn add(&mut self, data: &[u8]) -> &Self {
        // Randomly decrement p cells to make room for new elements.
        self.decrement();
        let (lower, upper) = self.hash_kernel(data);

        for i in 0..(self.k) {
            self.cells
                .set((lower as usize + upper as usize * i) % self.m, self.max);
        }

        self
    }

    /// Is equivalent to calling Test followed by Add. It returns true if
    /// the data is a member, false if not.
    fn test_and_add(&mut self, data: &[u8]) -> bool {
        let (lower, upper) = self.hash_kernel(data);
        let mut member = true;

        // If any of the K cells are 0, then it's not a member.
        for i in 0..(self.k) {
            self.index_buffer[i] = (lower as usize + upper as usize * i) % self.m;
            if self.cells.get(self.index_buffer[i]) == 0 {
                member = false;
            }
        }

        // Randomly decrement p cells to make room for new elements.
        self.decrement();
        // Set the K cells to max.
        for i in self.index_buffer.iter() {
            self.cells.set(*i, self.max);
        }

        member
    }
}

#[cfg(test)]
mod tests {
    use super::StableBloomFilter;
    use crate::optimal_k;
    use crate::Filter;
    use float_cmp::ApproxEq;
    use std::f64;

    fn round(val: f64, round_on: f64, places: usize) -> f64 {
        let pow = (10.0_f64).powf(places as f64);
        let digit = pow * val;
        let div = digit - digit.floor();
        let round = if div >= round_on {
            digit.ceil()
        } else {
            digit.floor()
        };

        round / pow
    }

    // Ensures that new_unstable creates a Stable Bloom Filter with p=0,
    // max=1 and k hash functions.
    #[test]
    fn test_new_unstable() {
        let f = StableBloomFilter::new_unstable(100, 0.1);
        let k = optimal_k(0.1);

        assert_eq!(f.k, k);
        assert_eq!(f.m, 100);
        assert_eq!(f.p(), 0);
        assert_eq!(f.max(), 1);
    }

    // Ensures that Cells returns the number of cells, m, in the Stable Bloom
    // Filter.
    #[test]
    fn test_cells() {
        let f = StableBloomFilter::new(100, 1, 0.1);

        assert_eq!(f.cells(), 100);
    }

    // Ensures that K returns the number of hash functions in the Stable Bloom
    // Filter.
    #[test]
    fn test_k() {
        let f = StableBloomFilter::new(100, 1, 0.01);
        assert_eq!(f.k(), 3);
    }

    // Ensures that Test, Add, and TestAndAdd behave correctly.
    #[test]
    fn test_test_and_add() {
        let mut f = StableBloomFilter::new_default(10_000, 0.01);
        assert!(!f.test(b"a"));

        f.add(b"a");
        assert!(f.test(b"a"));

        assert!(f.test_and_add(b"a"));

        assert!(!f.test_and_add(b"b"));
        assert!(f.test(b"a"));

        assert!(f.test(b"b"));

        assert!(!f.test(b"c"));

        for i in 0..1_000_000 {
            f.test_and_add(i.to_string().as_bytes());
        }

        // `a` should have been evicted.
        assert!(!f.test(b"a"));
    }

    // Ensures that StablePoint returns the expected fraction of zeros for large
    // iterations.
    #[test]
    fn test_stable_point() {
        let mut f = StableBloomFilter::new(1000, 1, 0.1);
        for i in 0..1_000_000 {
            f.add(i.to_string().as_bytes());
        }

        let mut zero = 0;
        for i in 0..(f.m) {
            if f.cells.get(i) == 0 {
                zero += 1;
            }
        }

        let actual = round(f64::from(zero) / (f.m as f64), 0.5, 1);
        let expected = round(f.stable_point(), 0.5, 1);

        assert!(actual.approx_eq(expected, (f64::EPSILON, 1)));
        // A classic Bloom filter is a special case of SBF where P is 0 and max is
        // 1. It doesn't have a stable point.
        let bf = StableBloomFilter::new_unstable(1000, 0.1);
        assert!(bf.stable_point().approx_eq(0.0, (f64::EPSILON, 1)));
    }

    // Ensures that FalsePositiveRate returns the upper bound on false positives
    // for stable filters.
    #[test]
    fn test_false_positive_rate() {
        let f = StableBloomFilter::new_default(1000, 0.01);
        let fps = round(f.false_positive_rate(), 0.5, 2);

        assert!(fps.approx_eq(0.01, (f64::EPSILON, 1)));

        // Classic Bloom filters have an unbounded rate of false positives. Once
        // they become full, every query returns a false positive.
        let bf = StableBloomFilter::new_unstable(1000, 0.1);
        assert!(bf.false_positive_rate().approx_eq(1.0, (f64::EPSILON, 1)));
    }

    // Ensures that Reset sets every cell to zero.
    #[test]
    fn test_reset() {
        let mut f = StableBloomFilter::new_default(1000, 0.01);

        for i in 0..1000 {
            f.add(i.to_string().as_bytes());
        }

        f.reset();

        for i in 0..(f.m) {
            assert_eq!(f.cells.get(i), 0);
        }
    }
}
