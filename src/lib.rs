pub mod buckets;
pub mod fnv;
pub mod stable;

pub trait Filter {
    fn test(&self, _data: &[u8]) -> bool;

    fn add(&mut self, _data: &[u8]) -> &Self;

    fn test_and_add(&mut self, _data: &[u8]) -> bool;
}

// Calculates the optimal number of hash functions to use for a Bloom
// filter based on the desired rate of false positives.
pub(crate) fn optimal_k(fp_rate: f64) -> usize {
    (1.0 / fp_rate).log2().ceil() as usize
}

// Returns the optimal number of cells to decrement, p, per
// iteration for the provided parameters of an SBF.
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
