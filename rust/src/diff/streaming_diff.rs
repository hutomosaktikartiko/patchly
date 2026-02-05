use super::block_index::BlockIndex;
use super::rolling_hash::RollingHash;
use crate::format::patch_format::Instruction;

/// Streaming diff generator that uses BlockIndex
pub struct StreamingDiff {
    // Block index built from source file
    index: BlockIndex,
    // Block size for matching
    block_size: usize,
    // Buffer for pending data
    buffer: Vec<u8>,
    // Generated instructions
    instructions: Vec<Instruction>,
    // Pending INSERT data
    insert_buffer: Vec<u8>,
}

impl StreamingDiff {
    /// Create a new StreamingDiff from a BlockIndex.
    pub fn new(index: BlockIndex) -> Self {
        let block_size = index.block_size();

        Self {
            index,
            block_size,
            buffer: Vec::new(),
            instructions: Vec::new(),
            insert_buffer: Vec::new(),
        }
    }

    /// Generate a chunk of target data.
    pub fn process_target_chunk(&mut self, chunk: &[u8]) {
        self.buffer.extend_from_slice(chunk);
        self.process_buffer();
    }

    /// Process buffered data using rolling hash
    fn process_buffer(&mut self) {
        if self.buffer.len() < self.block_size {
            return;
        }

        let mut hasher = RollingHash::new(self.block_size);
        let mut pos = 0;

        // Calculate initial hash
        let mut current_hash = hasher.hash_chunk(&self.buffer[0..self.block_size]);

        while pos + self.block_size <= self.buffer.len() {
            // Look up current hash in index - get first offset if exists
            let matched_offset = self.index.lookup(current_hash).first().copied();

            if let Some(source_offset) = matched_offset {
                //  Flush any pending INSERT data
                self.flush_insert_buffer();

                // Emit COPY instruction
                self.instructions.push(Instruction::Copy {
                    offset: source_offset,
                    length: self.block_size as u32,
                });

                // Skip past the matched block
                pos += self.block_size;

                // Recalculate hash for new position if we have enough data
                if pos + self.block_size <= self.buffer.len() {
                    current_hash = hasher.hash_chunk(&self.buffer[pos..pos + self.block_size]);
                }
            } else {
                // Add first byte to INSERT buffer
                self.insert_buffer.push(self.buffer[pos]);
                pos += 1;

                // Roll hash forward (O(1) operation!)
                if pos + self.block_size <= self.buffer.len() {
                    let old_byte = self.buffer[pos - 1];
                    let new_byte = self.buffer[pos + self.block_size - 1];
                    current_hash = hasher.roll(old_byte, new_byte);
                }
            }
        }

        // Keep remaining bytes
        self.buffer = self.buffer[pos..].to_vec();
    }

    /// Flust pending INSERT buffer to instructions
    fn flush_insert_buffer(&mut self) {
        if !self.insert_buffer.is_empty() {
            self.instructions.push(Instruction::Insert {
                data: std::mem::take(&mut self.insert_buffer),
            });
        }
    }
    /// finalize and get all instructions
    pub fn finalize(mut self) -> Vec<Instruction> {
        // Flush any remaining pending bytes to INSERT
        self.insert_buffer.extend_from_slice(&self.buffer);
        self.flush_insert_buffer();

        self.instructions
    }

    /// Get current instruction count
    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
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

        let mut diff = StreamingDiff::new(index);
        diff.process_target_chunk(data);
        let instructions = diff.finalize();

        // Should be all COPY instructions
        assert_eq!(instructions.len(), 4);
        for instr in &instructions {
            assert!(matches!(instr, Instruction::Copy { .. }));
        }
    }

    #[test]
    fn test_completely_different() {
        let source = b"aaaabbbbccccdddd";
        let target = b"eeeeffffgggghhhh";
        let index = build_index(source, 4);

        let mut diff = StreamingDiff::new(index);
        diff.process_target_chunk(target);
        let instructions = diff.finalize();

        // Should be all INSERT (no matches)
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

        let mut diff = StreamingDiff::new(index);
        diff.process_target_chunk(target);
        let instructions = diff.finalize();

        // Should have: INSERT(xxxx), COPY(bbbb), INSERT(yyyy), COPY(cccc)
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

        let mut diff = StreamingDiff::new(index);

        // Process in small chunks
        diff.process_target_chunk(b"aaaa");
        diff.process_target_chunk(b"bbbb");
        diff.process_target_chunk(b"cccc");
        diff.process_target_chunk(b"dddd");

        let instructions = diff.finalize();

        // Should still get 4 COPY instructions
        assert_eq!(instructions.len(), 4);
    }

    #[test]
    fn test_unaligned_chunks() {
        let source = b"aaaabbbbccccdddd";
        let index = build_index(source, 4);

        let mut diff = StreamingDiff::new(index);

        // Process in unaligned chunks
        diff.process_target_chunk(b"aa");
        diff.process_target_chunk(b"aabb");
        diff.process_target_chunk(b"bbcc");
        diff.process_target_chunk(b"ccdd");
        diff.process_target_chunk(b"dd");

        let instructions = diff.finalize();

        // Should still recognize all blocks
        assert_eq!(instructions.len(), 4);
    }

    #[test]
    fn test_empty_target() {
        let source = b"aaaabbbb";
        let index = build_index(source, 4);

        let diff = StreamingDiff::new(index);
        let instructions = diff.finalize();

        assert!(instructions.is_empty());
    }

    #[test]
    fn test_target_smaller_than_block() {
        let source = b"aaaabbbb";
        let target = b"xx";
        let index = build_index(source, 4);

        let mut diff = StreamingDiff::new(index);
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

        let mut diff = StreamingDiff::new(index);
        diff.process_target_chunk(b"aaaabbbb");

        // After processing, should have 2 COPY instructions
        assert_eq!(diff.instruction_count(), 2);
    }
}
