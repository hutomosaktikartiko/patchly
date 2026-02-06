//! Patchly WASM - Binary diff & patch engine for WebAssembly
//!
//! Provides streaming APIs for memory-efficient processing of large files.

pub mod diff;
pub mod format;

use wasm_bindgen::prelude::*;

use crate::diff::block_index::BlockIndex;
use crate::diff::streaming_diff::StreamingDiff;
use crate::format::patch_format::{calculate_hash, HashBuilder};

/// Default chunk size for diff matching (4KB)
const DEFAULT_CHUNK_SIZE: usize = 4096;

/// Streaming binary patch builder.
///
/// Processes source and target files in chunks to generate a binary patch.
/// Designed for memory-efficient handling of large files (multi-GB).
#[wasm_bindgen]
pub struct PatchBuilder {
    /// Block index for source file.
    source_index: BlockIndex,
    /// Hash builder for source verification.
    source_hasher: HashBuilder,
    /// Hash builder for target (identical file detection).
    target_hasher: HashBuilder,
    /// Total source bytes received.
    source_size: u64,
    /// Total target bytes received.
    target_size: u64,
    /// Expected total target size (for header).
    target_total_size: u64,
    /// Streaming diff processor.
    diff: Option<StreamingDiff>,
    /// Whether source has been finalized.
    source_finalized: bool,
    /// Chunk size for matching.
    chunk_size: usize,
    /// Serialized patch data ready to output.
    output_buffer: Vec<u8>,
    /// Whether header has been written.
    header_written: bool,
    /// Whether all target data has been processed.
    target_finalized: bool,
}

#[wasm_bindgen]
impl PatchBuilder {
    /// Creates a new `PatchBuilder` with default chunk size.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            source_index: BlockIndex::with_block_size(DEFAULT_CHUNK_SIZE),
            source_hasher: HashBuilder::new(),
            target_hasher: HashBuilder::new(),
            source_size: 0,
            target_size: 0,
            target_total_size: 0,
            diff: None,
            source_finalized: false,
            chunk_size: DEFAULT_CHUNK_SIZE,
            output_buffer: Vec::new(),
            header_written: false,
            target_finalized: false,
        }
    }

    /// Adds a chunk of source (old file) data.
    #[wasm_bindgen]
    pub fn add_source_chunk(&mut self, chunk: &[u8]) {
        if self.source_finalized {
            return;
        }

        self.source_hasher.update(chunk);
        self.source_index.add_chunk(chunk);
        self.source_size += chunk.len() as u64;
    }

    /// Finalizes source processing.
    #[wasm_bindgen]
    pub fn finalize_source(&mut self) {
        if self.source_finalized {
            return;
        }

        self.source_index.finalize();

        // Create StreamingDiff with the built index
        let index = std::mem::replace(
            &mut self.source_index,
            BlockIndex::with_block_size(self.chunk_size),
        );
        self.diff = Some(StreamingDiff::new(index));
        self.source_finalized = true;
    }

    /// Sets the expected total target size.
    ///
    /// Must be called before `add_target_chunk()` for proper header generation.
    #[wasm_bindgen]
    pub fn set_target_size(&mut self, size: u64) {
        self.target_total_size = size;
    }

    /// Adds a chunk of target (new file) data.
    ///
    /// Generates patch output immediately; call `flush_output()` to retrieve it.
    #[wasm_bindgen]
    pub fn add_target_chunk(&mut self, chunk: &[u8]) {
        if !self.source_finalized {
            self.finalize_source();
        }

        self.target_hasher.update(chunk);
        self.target_size += chunk.len() as u64;

        if let Some(diff) = &mut self.diff {
            diff.process_target_chunk(chunk);

            // Pull output from diff into our buffer
            if diff.has_output() {
                let output = diff.take_output();
                self.output_buffer.extend_from_slice(&output);
            }
        }
    }

    /// Finalizes target processing.
    ///
    /// Call this after all target chunks have been added.
    #[wasm_bindgen]
    pub fn finalize_target(&mut self) {
        if self.target_finalized {
            return;
        }

        if let Some(diff) = &mut self.diff {
            diff.finalize();

            // Pull final output
            if diff.has_output() {
                let output = diff.take_output();
                self.output_buffer.extend_from_slice(&output);
            }
        }

        self.target_finalized = true;
    }

    /// Returns the current source size in bytes.
    #[wasm_bindgen]
    pub fn source_size(&self) -> usize {
        self.source_size as usize
    }

    /// Returns the current target size in bytes.
    #[wasm_bindgen]
    pub fn target_size(&self) -> usize {
        self.target_size as usize
    }

    /// Checks if source and target files are identical.
    ///
    /// Only accurate after all data has been processed.
    #[wasm_bindgen]
    pub fn are_files_identical(&self) -> bool {
        let same_size = self.source_size == self.target_size;
        let same_hash = self.source_hasher.finalize() == self.target_hasher.finalize();
        same_size && same_hash
    }

    /// Checks if there's patch output available to read.
    #[wasm_bindgen]
    pub fn has_output(&self) -> bool {
        // Has output if: header not written yet, OR there's data in buffer
        !self.header_written || !self.output_buffer.is_empty()
    }

    /// Returns the next chunk of patch output.
    ///
    /// Returns serialized patch data ready to write to file.
    #[wasm_bindgen]
    pub fn flush_output(&mut self, max_size: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(max_size);

        // Write header first if not written
        if !self.header_written {
            let source_hash = self.source_hasher.finalize();

            // Magic(4) + version(1) + chunk_size(4) + source_size(8) + source_hash(8) + target_size(8) = 33 bytes
            result.extend_from_slice(b"PTCH");
            result.push(1); // VERSION
            result.extend_from_slice(&(self.chunk_size as u32).to_le_bytes());
            result.extend_from_slice(&self.source_size.to_le_bytes());
            result.extend_from_slice(&source_hash.to_le_bytes());
            result.extend_from_slice(&self.target_total_size.to_le_bytes());

            self.header_written = true;
        }

        // Fill remaining space with output buffer data
        let space_remaining = max_size - result.len();
        let bytes_to_take = space_remaining.min(self.output_buffer.len());

        if bytes_to_take > 0 {
            // Take from front of output_buffer
            result.extend_from_slice(&self.output_buffer[..bytes_to_take]);

            // Remove taken bytes from buffer
            self.output_buffer = self.output_buffer[bytes_to_take..].to_vec();
        }

        result
    }

    /// Returns the approximate pending output size.
    #[wasm_bindgen]
    pub fn pending_output_size(&self) -> usize {
        self.output_buffer.len()
    }

    /// Resets the builder for reuse.
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.source_index = BlockIndex::with_block_size(self.chunk_size);
        self.source_hasher = HashBuilder::new();
        self.target_hasher = HashBuilder::new();
        self.source_size = 0;
        self.target_size = 0;
        self.target_total_size = 0;
        self.diff = None;
        self.source_finalized = false;
        self.output_buffer.clear();
        self.header_written = false;
        self.target_finalized = false;
    }
}

