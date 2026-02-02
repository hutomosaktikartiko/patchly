//! Block matching using hash table lookup.
//!
//! Builds a signature of the source file (hash of each chunk),
//! then scans the target file to find matching blocks.

use super::rolling_hash::RollingHash;
use std::collections::HashMap;

/// Default chunk size (4kb) - balance between ganularity and performance.
pub const DEFAULT_CHUNK_SIZE: usize = 4096;

/// Represents a match found between source and target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockMatch {
    /// Offset in source (old file) where this block starts
    pub source_offset: usize,
    /// Offset in target (new file) where this block starts
    pub target_offset: usize,
    /// Length of the matching block
    pub length: usize,
}

/// Signature entry for a chunk in the source file.
#[derive(Debug, Clone)]
struct ChunkSignature {
    /// Offset where this cunk starts in source
    offset: usize,
    /// Weak hash (rolling hash) for quick comparison
    #[allow(dead_code)]
    weak_hash: u32,
}

/// Block matcher that finds common blocks between source and target.
#[derive(Debug)]
pub struct BlockMatcher {
    /// Size of each chunk
    chunk_size: usize,
    /// Hash table: weak_hash -> list of chunks with that hash
    /// (multiple chunks can have same hash due to collision)
    signatures: HashMap<u32, Vec<ChunkSignature>>,
    /// Original source data
    source_data: Vec<u8>,
}

impl BlockMatcher {
    /// Create a new BlockMatcher and build signatures from source data.
    ///
    /// # Arguments
    /// * `source` - The source (old) file data
    /// * `chunk_size` - Size of each chunk for matching
    pub fn new(source: &[u8], chunk_size: usize) -> Self {
        let mut matcher = Self {
            chunk_size,
            signatures: HashMap::new(),
            source_data: source.to_vec(),
        };

        matcher.build_signatures();
        matcher
    }

    /// Build hash signatures for all chunks in source.
    fn build_signatures(&mut self) {
        if self.source_data.len() < self.chunk_size {
            return;
        }

        let mut hasher = RollingHash::new(self.chunk_size);

        // Hash each non-overlapping chunk
        let mut offset = 0;
        while offset + self.chunk_size <= self.source_data.len() {
            let chunk = &self.source_data[offset..offset + self.chunk_size];
            let weak_hash = hasher.hash_chunk(chunk);

            let sig = ChunkSignature { offset, weak_hash };

            self.signatures
                .entry(weak_hash)
                .or_insert_with(Vec::new)
                .push(sig);

            offset += self.chunk_size;
        }
    }

    /// Find all matching blocks between source and target.
    ///
    /// # Arguments
    /// * `target` - The tarhet (new) file data
    ///
    /// # Returns
    /// Vector of BlockMatch, sorted by target_offsert
    pub fn find_matches(&self, target: &[u8]) -> Vec<BlockMatch> {
        let mut matches = Vec::new();

        if target.len() < self.chunk_size || self.signatures.is_empty() {
            return matches;
        }

        let mut hasher = RollingHash::new(self.chunk_size);

        // Initial hash of first chunk
        let mut current_hash = hasher.hash_chunk(&target[..self.chunk_size]);
        let mut pos = 0;

        loop {
            // Check if current hash matches any source chunk
            if let Some(candidates) = self.signatures.get(&current_hash) {
                // verify each cadidate with byte-by-byte comparison
                for candidate in candidates {
                    if self.verify_match(candidate.offset, target, pos) {
                        matches.push(BlockMatch {
                            source_offset: candidate.offset,
                            target_offset: pos,
                            length: self.chunk_size,
                        });
                        break;
                    }
                }
            }

            // Move to next position
            pos += 1;

            // Check is we can roll to next position
            if pos + self.chunk_size > target.len() {
                break;
            }

            // Roll the hash forward
            let old_byte = target[pos - 1];
            let new_byte = target[pos + self.chunk_size - 1];
            current_hash = hasher.roll(old_byte, new_byte);
        }

        // Sory by target offset for easier processing later
        matches.sort_by_key(|m| m.target_offset);

        matches
    }

