//! Block index for efficient content matching.
//!
//! Builds a hash-to-offset index from source file blocks, enabling
//! O(1) lookups during diff generation. Uses two-level hashing:
//! - Weak hash (32-bit rolling hash) for fast candidate lookup
//! - Strong hash (64-bit FNV-1a) for collision verification

use super::rolling_hash::RollingHash;
use crate::format::patch_format::calculate_hash;
use crate::DEFAULT_CHUNK_SIZE;
use std::collections::HashMap;

/// Entry storing block metadata for verification.
#[derive(Clone, Debug)]
pub struct BlockEntry {
    /// File offset where this block starts.
    pub offset: u64,
    /// Strong hash (FNV-1a 64-bit) for collision verification.
    pub strong_hash: u64,
}

/// Memory-efficient block index that stores hash-to-offset mappings.
///
/// # Memory Usage
///
/// - Per block: ~20 bytes (u32 weak_hash key + u64 offset + u64 strong_hash)
/// - 1GB file with 4KB blocks = ~250k blocks = ~5MB index
pub struct BlockIndex {
    /// Block size used for chunking.
    block_size: usize,
    /// Hash table: weak_hash -> list of block entries.
    index: HashMap<u32, Vec<BlockEntry>>,
    /// Total bytes indexed so far.
    bytes_indexed: u64,
    /// Buffer for incomplete block from previous chunk.
    pending: Vec<u8>,
}

impl BlockIndex {
    /// Creates a new empty `BlockIndex` with default block size.
    pub fn new() -> Self {
        Self::with_block_size(DEFAULT_CHUNK_SIZE)
    }

    /// Creates a new `BlockIndex` with custom block size.
    ///
    /// # Arguments
    ///
    /// * `block_size` - Size in bytes for each block.
    pub fn with_block_size(block_size: usize) -> Self {
        Self {
            block_size,
            index: HashMap::new(),
            bytes_indexed: 0,
            pending: Vec::with_capacity(block_size),
        }
    }

    /// Adds a chunk of source data to the index.
    ///
    /// Stores both weak hash (for lookup) and strong hash (for verification).
    ///
    /// # Arguments
    ///
    /// * `chunk` - Raw bytes to index.
    pub fn add_chunk(&mut self, chunk: &[u8]) {
        let mut hasher = RollingHash::new(self.block_size);

        // Combine pending bytes with new chunk
        let mut data = std::mem::take(&mut self.pending);
        data.extend_from_slice(chunk);

        // Process complete blocks
        let mut offset = 0;
        while offset + self.block_size <= data.len() {
            let block = &data[offset..offset + self.block_size];
            let weak_hash = hasher.hash_chunk(block);
            let strong_hash = calculate_hash(block);

            // Store weak_hash -> (offset, strong_hash) mapping
            self.index
                .entry(weak_hash)
                .or_insert_with(Vec::new)
                .push(BlockEntry {
                    offset: self.bytes_indexed,
                    strong_hash,
                });

            self.bytes_indexed += self.block_size as u64;
            offset += self.block_size;
        }

        // Save remaining bytes for next chunk
        if offset < data.len() {
            self.pending = data[offset..].to_vec();
        }
    }

    /// Finalizes indexing after all source chunks have been added.
    ///
    /// Note: Partial blocks at the end are NOT indexed.
    ///
    /// # Returns
    ///
    /// Total bytes indexed.
    pub fn finalize(&mut self) -> u64 {
        self.pending.clear();
        self.bytes_indexed
    }

    /// Looks up block entries where a given weak hash appears.
    ///
    /// # Returns
    ///
    /// Slice of block entries, or empty slice if hash not found.
    pub fn lookup(&self, weak_hash: u32) -> &[BlockEntry] {
        self.index
            .get(&weak_hash)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Finds a matching block by weak hash AND strong hash verification.
    ///
    /// # Arguments
    ///
    /// * `weak_hash` - Rolling hash of target block.
    /// * `target_block` - Actual bytes of target block for strong hash verification.
    ///
    /// # Returns
    ///
    /// Source offset if a verified match is found, None otherwise.
    pub fn find_verified_match(&self, weak_hash: u32, target_block: &[u8]) -> Option<u64> {
        let entries = self.lookup(weak_hash);
        if entries.is_empty() {
            return None;
        }

        // Compute strong hash of target block
        let target_strong_hash = calculate_hash(target_block);

        // Find entry with matching strong hash
        for entry in entries {
            if entry.strong_hash == target_strong_hash {
                return Some(entry.offset);
            }
        }

        None
    }

    /// Returns the block size.
    pub fn block_size(&self) -> usize {
        self.block_size
    }

    /// Returns the number of unique hashes in the index.
    pub fn unique_hash_count(&self) -> usize {
        self.index.len()
    }
}

impl Default for BlockIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_and_lookup() {
        let data = vec![0u8; 8192]; // Two blocks of 4KB
        let mut index = BlockIndex::with_block_size(4096);
        index.add_chunk(&data);
        index.finalize();

        // Both blocks have same content, so same hashes
        let mut hasher = RollingHash::new(4096);
        let hash = hasher.hash_chunk(&data[0..4096]);

        let entries = index.lookup(hash);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].offset, 0);
        assert_eq!(entries[1].offset, 4096);
    }

    #[test]
    fn test_verified_match_success() {
        let data = vec![42u8; 4096];
        let mut index = BlockIndex::with_block_size(4096);
        index.add_chunk(&data);
        index.finalize();

        let mut hasher = RollingHash::new(4096);
        let weak_hash = hasher.hash_chunk(&data);

        // Same data should find a match
        let result = index.find_verified_match(weak_hash, &data);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_verified_match_collision_rejected() {
        let data1 = vec![42u8; 4096];
        let mut index = BlockIndex::with_block_size(4096);
        index.add_chunk(&data1);
        index.finalize();

        // Create different data with same weak hash (simulated collision)
        let mut hasher = RollingHash::new(4096);
        let weak_hash = hasher.hash_chunk(&data1);

        // Different data should NOT match even if weak hash matches
        let different_data = vec![99u8; 4096];
        let result = index.find_verified_match(weak_hash, &different_data);
        assert_eq!(result, None);
    }

    #[test]
    fn test_pending_bytes() {
        let mut index = BlockIndex::with_block_size(4096);
        index.add_chunk(&[1u8; 2048]); // Half a block
        assert_eq!(index.bytes_indexed, 0); // Nothing indexed yet

        index.add_chunk(&[2u8; 2048]); // Complete the block
        assert_eq!(index.bytes_indexed, 4096); // One block indexed
    }

    #[test]
    fn test_empty_lookup() {
        let index = BlockIndex::new();
        assert!(index.lookup(12345).is_empty());
        assert_eq!(index.find_verified_match(12345, &[0u8; 4096]), None);
    }
}