impl Default for PatchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Parses only the patch header (33 bytes) without parsing instructions.
///
/// Returns JSON with sourceSize, sourceHash, targetSize, chunkSize, and headerSize.
/// TypeScript will parse instructions directly from OPFS to avoid loading entire patch.
#[wasm_bindgen]
pub fn parse_patch_header_only(header_data: &[u8]) -> Result<String, JsError> {
    if header_data.len() < 33 {
        return Err(JsError::new(
            "Header data too small (need at least 33 bytes)",
        ));
    }

    // Validate magic
    if &header_data[0..4] != b"PTCH" {
        return Err(JsError::new("Invalid patch file: bad magic bytes"));
    }

    // Validate version
    if header_data[4] != 1 {
        return Err(JsError::new(&format!(
            "Unsupported patch version: {} (expected 1)",
            header_data[4]
        )));
    }

    // Parse fields
    let chunk_size = u32::from_le_bytes([
        header_data[5],
        header_data[6],
        header_data[7],
        header_data[8],
    ]);
    let source_size = u64::from_le_bytes([
        header_data[9],
        header_data[10],
        header_data[11],
        header_data[12],
        header_data[13],
        header_data[14],
        header_data[15],
        header_data[16],
    ]);
    let source_hash = u64::from_le_bytes([
        header_data[17],
        header_data[18],
        header_data[19],
        header_data[20],
        header_data[21],
        header_data[22],
        header_data[23],
        header_data[24],
    ]);
    let target_size = u64::from_le_bytes([
        header_data[25],
        header_data[26],
        header_data[27],
        header_data[28],
        header_data[29],
        header_data[30],
        header_data[31],
        header_data[32],
    ]);

    Ok(format!(
        "{{\"sourceSize\":{},\"sourceHash\":\"{:016x}\",\"targetSize\":{},\"chunkSize\":{},\"headerSize\":33}}",
        source_size, source_hash, target_size, chunk_size
    ))
}

/// Returns the library version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// WASM-bindable streaming hash builder.
///
/// Use this to calculate hash incrementally from JavaScript without BigInt allocations.
#[wasm_bindgen]
pub struct StreamingHasher {
    /// Inner hash builder.
    inner: HashBuilder,
}

#[wasm_bindgen]
impl StreamingHasher {
    /// Creates a new hash builder.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: HashBuilder::new(),
        }
    }

    /// Updates the hash with a chunk of data.
    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    /// Finalizes and returns the hash as a hex string.
    pub fn finalize(&self) -> String {
        format!("{:016x}", self.inner.finalize())
    }

    /// Finalizes and returns the hash as a u64 for comparison.
    pub fn finalize_u64(&self) -> u64 {
        self.inner.finalize()
    }
}

/// Calculates hash of data and returns it as a hex string.
#[wasm_bindgen]
pub fn hash_data(data: &[u8]) -> String {
    format!("{:016x}", calculate_hash(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_files_detection() {
        let data = b"identical content here";

        let mut builder = PatchBuilder::new();
        builder.add_source_chunk(data);
        builder.finalize_source();
        builder.add_target_chunk(data);

        assert!(builder.are_files_identical());
    }

    #[test]
    fn test_different_files_detection() {
        let source = b"original content";
        let target = b"modified content";

        let mut builder = PatchBuilder::new();
        builder.add_source_chunk(source);
        builder.finalize_source();
        builder.add_target_chunk(target);

        assert!(!builder.are_files_identical());
    }

    #[test]
    fn test_reset() {
        let mut builder = PatchBuilder::new();
        builder.add_source_chunk(b"test");
        builder.finalize_source();
        builder.add_target_chunk(b"data");

        assert!(builder.source_size() > 0);

        builder.reset();

        assert_eq!(builder.source_size(), 0);
        assert_eq!(builder.target_size(), 0);
    }

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }

    #[test]
    fn test_hash_data() {
        let data = b"hello world";
        let hash = hash_data(data);

        assert_eq!(hash.len(), 16);
        assert_eq!(hash, hash_data(data));
    }
}
