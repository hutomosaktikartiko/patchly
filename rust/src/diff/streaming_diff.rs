use super::block_index::BlockIndex;
use super::rolling_hash::RollingHash;
use crate::format::patch_format::Instruction;

/// Streaming diff generator that uses BlockIndex
pub struct StreamingDiff {
    // Block index built from source file
    index: BlockIndex,
    // Block size for matching
    block_size: usize,
    // Rolling hasher
    hasher: RollingHash,
    // Buffer for unmatched bytes (pending INSERT)
    pending_insert: Vec<u8>,
    // Generated instructions
    instructions: Vec<Instruction>,
    // Current position in target
    target_position: usize,
    // Source file size (for COPY validation)
    source_size: u64,
}

impl StreamingDiff {
    /// Create a new StreamingDiff from a BlockIndex.
    pub fn new(index: BlockIndex, source_size: u64) -> Self {
        let block_size = index.block_size();

        Self {
            index,
            block_size,
            hasher: RollingHash::new(block_size),
            pending_insert: Vec::new(),
            instructions: Vec::new(),
            target_position: 0,
            source_size,
        }
    }

    /// Generate a chunk of target data.
    /// Generated instructions for matches found
    pub fn process_target_chunk(&mut self, chunk: &[u8]) {
        // Add to pending buffer for processing
        self.pending_insert.extend_from_slice(chunk);

        // Process while we have enough data for a block
        while self.pending_insert.len() >= self.block_size {
            self.try_match_block();
        }
    }

    /// Try to match current block agains index.
    fn try_match_block(&mut self) {
        if self.pending_insert.len() < self.block_size {
            return;
        }

        let block = &self.pending_insert[..self.block_size];
        let hash = self.hasher.hash_chunk(block);

        // Look up hash in index
        let offsets = self.index.lookup(hash);

        if !offsets.is_empty() {
            // Use first offset
            let source_offset = offsets[0];

            // Emit COPY instruction
            self.instructions.push(Instruction::Copy {
                offset: source_offset,
                length: self.block_size as u32,
            });

            // Remove matched block from pending
            self.pending_insert.drain(..self.block_size);
            self.target_position += self.block_size;
        } else {
            // Move first byte to INSERT buffer
            let byte = self.pending_insert.remove(0);

            // Extend last INSERT instruction
            if let Some(Instruction::Insert { data }) = self.instructions.last_mut() {
                data.push(byte);
            } else {
                self.instructions
                    .push(Instruction::Insert { data: vec![byte] });
            }
            self.target_position += 1;
        }
    }

    /// finalize and get all instructions
    pub fn finalize(mut self) -> Vec<Instruction> {
        // Flush any remaining pending bytes as INSERT
        if !self.pending_insert.is_empty() {
            if let Some(Instruction::Insert { data }) = self.instructions.last_mut() {
                data.extend_from_slice(&self.pending_insert);
            } else {
                self.instructions.push(Instruction::Insert {
                    data: std::mem::take(&mut self.pending_insert),
                });
            }
        }

        // Merge consecutive INSERTs
        self.merge_inserts();

        self.instructions
    }

    /// Merge consecutive INSERT instructions.
    fn merge_inserts(&mut self) {
        let mut merged = Vec::with_capacity(self.instructions.len());

        for instruction in self.instructions.drain(..) {
            match (&mut merged.last_mut(), instruction) {
                (
                    Some(Instruction::Insert { data: existing }),
                    Instruction::Insert { data: new },
                ) => {
                    existing.extend(new);
                }
                (_, other) => {
                    merged.push(other);
                }
            }
        }

        self.instructions = merged;
    }

