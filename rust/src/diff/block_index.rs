use super::rolling_hash::RollingHash;
use crate::DEFAULT_CHUNK_SIZE;
use std::collections::HashMap;

/// Memory-efficient block index that only stores hash -> offset mappings.
///
/// # memory Usage
/// - Per block: ~12 bytes (u32 hash + u64 offset)
/// - 1GB file with 4KB blocks = ~250k blocks = ~3MB index
pub struct BlockIndex {
    // Block size used for chunking
    block_size: usize,
    // Hash table: weak_hash -> list of offsets with that hash
    index: HashMap<u32, Vec<u64>>,
    // Total bytes indexed so far
    bytes_indexed: u64,
    // Buffer for incomplete block from previous chunk
    pending: Vec<u8>,
}

impl BlockIndex {
    /// Create a new empty BlockIndex.
    pub fn new() -> Self {
        Self::with_block_size(DEFAULT_CHUNK_SIZE)
    }

    /// Create a new BlockIndex with custom block size
    pub fn with_block_size(block_size: usize) -> Self {
        Self {
            block_size,
            index: HashMap::new(),
            bytes_indexed: 0,
            pending: Vec::with_capacity(block_size),
        }
    }

    /// Add a chunk of source data to the index.
    /// Only stores hash -> offset mapping, discards raw data.
    ///
    /// # Arguments
    /// * `chunk` - Raw bytes to index
    pub fn add_chunk(&mut self, chunk: &[u8]) {
        let mut hasher = RollingHash::new(self.block_size);

        // Combine pending bytes with new chunk
        let mut data = std::mem::take(&mut self.pending);
        data.extend_from_slice(chunk);

        // Process complete blocks
        let mut offset = 0;
        while offset + self.block_size <= data.len() {
            let block = &data[offset..offset + self.block_size];
            let hash = hasher.hash_chunk(block);

            // Store hash -> offset mapping
            self.index
                .entry(hash)
                .or_insert_with(Vec::new)
                .push(self.bytes_indexed);

            self.bytes_indexed += self.block_size as u64;
            offset += self.block_size;
        }

        // Save remaining bytes for next chunk
        if offset < data.len() {
            self.pending = data[offset..].to_vec();
        }
    }

    /// Finalize indexing. Call after all source chunks added.
    /// Note: Partial blocks at the end are NOT indexed.
    /// Returns total bytes indexed
    pub fn finalize(&mut self) -> u64 {
        // Clear pending buffer
        self.pending.clear();
        self.bytes_indexed
    }

    /// Look up offsets where a given hash appears.
    /// Returns empty slice if hash not found.
    pub fn lookup(&self, hash: u32) -> &[u64] {
        self.index.get(&hash).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get the block size
    pub fn block_size(&self) -> usize {
        self.block_size
    }

    /// Get number of unique hashes in index.
    pub fn unique_hash_count(&self) -> usize {
        self.index.len()
    }

    /// Get total number of blocks indexed.
    pub fn block_count(&self) -> usize {
        self.index.values().map(|v| v.len()).sum()
    }

    /// Get total bytes indexed.
    pub fn bytes_indexed(&self) -> u64 {
        self.bytes_indexed
    }

    /// Clear the index and reset state.
    pub fn clear(&mut self) {
        self.index.clear();
        self.bytes_indexed = 0;
        self.pending.clear();
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
    fn test_empty_index() {
        let index = BlockIndex::new();
        assert_eq!(index.block_count(), 0);
        assert_eq!(index.bytes_indexed(), 0);
        assert!(index.lookup(12345).is_empty());
    }

    #[test]
    fn test_partial_block_not_indexed() {
        let mut index = BlockIndex::with_block_size(4);
        index.add_chunk(b"aaaabb"); // 6 bytes = 1 complete + partial
        index.finalize();

        assert_eq!(index.block_count(), 1);
        assert_eq!(index.bytes_indexed(), 4);
    }

    #[test]
    fn test_chunked_input() {
        // Simulate streaming: data arrives in chunks
        let mut index = BlockIndex::with_block_size(4);
        index.add_chunk(b"aa"); // 2 bytes - pending
        index.add_chunk(b"aabb"); // 4 more = "aaaa" complete + "bb" pending
        index.add_chunk(b"bb"); // 2 more = "bbbb" complete
        index.finalize();

        assert_eq!(index.block_count(), 2);
        assert_eq!(index.bytes_indexed(), 8);
    }

    #[test]
    fn test_lookup_existing_hash() {
        let mut index = BlockIndex::with_block_size(4);
        index.add_chunk(b"aaaa");
        index.finalize();

        // Get the hash for "aaaa"
        let mut hasher = RollingHash::new(4);
        let hash = hasher.hash_chunk(b"aaaa");

        let offsets = index.lookup(hash);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0], 0);
    }

    #[test]
    fn test_lookup_nonexistent_hash() {
        let mut index = BlockIndex::with_block_size(4);
        index.add_chunk(b"aaaa");
        index.finalize();

        // Random hash that shouldn't exist
        let offsets = index.lookup(99999);
        assert!(offsets.is_empty());
    }

    #[test]
    fn test_duplicate_blocks() {
        let mut index = BlockIndex::with_block_size(4);
        // "aaaa" appears twice at offset 0 amd 8
        index.add_chunk(b"aaaabbbbaaaa");
        index.finalize();

        let mut hasher = RollingHash::new(4);
        let hash = hasher.hash_chunk(b"aaaa");

        let offsets = index.lookup(hash);
        assert_eq!(offsets.len(), 2);
        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[1], 8);
    }

    #[test]
    fn test_clear() {
        let mut index = BlockIndex::with_block_size(4);
        index.add_chunk(b"aaaabbbb");
        index.finalize();

        assert_eq!(index.block_count(), 2);

        index.clear();

        assert_eq!(index.block_count(), 0);
        assert_eq!(index.bytes_indexed(), 0);
    }

    #[test]
    fn test_offsets_are_correct() {
        let mut index = BlockIndex::with_block_size(4);
        index.add_chunk(b"aaaabbbbccccdddd"); // 4 blocks at 0, 4, 8, 12
        index.finalize();

        let mut hasher = RollingHash::new(4);

        // Check each block's offset
        let hash_a = hasher.hash_chunk(b"aaaa");
        let hash_b = hasher.hash_chunk(b"bbbb");
        let hash_c = hasher.hash_chunk(b"cccc");
        let hash_d = hasher.hash_chunk(b"dddd");

        assert_eq!(index.lookup(hash_a), &[0]);
        assert_eq!(index.lookup(hash_b), &[4]);
        assert_eq!(index.lookup(hash_c), &[8]);
        assert_eq!(index.lookup(hash_d), &[12]);
    }
}
