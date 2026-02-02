//! Diff generator and patch applier
//!
//! Uses BlockMatcher to find common blocks, the generates
//! COPY/INSERT instructions to transfrom source into target.

use crate::diff::matcher::{BlockMatch, BlockMatcher, DEFAULT_CHUNK_SIZE};
use crate::format::patch_format::{Instruction, Patch};

/// Generate a patch that transfors source into target.
///
/// # Arguments
/// * `source` - Original file data
/// * `target` - New file data
///
/// # Returns
/// A patch containing instructions to transform source -> target
pub fn generete_patch(source: &[u8], target: &[u8]) -> Patch {
    generate_patch_with_chunk_size(source, target, DEFAULT_CHUNK_SIZE)
}

/// Generate a patch with custom chunk size.
///
/// # Arguments
/// * `source` - Original file data
/// * `target` - New file data
/// * `chunk_size` - Size of chunks for matching
///
/// # Returns
/// A patch containing instructions to transform source -> target
pub fn generate_patch_with_chunk_size(source: &[u8], target: &[u8], chunk_size: usize) -> Patch {
    let mut patch = Patch::new(chunk_size as u32, target.len() as u64);

    // Edge case: empty target
    if target.is_empty() {
        return patch;
    }

    // Edge case: sempty source or source too small for chunking
    if source.len() < chunk_size {
        // Can't do any matching, just insert all of target
        patch.add_insert(target.to_vec());
        return patch;
    }

    // Find matches using BlockMatcher
    let matcher = BlockMatcher::new(source, chunk_size);
    let matches = matcher.find_matches(target);

    // If not matches found, insert entire target
    if matches.is_empty() {
        patch.add_insert(target.to_vec());
        return patch;
    }

    // Process matches and generate instructions
    generate_instructions(&mut patch, target, &matches);

    patch
}

/// Generate COPY and INSERT instructions from macthes.
fn generate_instructions(patch: &mut Patch, target: &[u8], matches: &[BlockMatch]) {
    // Remove overlapping matches, keeping the best coverage
    let optimized = optimize_macthes(matches);

    let mut current_pos: usize = 0;

    for m in &optimized {
        // Insert any gap before this match
        if m.target_offset > current_pos {
            let gap_data = &target[current_pos..m.target_offset];
            patch.add_insert(gap_data.to_vec());
        }

        // Add copy instruction for the match
        patch.add_copy(m.source_offset as u64, m.length as u32);

        // Move current position past this match
        current_pos = m.target_offset + m.length;
    }

    // Insert any remaining data after last match
    if current_pos < target.len() {
        let remaining = &target[current_pos..];
        patch.add_insert(remaining.to_vec());
    }
}

