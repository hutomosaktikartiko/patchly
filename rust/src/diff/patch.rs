//! Diff generator and patch applier
//!
//! Uses BlockMatcher to find common blocks, the generates
//! COPY/INSERT instructions to transfrom source into target.

use crate::diff::matcher::{BlockMatch, BlockMatcher, DEFAULT_CHUNK_SIZE};
use crate::format::patch_format::{calculate_hash, Instruction, Patch, ValidationError};

/// Generate a patch that transfors source into target.
///
/// # Arguments
/// * `source` - Original file data
/// * `target` - New file data
///
/// # Returns
/// A patch containing instructions to transform source -> target
pub fn generate_patch(source: &[u8], target: &[u8]) -> Patch {
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
    let source_hash = calculate_hash(source);
    let mut patch = Patch::new(
        chunk_size as u32,
        source.len() as u64,
        source_hash,
        target.len() as u64,
    );

    // Edge case: empty target
    if target.is_empty() {
        return patch;
    }

    // Edge case: empty source or source too small for chunking
    if source.len() < chunk_size {
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

/// Apply a patch to source data to produce target data.
///
/// # Arguments
/// * `source` - Original file data
/// * `patch` - Patch containing transformation instructions
///
/// # Returns
/// Result containing the reconstructed target data, or an error
pub fn apply_patch(source: &[u8], patch: &Patch) -> Result<Vec<u8>, PatchError> {
    // Validate source
    let source_hash = calculate_hash(source);
    patch
        .validate_source(source.len() as u64, source_hash)
        .map_err(PatchError::ValidationFailed)?;

    let mut output = Vec::with_capacity(patch.target_size as usize);

    for (idx, instruction) in patch.instructions.iter().enumerate() {
        match instruction {
            Instruction::Copy { offset, length } => {
                let start = *offset as usize;
                let end = start + *length as usize;

                // Validate bounds
                if end > source.len() {
                    return Err(PatchError::CopyOutOfBounds {
                        instruction_index: idx,
                        offset: *offset,
                        length: *length,
                        source_len: source.len(),
                    });
                }

                output.extend_from_slice(&source[start..end]);
            }
            Instruction::Insert { data } => {
                output.extend_from_slice(data);
            }
        }
    }

    // Validate final size
    if output.len() != patch.target_size as usize {
        return Err(PatchError::SizeMismatch {
            expected: patch.target_size as usize,
            actual: output.len(),
        });
    }

    Ok(output)
}

/// Errors that can occur when applying a patch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchError {
    /// Source file validation failed (wrong file)
    ValidationFailed(ValidationError),
    /// COPY instruction references data outside source bounds
    CopyOutOfBounds {
        instruction_index: usize,
        offset: u64,
        length: u32,
        source_len: usize,
    },
    /// Final output size doesn't match expected target size
    SizeMismatch { expected: usize, actual: usize },
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchError::ValidationFailed(e) => write!(f, "Source validation failed: {}", e),
            PatchError::CopyOutOfBounds {
                instruction_index,
                offset,
                length,
                source_len,
            } => {
                write!(
                    f,
                    "COPY instruction {} out of bounds: offset={}, length={}, source_len={}",
                    instruction_index, offset, length, source_len
                )
            }
            PatchError::SizeMismatch { expected, actual } => write!(
                f,
                "Output size mismatch: expected {}, got {}",
                expected, actual,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_source() {
        let patch = generate_patch_with_chunk_size(&[], b"hello", 4);

        assert_eq!(patch.instructions.len(), 1);
        assert_eq!(patch.source_size, 0);
        match &patch.instructions[0] {
            Instruction::Insert { data } => assert_eq!(data, b"hello"),
            _ => panic!("Expected Insert"),
        }
    }

    #[test]
    fn test_empty_target() {
        let patch = generate_patch_with_chunk_size(b"hello world", &[], 4);

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
    fn test_target_size_recorded() {
        let source = b"aaaabbbb";
        let target = b"aaaabbbbcccc";

        let patch = generate_patch_with_chunk_size(source, target, 4);

        assert_eq!(patch.target_size, 12);
    }

    #[test]
    fn test_source_hash_recorded() {
        let source = b"aaaabbbb";
        let target = b"aaaabbbbcccc";

        let patch = generate_patch_with_chunk_size(source, target, 4);

        assert_eq!(patch.source_size, 8);
        assert_eq!(patch.source_hash, calculate_hash(source));
    }

    #[test]
    fn test_optimize_matches_no_overflap() {
        let matches = vec![
            BlockMatch {
                source_offset: 0,
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
    }

    #[test]
    fn test_apply_empty_patch() {
        let source = b"hello";
        let patch = Patch::new(4, source.len() as u64, calculate_hash(source), 0);

        let result = apply_patch(source, &patch).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_insert_only() {
        let source = b"";
        let mut patch = Patch::new(4, 0, calculate_hash(source), 5);
        patch.add_insert(b"hello".to_vec());

        let result = apply_patch(source, &patch).unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn test_apply_copy_only() {
        let source = b"hello world";
        let mut patch = Patch::new(4, source.len() as u64, calculate_hash(source), 5);
        patch.add_copy(0, 5); // Copy "hello"

        let result = apply_patch(source, &patch).unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn test_apply_mixed_instructions() {
        let source = b"AAAA....BBBB";
        let mut patch = Patch::new(4, source.len() as u64, calculate_hash(source), 12);
        patch.add_copy(0, 4); // "AAAA"
        patch.add_insert(b"NEW!".to_vec()); // "NEW!"
        patch.add_copy(8, 4); // "BBBB"

        let result = apply_patch(source, &patch).unwrap();
        assert_eq!(result, b"AAAANEW!BBBB");
    }

    #[test]
    fn test_apply_copy_out_of_bounds() {
        let source = b"short";
        let mut patch = Patch::new(4, source.len() as u64, calculate_hash(source), 10);
        patch.add_copy(0, 100); // Way to long!

        let result = apply_patch(source, &patch);
        assert!(result.is_err());
        match result.unwrap_err() {
            PatchError::CopyOutOfBounds { .. } => {}
            _ => panic!("Expected CopyOutBounds error"),
        }
    }

    #[test]
    fn test_apply_size_mismatch() {
        let source = b"hello";
        let mut patch = Patch::new(4, source.len() as u64, calculate_hash(source), 100); // Claims target is 100 bytes
        patch.add_copy(0, 5);

        let result = apply_patch(source, &patch);
        assert!(result.is_err());
        match result.unwrap_err() {
            PatchError::SizeMismatch {
                expected: 100,
                actual: 5,
            } => {}
            _ => panic!("Expected SizeMismatch error"),
        }
    }

    #[test]
    fn test_apply_wrong_source_file() {
        let correct_source = b"correct file";
        let wrong_source = b"wrong file!!";

        let mut patch = Patch::new(
            4,
            correct_source.len() as u64,
            calculate_hash(correct_source),
            5,
        );
        patch.add_insert(b"hello".to_vec());

        // Try to apply with wrong source
        let result = apply_patch(wrong_source, &patch);
        assert!(result.is_err());
        match result.unwrap_err() {
            PatchError::ValidationFailed(_) => {}
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_roundtrip_generate_and_apply() {
        let source = b"aaaabbbbccccdddd";
        let target = b"aaaaNEWWccccdddd"; // Changed "bbbb" to "NEWW"

        // Generate patch
        let patch = generate_patch_with_chunk_size(source, target, 4);

        // Apply patch
        let reconstructed = apply_patch(source, &patch).unwrap();

        // Should get back the target
        assert_eq!(reconstructed, target);
    }

    #[test]
    fn test_roundtrip_with_serialize() {
        let source = b"originaldatahere";
        let target = b"originalNEWWhere!";

        // Generate patch
        let patch = generate_patch_with_chunk_size(source, target, 4);

        // Serialize and deserialize
        let bytes = patch.serialize().unwrap();
        let restored_patch = Patch::deserialize(&bytes).unwrap();

        // Apply restored patch
        let reconstructed = apply_patch(source, &restored_patch).unwrap();

        assert_eq!(reconstructed, target);
    }

    #[test]
    fn test_roundtrip_indentical_files() {
        let data = b"testdatatestdata";

        let patch = generate_patch_with_chunk_size(data, data, 4);
        let reconstructred = apply_patch(data, &patch).unwrap();

        assert_eq!(reconstructred, data);
    }

    #[test]
    fn test_roundtrip_completely_new() {
        let source = b"olddata!olddata!";
        let target = b"newstuffnewstuff";

        let patch = generate_patch_with_chunk_size(source, target, 4);
        let recontsructured = apply_patch(source, &patch).unwrap();

        assert_eq!(recontsructured, target);
    }
}
