//! Streaming diff generator for patch creation.
//!
//! Processes target file chunks and generates serialized patch instructions
//! by comparing against a pre-built source file index.

use super::block_index::BlockIndex;
use super::rolling_hash::RollingHash;
use crate::format::patch_format::{TYPE_COPY, TYPE_INSERT};

/// Streaming diff generator that outputs serialized patch data directly.
///
/// Compares target file data against a source file's block index to find
/// matching blocks (COPY) and new data (INSERT).
pub struct StreamingDiff {
    /// Block index built from source file.
    index: BlockIndex,
    /// Block size for matching.
    block_size: usize,
    /// Buffer for pending target data to process.
    buffer: Vec<u8>,
    /// Pending INSERT data.
    insert_buffer: Vec<u8>,
    /// Serialized output ready to be consumed.
    output_buffer: Vec<u8>,
}

impl StreamingDiff {
    /// Creates a new `StreamingDiff` from a `BlockIndex`.
    pub fn new(index: BlockIndex) -> Self {
        let block_size = index.block_size();

        Self {
            index,
            block_size,
            buffer: Vec::new(),
            insert_buffer: Vec::new(),
            output_buffer: Vec::new(),
        }
    }

    /// Processes a chunk of target data.
    ///
    /// This may generate serialized output in the output buffer.
    pub fn process_target_chunk(&mut self, chunk: &[u8]) {
        self.buffer.extend_from_slice(chunk);
        self.process_buffer();
    }

    /// Processes buffered data using rolling hash.
    fn process_buffer(&mut self) {
        if self.buffer.len() < self.block_size {
            return;
        }

        let mut hasher = RollingHash::new(self.block_size);
        let mut pos = 0;

        // Calculate initial hash
        let mut current_hash = hasher.hash_chunk(&self.buffer[0..self.block_size]);

        while pos + self.block_size <= self.buffer.len() {
            // Look up current hash in index
            let matched_offset = self.index.lookup(current_hash).first().copied();

            if let Some(source_offset) = matched_offset {
                // Found a match - flush pending INSERT data first
                self.flush_insert_buffer();

                // Emit COPY instruction
                self.emit_copy(source_offset, self.block_size as u32);

                // Skip past the matched block
                pos += self.block_size;

                // Recalculate hash for new position
                if pos + self.block_size <= self.buffer.len() {
                    current_hash = hasher.hash_chunk(&self.buffer[pos..pos + self.block_size]);
                }
            } else {
                // No match - add byte to INSERT buffer
                self.insert_buffer.push(self.buffer[pos]);
                pos += 1;

                // Roll hash forward - O(1) operation
                if pos + self.block_size <= self.buffer.len() {
                    let old_byte = self.buffer[pos - 1];
                    let new_byte = self.buffer[pos + self.block_size - 1];
                    current_hash = hasher.roll(old_byte, new_byte);
                }
            }
        }

        // Keep remaining bytes that couldn't form a complete block
        self.buffer = self.buffer[pos..].to_vec();
    }

    /// Flushes pending INSERT data to output buffer.
    fn flush_insert_buffer(&mut self) {
        if !self.insert_buffer.is_empty() {
            // Serialize INSERT: type(1) + length(4) + data
            self.output_buffer.push(TYPE_INSERT);
            self.output_buffer
                .extend_from_slice(&(self.insert_buffer.len() as u32).to_le_bytes());
            self.output_buffer.extend_from_slice(&self.insert_buffer);

            self.insert_buffer.clear();
        }
    }

    /// Emits a COPY instruction to output buffer.
    fn emit_copy(&mut self, offset: u64, length: u32) {
        // Serialize COPY: type(1) + offset(8) + length(4)
        self.output_buffer.push(TYPE_COPY);
        self.output_buffer.extend_from_slice(&offset.to_le_bytes());
        self.output_buffer.extend_from_slice(&length.to_le_bytes());
    }

    /// Finalizes processing and flushes remaining data.
    pub fn finalize(&mut self) {
        // Any remaining bytes in buffer go to INSERT
        self.insert_buffer.extend_from_slice(&self.buffer);
        self.buffer.clear();

        // Flush final INSERT if any
        self.flush_insert_buffer();
    }

    /// Takes the output buffer, transferring ownership.
    ///
    /// Returns serialized patch instructions ready to write.
    pub fn take_output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.output_buffer)
    }

    /// Returns the current output buffer size.
    pub fn output_len(&self) -> usize {
        self.output_buffer.len()
    }

    /// Checks if there's pending output to consume.
    pub fn has_output(&self) -> bool {
        !self.output_buffer.is_empty()
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
    fn test_streaming_output() {
        let source = b"aaaabbbbccccdddd";
        let target = b"aaaabbbbccccdddd";
        let index = build_index(source, 4);

        let mut diff = StreamingDiff::new(index);
        diff.process_target_chunk(target);
        diff.finalize();

        let output = diff.take_output();

        // Should have 4 COPY instructions (13 bytes each)
        // Each COPY: type(1) + offset(8) + length(4) = 13 bytes
        assert_eq!(output.len(), 4 * 13);

        // First byte of each instruction should be TYPE_COPY
        assert_eq!(output[0], 0x01);
        assert_eq!(output[13], 0x01);
        assert_eq!(output[26], 0x01);
        assert_eq!(output[39], 0x01);
    }

    #[test]
    fn test_insert_output() {
        let source = b"aaaabbbb";
        let target = b"xxxx"; // Completely different
        let index = build_index(source, 4);

        let mut diff = StreamingDiff::new(index);
        diff.process_target_chunk(target);
        diff.finalize();

        let output = diff.take_output();

        // Should have 1 INSERT instruction
        // INSERT: type(1) + length(4) + data(4) = 9 bytes
        assert_eq!(output.len(), 9);
        assert_eq!(output[0], 0x02); // TYPE_INSERT

        // Length should be 4
        let length = u32::from_le_bytes([output[1], output[2], output[3], output[4]]);
        assert_eq!(length, 4);

        // Data should be "xxxx"
        assert_eq!(&output[5..9], b"xxxx");
    }

    #[test]
    fn test_mixed_output() {
        let source = b"aaaabbbbccccdddd";
        let target = b"xxxxbbbbyyyycccc";
        let index = build_index(source, 4);

        let mut diff = StreamingDiff::new(index);
        diff.process_target_chunk(target);
        diff.finalize();

        let output = diff.take_output();

        // Should have: INSERT(xxxx) + COPY(bbbb) + INSERT(yyyy) + COPY(cccc)
        // INSERT: 1 + 4 + 4 = 9 bytes
        // COPY: 1 + 8 + 4 = 13 bytes
        // Total: 9 + 13 + 9 + 13 = 44 bytes
        assert_eq!(output.len(), 44);
    }

    #[test]
    fn test_incremental_output() {
        let source = b"aaaabbbbccccdddd";
        let index = build_index(source, 4);

        let mut diff = StreamingDiff::new(index);

        // Process in small chunks
        diff.process_target_chunk(b"aaaa");
        let out1 = diff.output_len();

        diff.process_target_chunk(b"bbbb");
        let out2 = diff.output_len();

        // Output should grow as we process
        assert!(out2 >= out1);

        diff.process_target_chunk(b"cccc");
        diff.process_target_chunk(b"dddd");
        diff.finalize();

        let output = diff.take_output();
        assert_eq!(output.len(), 4 * 13); // 4 COPY instructions
    }
}