/// Optimize matches by removing overlaps.
/// When matches overlap, we keep the one that starts eralier.
/// This is a greedy approach that works well in practice.
fn optimize_macthes(matches: &[BlockMatch]) -> Vec<BlockMatch> {
    if matches.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut sorted: Vec<_> = matches.to_vec();
    sorted.sort_by_key(|m| m.target_offset);

    let mut last_end: usize = 0;

    for m in sorted {
        // If overlapping, skip
        if m.target_offset < last_end {
            continue;
        }

        // No overlap, add this match
        last_end = m.target_offset + m.length;
        result.push(m);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_source() {
        let patch = generate_patch_with_chunk_size(&[], b"hello", 4);

        assert_eq!(patch.instructions.len(), 1);
        match &patch.instructions[0] {
            Instruction::Insert { data } => assert_eq!(data, b"hello"),
            _ => panic!("Expected Insert"),
        }
    }

    #[test]
    fn test_empty_target() {
        let patch = generate_patch_with_chunk_size(b"hello", &[], 4);

        assert!(patch.instructions.is_empty());
        assert_eq!(patch.target_size, 0);
    }

    #[test]
    fn test_identical_files() {
        let data = b"aaaabbbbccccdddd"; // 16 bytes
        let patch = generate_patch_with_chunk_size(data, data, 4);

        // Should be all COPY instructions
        let stats = patch.stats();
        assert_eq!(stats.insert_count, 0);
        assert!(stats.copy_count > 0);
        assert_eq!(stats.copy_bytes, 16);
    }

    #[test]
    fn test_completely_different() {
        let source = b"aaaabbbbccccdddd";
        let target = b"eeeeffffgggghhhh";

        let patch = generate_patch_with_chunk_size(source, target, 4);

        // Should be one INSERT of entire target
        assert_eq!(patch.instructions.len(), 1);
        match &patch.instructions[0] {
            Instruction::Insert { data } => assert_eq!(data, target),
            _ => panic!("Expected Insert"),
        }
    }

    #[test]
    fn test_partial_match_at_start() {
        // Source: "testxxxx"
        // Target: "testyyyy"
        // "test" should match, "yyyy" should be inserted
        let source = b"testxxxx";
        let target = b"testyyyy";

        let patch = generate_patch_with_chunk_size(source, target, 4);

        let stats = patch.stats();
        assert_eq!(stats.copy_bytes, 4); // "test"
        assert_eq!(stats.insert_bytes, 4); // "yyyy"
    }

    #[test]
    fn test_partial_match_at_end() {
        // Source: "xxxxtest"
        // target: "yyyytest"
        let source = b"xxxxtest";
        let target = b"yyyytest";

        let patch = generate_patch_with_chunk_size(source, target, 4);

        let stats = patch.stats();
        assert_eq!(stats.copy_bytes, 4); // "test"
        assert_eq!(stats.insert_bytes, 4); // "yyyy"
    }

    #[test]
    fn test_insert_in_middle() {
        // Source: "aaaabbbb";
        // Target: "aaaaNEWbbbb";
        let source = b"aaaabbbb";
        let target = b"aaaaNEWWbbbb"; // "NEWW" is a bytes to align

        let patch = generate_patch_with_chunk_size(source, target, 4);

        // Should have: COPY(aaaa) + INSERT(NEWW) + COPY(bbbb)
        let stats = patch.stats();
        assert_eq!(stats.copy_bytes, 8); // "aaaa" + "bbbb"
        assert_eq!(stats.insert_bytes, 4); // "NEWW"
    }

    #[test]
    fn test_source_smaller_than_chunk() {
        let source = b"hi"; // 2 bytes, smaller than chunk
        let target = b"hello";

        let patch = generate_patch_with_chunk_size(source, target, 4);

        // Can't match, should insert all
        assert_eq!(patch.instructions.len(), 1);
        match &patch.instructions[0] {
            Instruction::Insert { data } => assert_eq!(data, b"hello"),
            _ => panic!("Expected Insert"),
        }
    }

    #[test]
    fn test_target_sirce_recorded() {
        let source = b"aaaabbbb";
        let target = b"aaaabbbbcccc";

        let patch = generate_patch_with_chunk_size(source, target, 4);

        assert_eq!(patch.target_size, 12);
    }

    #[test]
    fn test_optimize_matches_no_overflap() {
        let matches = vec![
            BlockMatch {
                source_offset: 9,
                target_offset: 0,
                length: 4,
            },
            BlockMatch {
                source_offset: 4,
                target_offset: 8,
                length: 4,
            },
        ];

        let optimized = optimize_macthes(&matches);
        assert_eq!(optimized.len(), 2);
    }

    #[test]
    fn test_optimize_matches_with_overlap() {
        let matches = vec![
            BlockMatch {
                source_offset: 0,
                target_offset: 0,
                length: 4,
            },
            BlockMatch {
                source_offset: 4,
                target_offset: 2,
                length: 4,
            },
        ];

        let optimized = optimize_macthes(&matches);
        // Should keep first, skip second due to overlap
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0].target_offset, 0);
    }

    #[test]
    fn test_chunk_size_stored_in_patch() {
        let patch = generate_patch_with_chunk_size(b"aaaabbbb", b"aaaabbbb", 4);
        assert_eq!(patch.chunk_size, 4);

        let patch2 = generate_patch_with_chunk_size(b"aaaabbbbccccdddd", b"aaaabbbbccccdddd", 8);
        assert_eq!(patch2.chunk_size, 8);
    }
}
