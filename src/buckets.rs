/// Buckets is a fast, space-efficient array of buckets where each bucket can
/// store up to a configured maximum value.
pub struct Buckets {
    data: Vec<u8>,
    bucket_size: u8,
    max: u8,
    count: usize,
}

impl Buckets {
    /// Creates a new Buckets with the provided number of buckets where
    /// each bucket is the specified number of bits.
    pub fn new(count: usize, bucket_size: u8) -> Self {
        if bucket_size > 8 {
            panic!("max bucket_size is 8");
        }
        Buckets {
            count,
            bucket_size,
            data: vec![0; (count * usize::from(bucket_size) + 7) / 8],
            max: ((1u16 << u16::from(bucket_size)) - 1) as u8,
        }
    }

    /// Returns the maximum value that can be stored in a bucket.
    pub fn max_bucket_value(&self) -> u8 {
        self.max
    }

    /// Returns the number of buckets.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Decrease the value in the specified bucket by the provided delta.
    /// The value is clamped to zero and the maximum bucket value.
    /// Returns itself to allow for chaining.
    #[inline]
    pub fn decrease(&mut self, bucket: usize, delta: u8) -> &Self {
        let val = (self.get_bits(bucket * usize::from(self.bucket_size), self.bucket_size) as u8)
            .saturating_sub(delta);

        self.set_bits(
            (bucket as u32) * u32::from(self.bucket_size),
            self.bucket_size,
            val,
        );
        self
    }

    /// Increment the value in the specified bucket by the provided delta.
    /// The value is clamped to zero and the maximum bucket value.
    /// Returns itself to allow for chaining.
    #[inline]
    pub fn increment(&mut self, bucket: usize, delta: u8) -> &Self {
        let val = (self.get_bits(bucket * usize::from(self.bucket_size), self.bucket_size) as u8)
            .saturating_add(delta)
            .min(self.max);

        self.set_bits(
            (bucket as u32) * u32::from(self.bucket_size),
            self.bucket_size,
            val,
        );
        self
    }

    /// Set the bucket value. The value is clamped to zero and the maximum
    /// bucket value. Returns itself to allow for chaining.
    #[inline]
    pub fn set(&mut self, bucket: usize, value: u8) -> &Self {
        let value = value.min(self.max);

        self.set_bits(
            (bucket as u32) * u32::from(self.bucket_size),
            self.bucket_size,
            value,
        );
        self
    }

    /// Returns the value in the specified bucket.
    #[inline]
    pub fn get(&self, bucket: usize) -> u8 {
        self.get_bits(bucket * usize::from(self.bucket_size), self.bucket_size) as u8
    }

    /// Reset restores the Buckets to the original state.
    /// Returns itself to allow for chaining.
    pub fn reset(&mut self) -> &Self {
        self.data = vec![0; (self.count * usize::from(self.bucket_size) + 7) / 8];
        self
    }

    /// Returns the bits at the specified offset and length.
    #[inline]
    fn get_bits(&self, offset: usize, length: u8) -> u32 {
        let byte_index = offset / 8;
        let byte_offset = offset % 8;
        if byte_offset as u8 + length > 8 {
            let rem = 8 - byte_offset as u8;
            return self.get_bits(offset, rem)
                | (self.get_bits(offset + rem as usize, length - rem) << rem);
        }

        let bit_mask = (1 << length) - 1;
        (u32::from(self.data[byte_index as usize]) & (bit_mask << byte_offset) as u32)
            >> byte_offset
    }

    /// setBits sets bits at the specified offset and length.
    #[inline]
    fn set_bits(&mut self, offset: u32, length: u8, bits: u8) {
        let byte_index = offset / 8;
        let byte_offset = offset % 8;
        if byte_offset as u8 + length > 8 {
            let rem = 8 - byte_offset as u8;
            self.set_bits(offset, rem, bits);
            self.set_bits(offset + u32::from(rem), length - rem, bits >> rem);
            return;
        }

        let bit_mask: u32 = (1 << length) - 1;
        self.data[byte_index as usize] =
            (u32::from(self.data[byte_index as usize]) & !(bit_mask << byte_offset)) as u8;
        self.data[byte_index as usize] = (u32::from(self.data[byte_index as usize])
            | ((u32::from(bits) & bit_mask) << byte_offset))
            as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::Buckets;

    // Ensures that MaxBucketValue returns the correct maximum based on the bucket
    // size.
    #[test]
    fn test_max_bucket_value() {
        let b = Buckets::new(10, 2);
        assert_eq!(b.max_bucket_value(), 3);
    }

    // Ensures that Count returns the number of buckets.
    #[test]
    fn test_buckets_count() {
        let b = Buckets::new(10, 2);
        assert_eq!(b.count(), 10);
    }

    // Ensures that Increment increments the bucket value by the correct delta and
    // clamps to zero and the maximum, Get returns the correct bucket value, and
    // Set sets the bucket value correctly.
    #[test]
    fn test_buckets_increment_decrease_and_get_and_set() {
        // bucket_size = 2
        let mut b = Buckets::new(5, 2);

        let _b = b.increment(0, 1);
        assert_eq!(b.get(0), 1);

        let _b = b.decrease(1, 1);
        assert_eq!(b.get(1), 0);

        let _b = b.set(2, 100);
        assert_eq!(b.get(2), 3);

        let _b = b.increment(3, 2);
        assert_eq!(b.get(3), 2);
        // bucket_size = 3
        let mut b = Buckets::new(5, 3);

        let _b = b.increment(0, 1);
        assert_eq!(b.get(0), 1);

        let _b = b.decrease(1, 1);
        assert_eq!(b.get(1), 0);

        let _b = b.set(2, 100);
        assert_eq!(b.get(2), 7);

        let _b = b.increment(3, 2);
        assert_eq!(b.get(3), 2);
        // bucket_size = 8
        let mut b = Buckets::new(5, 8);

        let _b = b.increment(0, 1);
        assert_eq!(b.get(0), 1);

        let _b = b.decrease(1, 1);
        assert_eq!(b.get(1), 0);

        let _b = b.set(2, 255);
        assert_eq!(b.get(2), 255);

        let _b = b.increment(3, 2);
        assert_eq!(b.get(3), 2);
    }

    // Ensures that Reset restores the Buckets to the original state.
    #[test]
    fn test_buckets_reset() {
        let mut b = Buckets::new(5, 2);

        for i in 0..5 {
            b.increment(i, 1);
        }

        let _b = b.reset();

        for i in 0..5 {
            assert_eq!(b.get(i), 0);
        }
    }
}