    /// Get current instruction count
    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }

    /// Get bytes processed so far
    pub fn bytes_processed(&self) -> usize {
        self.target_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_index(data: &[u8], block_size: usize) -> BlockIndex {
        let mut index = BlockIndex::with_block_size(block_size);
        index.add_chunk(data);
        index.finalize();
        index
    }

    #[test]
    fn test_identical_data() {
        let data = b"aaaabbbbccccdddd";
        let index = build_index(data, 4);

        let mut diff = StreamingDiff::new(index, data.len() as u64);
        diff.process_target_chunk(data);
        let instructions = diff.finalize();

        // Should be all COPY instructions
        assert_eq!(instructions.len(), 4);
        for instr in &instructions {
            assert!(matches!(instr, Instruction::Copy { .. }));
        }
    }

    #[test]
    fn test_completelty_different() {
        let source = b"aaaabbbbccccdddd";
        let target = b"eeeeffffgggghhhh";
        let index = build_index(source, 4);

        let mut diff = StreamingDiff::new(index, source.len() as u64);
        diff.process_target_chunk(target);
        let instructions = diff.finalize();

        // Should be all INSERT (not matches)
        assert_eq!(instructions.len(), 1);
        if let Instruction::Insert { data } = &instructions[0] {
            assert_eq!(data, target);
        } else {
            panic!("Expected INSERT instruction");
        }
    }

    #[test]
    fn test_partial_match() {
        let source = b"aaaabbbbccccdddd";
        let target = b"xxxxbbbbyyyycccc";
        let index = build_index(source, 4);

        let mut diff = StreamingDiff::new(index, source.len() as u64);
        diff.process_target_chunk(target);
        let instructions = diff.finalize();

        // Should have: INSERT(xxxx), COPY(bbbb), INSERT(yyyy), COPY(cccc)
        assert!(instructions.len() >= 3);

        let has_copy = instructions
            .iter()
            .any(|i| matches!(i, Instruction::Copy { .. }));
        let has_insert = instructions
            .iter()
            .any(|i| matches!(i, Instruction::Insert { .. }));
        assert!(has_copy);
        assert!(has_insert);
    }

    #[test]
    fn test_chunked_processing() {
        let source = b"aaaabbbbccccdddd";
        let index = build_index(source, 4);

        let mut diff = StreamingDiff::new(index, source.len() as u64);
        diff.process_target_chunk(b"aaaa");
        diff.process_target_chunk(b"bbbb");
        diff.process_target_chunk(b"cccc");
        diff.process_target_chunk(b"dddd");

        let instruction = diff.finalize();
        assert_eq!(instruction.len(), 4);
    }

    #[test]
    fn test_unaligned_chunks() {
        let source = b"aaaabbbbccccdddd";
        let index = build_index(source, 4);

        let mut diff = StreamingDiff::new(index, source.len() as u64);
        diff.process_target_chunk(b"aa");
        diff.process_target_chunk(b"aabb");
        diff.process_target_chunk(b"bbcc");
        diff.process_target_chunk(b"ccdd");
        diff.process_target_chunk(b"dd");

        let instructions = diff.finalize();
        assert_eq!(instructions.len(), 4);
    }

    #[test]
    fn test_empty_target() {
        let source = b"aaaabbbb";
        let index = build_index(source, 4);

        let diff = StreamingDiff::new(index, source.len() as u64);
        let instructions = diff.finalize();
        assert!(instructions.is_empty());
    }

    #[test]
    fn test_target_smaller_that_block() {
        let source = b"aaaabbbb";
        let target = b"xx";
        let index = build_index(source, 4);

        let mut diff = StreamingDiff::new(index, source.len() as u64);
        diff.process_target_chunk(target);
        let instructions = diff.finalize();

        // Should be single INSERT
        assert_eq!(instructions.len(), 1);
        if let Instruction::Insert { data } = &instructions[0] {
            assert_eq!(data, b"xx");
        }
    }

    #[test]
    fn test_instruction_count() {
        let source = b"aaaabbbb";
        let index = build_index(source, 4);

        let mut diff = StreamingDiff::new(index, source.len() as u64);
        assert_eq!(diff.instruction_count(), 0);

        diff.process_target_chunk(b"aaaa");
        assert_eq!(diff.instruction_count(), 1);

        diff.process_target_chunk(b"bbbb");
        assert_eq!(diff.instruction_count(), 2);
    }
}