    /// Verify that source chunk at `source_offset` matches target at `target_offset`.
    /// This handles hash collisions by coing actual byte comparison.
    fn verify_match(&self, source_offset: usize, target: &[u8], target_offset: usize) -> bool {
        let source_chunk = &self.source_data[source_offset..source_offset + self.chunk_size];
        let target_chunk = &target[target_offset..target_offset + self.chunk_size];

        source_chunk == target_chunk
    }

    /// Get the chunk size used by this matcher
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Get number of unique hashes in signature table.
    pub fn signature_count(&self) -> usize {
        self.signatures.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_source() {
        let matcher = BlockMatcher::new(&[], 4);
        let matches = matcher.find_matches(b"hello");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_empty_target() {
        let matcher = BlockMatcher::new(b"hello world", 4);
        let matches = matcher.find_matches(&[]);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_source_smaller_than_chunk() {
        let matcher = BlockMatcher::new(b"hi", 4);
        assert_eq!(matcher.signature_count(), 0);
    }

    #[test]
    fn test_identical_files() {
        let data = b"aaaabbbbccccdddd"; // 16 bytes, 4 chunks of 4
        let matcher = BlockMatcher::new(data, 4);
        let matches = matcher.find_matches(data);

        // Should find 4 matches at positions 0, 4, 8, 12
        assert_eq!(matches.len(), 4);
        assert_eq!(matches[0].source_offset, 0);
        assert_eq!(matches[0].target_offset, 0);
    }

    #[test]
    fn test_partial_match() {
        let source = b"aaaabbbbccccdddd";
        let target = b"xxxxbbbbyyyycccc"; // "bbbb" and "cccc" should match

        let matcher = BlockMatcher::new(source, 4);
        let matches = matcher.find_matches(target);

        // Should find "bbbb" at target offset 4 (source offset 4)
        // Should find "cccc" at target offset 12 (source offset 8)
        assert_eq!(matches.len(), 2);

        let bbbb_match = matches.iter().find(|m| m.target_offset == 4).unwrap();
        assert_eq!(bbbb_match.source_offset, 4);

        let cccc_match = matches.iter().find(|m| m.target_offset == 12).unwrap();
        assert_eq!(cccc_match.source_offset, 8);
    }

    #[test]
    fn test_no_match() {
        let source = b"aaaabbbbccccdddd";
        let target = b"eeeeffffgggghhhh";

        let matcher = BlockMatcher::new(source, 4);
        let matches = matcher.find_matches(target);

        assert!(matches.is_empty());
    }

    #[test]
    fn test_match_at_different_offset() {
        // Source has "test" at offset 0
        // Target has "test" at offset 8
        let source = b"testxxxxyyyyzzzz";
        let target = b"aaaabbbbtest0000";

        let matcher = BlockMatcher::new(source, 4);
        let matches = matcher.find_matches(target);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].source_offset, 0); // "test" is at offset 0 in source
        assert_eq!(matches[0].target_offset, 8); // "test" is at offset 8 in target
    }

    #[test]
    fn test_rolling_finds_unaligned_match() {
        // Source chunks at 0, 4, 8, 12
        let source = b"aaaabbbbccccdddd";
        // Target: "xxbbbbcc..." - "bbbb" starts at offset 2 (unaligned!)
        let target = b"xxbbbbccyyyyyyyy";

        let matcher = BlockMatcher::new(source, 4);
        let matches = matcher.find_matches(target);

        // Should find "bbbb" even though it's at unaligned position
        let bbbb_match = matches.iter().find(|m| m.source_offset == 4);
        assert!(bbbb_match.is_some());
        assert_eq!(bbbb_match.unwrap().target_offset, 2);
    }

    #[test]
    fn test_match_length() {
        let source = b"testdata";
        let target = b"testdata";

        let matcher = BlockMatcher::new(source, 4);
        let matches = matcher.find_matches(target);

        // All matches should have length equal to chunk_size
        for m in &matches {
            assert_eq!(m.length, 4);
        }
    }
}
