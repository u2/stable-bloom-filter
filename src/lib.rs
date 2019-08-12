// StableBloomFilter implements a Stable Bloom Filter as described by Deng and
// Rafiei in Approximately Detecting Duplicates for Streaming Data using Stable
// Bloom Filters:
//
// http://webdocs.cs.ualberta.ca/~drafiei/papers/DupDet06Sigmod.pdf
//
// A Stable Bloom Filter (SBF) continuously evicts stale information so that it
// has room for more recent elements. Like traditional Bloom filters, an SBF
// has a non-zero probability of false positives, which is controlled by
// several parameters. Unlike the classic Bloom filter, an SBF has a tight
// upper bound on the rate of false positives while introducing a non-zero rate
// of false negatives. The false-positive rate of a classic Bloom filter
// eventually reaches 1, after which all queries result in a false positive.
// The stable-point property of an SBF means the false-positive rate
// asymptotically approaches a configurable fixed constant. A classic Bloom
// filter is actually a special case of SBF where the eviction rate is zero, so
// this package provides support for them as well.
//
// Stable Bloom Filters are useful for cases where the size of the data set
// isn't known a priori, which is a requirement for traditional Bloom filters,
// and memory is bounded.  For example, an SBF can be used to deduplicate
// events from an unbounded event stream with a specified upper bound on false
// positives and minimal false negatives.
pub mod buckets;
pub mod fnv;
pub mod stable;

pub trait Filter {
    fn test(&self, _data: &[u8]) -> bool;

    fn add(&mut self, _data: &[u8]) -> &Self;

    fn test_and_add(&mut self, _data: &[u8]) -> bool;
}

/// Calculates the optimal number of hash functions to use for a Bloom
/// filter based on the desired rate of false positives.
pub(crate) fn optimal_k(fp_rate: f64) -> usize {
    (1.0 / fp_rate).log2().ceil() as usize
}

/// Returns the optimal number of cells to decrement, p, per
/// iteration for the provided parameters of an SBF.
pub(crate) fn optimal_stable_p(m: usize, k: usize, d: u8, fp_rate: f64) -> usize {
    let max = (2_u64.pow(u32::from(d)) - 1) as f64;
    let sub_denom = (1.0 - fp_rate.powf(1.0 / (k as f64))).powf(1.0 / max);
    let denom = (1.0 / sub_denom - 1.0) * (1.0 / (k as f64) - 1.0 / (m as f64));

    let mut p = (1.0 / denom) as usize;

    if p == 0 {
        p = 1;
    }

    p
}
