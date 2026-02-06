//! Rolling hash implementation for content-defined chunking.
//!
//! Uses an Adler-32 style algorithm with two sums:
//! - `sum_a`: Sum of all bytes in the window
//! - `sum_b`: Weighted sum (positional)
//!
//! This allows O(1) "rolling" when sliding the window by one byte.

/// Modulus for hash calculation to prevent overflow.
/// Using a prime close to 2^16 for better distribution.
const MODULUS: u32 = 65521;

/// Rolling hash calculator for efficient window-based hashing.
///
/// Supports O(1) hash updates when sliding a window through data,
/// making it ideal for delta encoding and deduplication.
#[derive(Debug, Clone)]
pub struct RollingHash {
    /// Sum of all bytes in the current window.
    sum_a: u32,
    /// Weighted positional sum.
    sum_b: u32,
    /// Size of the sliding window in bytes.
    window_size: usize,
}

impl RollingHash {
    /// Creates a new `RollingHash` with the specified window size.
    ///
    /// # Arguments
    ///
    /// * `window_size` - Number of bytes in the sliding window (chunk size).
    pub fn new(window_size: usize) -> Self {
        Self {
            sum_a: 0,
            sum_b: 0,
            window_size,
        }
    }

    /// Calculates hash from a complete chunk of data.
    ///
    /// Use this for initial hash calculation or when you have a complete chunk.
    ///
    /// # Arguments
    ///
    /// * `data` - Byte slice, should be exactly `window_size` bytes.
    ///
    /// # Returns
    ///
    /// The computed 32-bit hash value.
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

    /// Rolls the hash forward by one byte.
    ///
    /// This is an O(1) operation - the core efficiency of rolling hash.
    ///
    /// # Arguments
    ///
    /// * `old_byte` - Byte leaving the window (leftmost).
    /// * `new_byte` - Byte entering the window (rightmost).
    ///
    /// # Returns
    ///
    /// The updated 32-bit hash value.
    pub fn roll(&mut self, old_byte: u8, new_byte: u8) -> u32 {
        let old = old_byte as u32;
        let new = new_byte as u32;

        // Update sum_a: remove old, add new
        self.sum_a = (self.sum_a + MODULUS - (old % MODULUS) + new) % MODULUS;

        // Update sum_b: old had highest weight, new gets sum_a contribution
        let ws = self.window_size as u32;
        self.sum_b = (self.sum_b + MODULUS - ((ws * old) % MODULUS) + self.sum_a) % MODULUS;

        self.digest()
    }

    /// Combines sum_a and sum_b into the final hash value.
    #[inline]
    pub fn digest(&self) -> u32 {
        (self.sum_b << 16) | self.sum_a
    }

    /// Returns the current sum_a value.
    pub fn sum_a(&self) -> u32 {
        self.sum_a
    }

    /// Returns the current sum_b value.
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

        // Hash should be non-zero
        assert_ne!(hash, 0);

        // Same input should produce same hash
        let mut rh2 = RollingHash::new(4);
        let hash2 = rh2.hash_chunk(b"abcd");
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_different_data_different_hash() {
        let mut rh = RollingHash::new(4);
        let hash1 = rh.hash_chunk(b"abcd");
        let hash2 = rh.hash_chunk(b"abce");

        assert_ne!(hash1, hash2);
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
