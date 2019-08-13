# Stable Bloom-Filter

A Rust-implementation of a stable Bloom filter for filtering duplicates out of data streams, port of [BoomFilters](https://github.com/tylertreat/BoomFilters)

[![Travis CI]](https://travis-ci.com/u2/stable-bloom-filter) [![Stable Badge]](https://crates.io/crates/stable-bloom-filter)

[Travis CI]: https://img.shields.io/travis/com/u2/stable-bloom-filter.svg
[Stable Badge]: https://img.shields.io/crates/v/stable-bloom-filter.svg

This is an implementation of Stable Bloom Filters as described by Deng and Rafiei in Approximately Detecting Duplicates for Streaming Data using Stable Bloom Filters.

A Stable Bloom Filter (SBF) continuously evicts stale information so that it has room for more recent elements. Like traditional Bloom filters, an SBF has a non-zero probability of false positives, which is controlled by several parameters. Unlike the classic Bloom filter, an SBF has a tight upper bound on the rate of false positives while introducing a non-zero rate of false negatives. The false-positive rate of a classic Bloom filter eventually reaches 1, after which all queries result in a false positive. The stable-point property of an SBF means the false-positive rate asymptotically approaches a configurable fixed constant. A classic Bloom filter is actually a special case of SBF where the eviction rate is zero and the cell size is one, so this provides support for them as well (in addition to bitset-based Bloom filters).

Stable Bloom Filters are useful for cases where the size of the data set isn't known a priori and memory is bounded. For example, an SBF can be used to deduplicate events from an unbounded event stream with a specified upper bound on false positives and minimal false negatives.

## Usage

```toml
stable-bloom-filter = "0.3"
```

```rust
use stable-bloom-filter::StableBloomFilter;

let mut f = StableBloomFilter::new_default(10_000, 0.01);
assert!(!f.test(b"a"));

f.add(b"a");
assert!(f.test(b"a"));

assert!(f.test_and_add(b"a"));

assert!(!f.test_and_add(b"b"));
assert!(f.test(b"a"));

assert!(f.test(b"b"));

assert!(!f.test(b"c"));
```
