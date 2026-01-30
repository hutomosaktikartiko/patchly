//! Rolling hash implementation for chunk fingerprinting.
//!
//! Uses Adler-32 style algorithm with two sums:
//! - `sum_a`: Sum of all bytes in window
//! - `sum_b`: Weighted sum (positional)
//!
//! This allows O(1) "rolling" when sliding the window by 1 byte.

/// Modulus for hash calculation to prevent overflow.
/// Using prime close to 2^16 for better distribution.
const MODULUS: u32 = 65521;

#[derive(Debug, Clone)]
pub struct RollingHash {
    sum_a: u32,
    sum_b: u32,
    window_size: usize,
}

impl RollingHash {
    /// Create a new RollingHash with specified window size.
    ///
    /// # Arguments
    /// * `window_size` - Number of bytes in the sliding window (chunk size)
    pub fn new(window_size: usize) -> Self {
        Self {
            sum_a: 0,
            sum_b: 0,
            window_size,
        }
    }

    /// Calculate hash from a full chunk of data.
    /// Use this for initial hash or when you have complete chunk.
    ///
    /// # Arguments
    /// * `data` - Byte slice, should be exactly `window_size` bytes
    ///
    /// # Returns
    /// * `u32` - Computed hash value
    pub fn hash_chunk(&mut self, data: &[u8]) -> u32 {
        self.sum_a = 0;
        self.sum_b = 0;

        for (i, &byte) in data.iter().enumerate() {
            self.sum_a = (self.sum_a + byte as u32) % MODULUS;
            // Weight: window_size - i (first byte has highest weight)
            self.sum_b = (self.sum_b + (self.window_size - i) as u32 * byte as u32) % MODULUS;
        }

        self.digest()
    }

    /// Roll the hash forward by removing `old_byte` and adding `new_byte`.
    /// This is O(1) operation - the core efficiency of rolling hash.
    ///
    /// # Arguments
    /// * `old_byte`: Byte leading the window (leftmost)
    /// * `new_byte`: Byte entering the window (rightmost)
    ///
    /// # Returns
    /// * `u32` - Updated hash value
    pub fn roll(&mut self, old_byte: u8, new_byte: u8) -> u32 {
        let old = old_byte as u32;
        let new = new_byte as u32;

        // Remove old_byte contribution, add new_byte
        // sum_a = sum_a - old + new
        self.sum_a = (self.sum_a + MODULUS - (old % MODULUS) + new) % MODULUS;

        // sum_b loses: window_size * old (old had highest weight)
        // sum_b gains: sum_a (new byte position 1, but we also shifted all weights)
        // Simplifier: sum_b = sum_b - window_size * old + sum_a
        let ws = self.window_size as u32;
        self.sum_b = (self.sum_b + MODULUS - ((ws * old) % MODULUS) + self.sum_a) % MODULUS;

        self.digest()
    }

    /// Combine sum_a and sum_b into final hash value.
    #[inline]
    pub fn digest(&self) -> u32 {
        (self.sum_b << 16) | self.sum_a
    }

    /// Get current sum_a
    pub fn sum_a(&self) -> u32 {
        self.sum_a
    }

    /// Get current sum_b
    pub fn sum_b(&self) -> u32 {
        self.sum_b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_chunk_basic() {
        let mut rh = RollingHash::new(4);
        let hash = rh.hash_chunk(b"abcd");

        // Hash should be non-sero
        assert_ne!(hash, 0);

        // Same input should produced same hash
        let mut rh2 = RollingHash::new(4);
        let hash2 = rh2.hash_chunk(b"abcd");
        assert_eq!(hash, hash2);
    }

    #[test]
    pub fn test_different_data_different_hash() {
        let mut rh = RollingHash::new(4);
        let hash1 = rh.hash_chunk(b"abcd");
        let has2 = rh.hash_chunk(b"abce");

        assert_ne!(hash1, has2);
    }

    #[test]
    fn test_roll_equivalence() {
        // Rolling from "abcd" to "bcde" should equal direct hash of "bcde"
        let mut rh1 = RollingHash::new(4);
        rh1.hash_chunk(b"abcd");
        let rolled = rh1.roll(b'a', b'e');

        let mut rh2 = RollingHash::new(4);
        let direct = rh2.hash_chunk(b"bcde");

        assert_eq!(rolled, direct);
    }

    #[test]
    fn test_roll_multiple_times() {
        // Test rolling through "abcdef": abcd -> bcde -> cdef
        let mut rh = RollingHash::new(4);
        rh.hash_chunk(b"abcd");

        let hash_bcde = rh.roll(b'a', b'e');
        let hash_cdef = rh.roll(b'b', b'f');

        // Verify against direct computation
        let mut rh2 = RollingHash::new(4);
        assert_eq!(hash_bcde, rh2.hash_chunk(b"bcde"));
        assert_eq!(hash_cdef, rh2.hash_chunk(b"cdef"));
    }

    #[test]
    fn test_window_size_1() {
        let mut rh = RollingHash::new(1);
        let hash_a = rh.hash_chunk(b"a");
        let hash_b = rh.roll(b'a', b'b');

        let mut rh2 = RollingHash::new(1);
        assert_eq!(hash_a, rh2.hash_chunk(b"a"));
        assert_eq!(hash_b, rh2.hash_chunk(b"b"));
    }

    #[test]
    fn test_larger_window() {
        let data = b"hello world!";
        let mut rh = RollingHash::new(12);
        let hash = rh.hash_chunk(data);

        assert_ne!(hash, 0);
    }

    #[test]
    fn test_binary_data() {
        // Test with non-ASCII binary data
        let data: [u8; 4] = [0x00, 0xFF, 0x80, 0x7F];
        let mut rh = RollingHash::new(4);
        let hash = rh.hash_chunk(&data);

        assert_ne!(hash, 0);
    }
}
