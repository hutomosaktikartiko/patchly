//! Patchly WASM - Binary diff & patch engine for WebAssembly
//!
//! Provides streaming APIs for memory-efficient processing of large files.

pub mod diff;
pub mod format;
pub mod utils;

use wasm_bindgen::prelude::*;

use crate::diff::block_index::BlockIndex;
use crate::diff::streaming_diff::StreamingDiff;
use crate::format::patch_format::{
    calculate_hash, HashBuilder, ParsedInstruction, Patch, PatchMetadata,
};
use crate::utils::buffer::ChunkBuffer;

/// Default chunk size for diff matching (4KB)
const DEFAULT_CHUNK_SIZE: usize = 4096;

/// Streaming patch build
#[wasm_bindgen]
pub struct PatchBuilder {
    // Block index for source
    source_index: BlockIndex,
    // Hash builder for source verification
    source_hasher: HashBuilder,
    // Hash builder for target (identical file detection)
    target_hasher: HashBuilder,
    // Total source bytes received
    source_size: u64,
    // Total target bytes received
    target_size: u64,
    // Expected total target size (set upfront header)
    target_total_size: u64,
    // Streaming diff processor
    diff: Option<StreamingDiff>,
    // Whether source has been finalized
    source_finalized: bool,
    // Chunk size for matching
    chunk_size: usize,
    // Serialized patch data ready to output
    output_buffer: Vec<u8>,
    // Whether header has been written
    header_written: bool,
    // Whether all target data processed
    target_finalized: bool,
}

#[wasm_bindgen]
impl PatchBuilder {
    /// Create a new PatchBuilder with default chunk size
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

    /// Add a chunk of source (old file) data.
    #[wasm_bindgen]
    pub fn add_source_chunk(&mut self, chunk: &[u8]) {
        if self.source_finalized {
            return;
        }

        self.source_hasher.update(chunk);
        self.source_index.add_chunk(chunk);
        self.source_size += chunk.len() as u64;
    }

    /// Finalize source processing.
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

    /// Set the expected total target size.
    /// Must be called before add_target_chunk() for proper header generation.
    #[wasm_bindgen]
    pub fn set_target_size(&mut self, size: u64) {
        self.target_total_size = size;
    }

    /// Add a chunk of target (new file) data.
    /// This immediately generates patch output - call flush_output() to retrieve it.
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

    /// Finalize target processing.
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

    /// Get current source size (bytes received so far).
    #[wasm_bindgen]
    pub fn source_size(&self) -> usize {
        self.source_size as usize
    }

    /// Get current target size (bytes received so far).
    #[wasm_bindgen]
    pub fn target_size(&self) -> usize {
        self.target_size as usize
    }

    /// Check if source and target files are identical.
    /// Only accurate after all data has been processed.
    #[wasm_bindgen]
    pub fn are_files_identical(&self) -> bool {
        let same_size = self.source_size == self.target_size;
        let same_hash = self.source_hasher.finalize() == self.target_hasher.finalize();
        same_size && same_hash
    }

    /// Check if there's patch output available to read.
    #[wasm_bindgen]
    pub fn has_output(&self) -> bool {
        // Has output if: header not written yet, OR there's data in buffer
        !self.header_written || !self.output_buffer.is_empty()
    }

    /// Get next chunk of patch output.
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

    /// Get approximate pending output size
    #[wasm_bindgen]
    pub fn pending_output_size(&self) -> usize {
        self.output_buffer.len()
    }

    /// Reset the builder for reuse.
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

/// Applier for pacthes with streaming output supoort.
#[wasm_bindgen]
pub struct PatchApplier {
    // Buffer for source file chunks (used directly for random access, no merge needed)
    source_buffer: ChunkBuffer,
    // Hash builder for source verification
    source_hasher: HashBuilder,
    // Patch data
    patch_data: Vec<u8>,
    // Whether patch has been loaded
    patch_loaded: bool,
    // Parsed instructions
    instructions: Vec<ParsedInstruction>,
    // Current instruction index
    current_instruction: usize,
    // Offset within current instruction
    instruction_offset: usize,
    // Total bytes written so far
    output_written: u64,
    // Expected target size
    target_size: u64,
    // Whether prepare has been called
    prepared: bool,
}

#[wasm_bindgen]
impl PatchApplier {
    /// Create a new PatchApplier
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            source_buffer: ChunkBuffer::new(),
            source_hasher: HashBuilder::new(),
            patch_data: Vec::new(),
            patch_loaded: false,
            instructions: Vec::new(),
            current_instruction: 0,
            instruction_offset: 0,
            output_written: 0,
            target_size: 0,
            prepared: false,
        }
    }

    /// Add a chunk of source (old file) data.
    #[wasm_bindgen]
    pub fn add_source_chunk(&mut self, chunk: &[u8]) {
        self.source_hasher.update(chunk);
        self.source_buffer.push_slice(chunk);
    }

    /// Set the patch data.
    #[wasm_bindgen]
    pub fn set_patch(&mut self, patch_data: &[u8]) {
        self.patch_data = patch_data.to_vec();
        self.patch_loaded = true;
    }

    /// Get current source size.
    #[wasm_bindgen]
    pub fn source_size(&self) -> usize {
        self.source_buffer.total_size()
    }

    /// Validate source file before applying.
    #[wasm_bindgen]
    pub fn validate_source(&self) -> Result<bool, JsError> {
        if !self.patch_loaded {
            return Err(JsError::new("Patch not loaded"));
        }

        let patch = Patch::deserialize(&self.patch_data)
            .map_err(|e| JsError::new(&format!("Invalid patch: {}", e)))?;

        let source_hash = self.source_hasher.finalize();
        let source_size = self.source_buffer.total_size() as u64;

        match patch.validate_source(source_size, source_hash) {
            Ok(_) => Ok(true),
            Err(e) => Err(JsError::new(&format!("Validation failed: {}", e))),
        }
    }

    /// Get expected source size from patch metadata.
    #[wasm_bindgen]
    pub fn expected_source_size(&self) -> Result<u64, JsError> {
        if !self.patch_loaded {
            return Err(JsError::new("Patch not loaded"));
        }

        let patch = Patch::deserialize(&self.patch_data)
            .map_err(|e| JsError::new(&format!("Invalid patch: {}", e)))?;

        Ok(patch.source_size)
    }

    /// Get expected target size from patch metadata.
    #[wasm_bindgen]
    pub fn expected_target_size(&self) -> Result<u64, JsError> {
        if !self.patch_loaded {
            return Err(JsError::new("Patch not loaded"));
        }

        let patch = Patch::deserialize(&self.patch_data)
            .map_err(|e| JsError::new(&format!("Invalid patch: {}", e)))?;

        Ok(patch.target_size)
    }

    /// Prepare for streaming output.
    #[wasm_bindgen]
    pub fn prepare(&mut self) -> Result<(), JsError> {
        if !self.patch_loaded {
            return Err(JsError::new("Patch not loaded"));
        }

        // No merge needed - use source_buffer directly via read_at()
        // This avoids doubling memory during the merge step

        // Parse patch to get instructions
        let patch = PatchMetadata::parse(&self.patch_data)
            .map_err(|e| JsError::new(&format!("Invalid patch: {}", e)))?;

        // Store instructions and metadata
        self.instructions = patch.instructions;
        self.target_size = patch.target_size;
        self.current_instruction = 0;
        self.instruction_offset = 0;
        self.output_written = 0;
        self.prepared = true;

        Ok(())
    }

    /// Check if there's more output to read.
    #[wasm_bindgen]
    pub fn has_more_output(&self) -> bool {
        if !self.prepared {
            return false;
        }

        self.current_instruction < self.instructions.len()
    }

    /// Get next chunk of output data.
    #[wasm_bindgen]
    pub fn next_output_chunk(&mut self, max_size: usize) -> Vec<u8> {
        if !self.prepared || self.current_instruction >= self.instructions.len() {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(max_size);

        while result.len() < max_size && self.current_instruction < self.instructions.len() {
            let instruction = &self.instructions[self.current_instruction];

            match instruction {
                ParsedInstruction::Copy { offset, length } => {
                    let offset = *offset as usize;
                    let length = *length as usize;

                    // Calculate how much of this Copy we still need to output
                    let remaining_in_instruction = length - self.instruction_offset;
                    let space_in_result = max_size - result.len();
                    let bytes_to_copy = remaining_in_instruction.min(space_in_result);

                    // Read from source_buffer using read_at (no merge needed)
                    let start = offset + self.instruction_offset;
                    if let Some(data) = self.source_buffer.read_at(start, bytes_to_copy) {
                        result.extend_from_slice(&data);
                    }

                    self.instruction_offset += bytes_to_copy;

                    // Move to next instruction if this one is complete
                    if self.instruction_offset >= length {
                        self.current_instruction += 1;
                        self.instruction_offset = 0;
                    }
                }
                ParsedInstruction::Insert {
                    patch_offset,
                    length,
                } => {
                    let length = *length as usize;

                    // Calculate how much of this Insert we still need to output
                    let remaining_in_instruction = length - self.instruction_offset;
                    let space_in_result = max_size - result.len();
                    let bytes_to_copy = remaining_in_instruction.min(space_in_result);

                    // Read INSERT data directly from patch_data
                    let start = patch_offset + self.instruction_offset;
                    let end = start + bytes_to_copy;
                    result.extend_from_slice(&self.patch_data[start..end]);

                    self.instruction_offset += bytes_to_copy;

                    // Move to next instruction if this one is complete
                    if self.instruction_offset >= length {
                        self.current_instruction += 1;
                        self.instruction_offset = 0;
                    }
                }
            }
        }

        self.output_written += result.len() as u64;
        result
    }

    /// Add a chunk of patch data.
    #[wasm_bindgen]
    pub fn add_patch_chunk(&mut self, chunk: &[u8]) {
        self.patch_data.extend_from_slice(chunk);
    }

    /// Finalize patch loading and parse header.
    #[wasm_bindgen]
    pub fn finalize_patch(&mut self) -> Result<(), JsError> {
        if self.patch_data.len() < 33 {
            return Err(JsError::new("Patch data too small"));
        }
        self.patch_loaded = true;
        Ok(())
    }

    /// Get remaining bytes to output
    #[wasm_bindgen]
    pub fn remaining_output_size(&self) -> u64 {
        if !self.prepared || self.target_size == 0 {
            return 0;
        }

        self.target_size.saturating_sub(self.output_written)
    }

    /// Reset the applier for reuse.
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.source_buffer.clear();
        self.source_buffer.shrink_to_fit();

        self.source_hasher = HashBuilder::new();

        self.patch_data.clear();
        self.patch_data.shrink_to_fit();

        self.patch_loaded = false;

        self.instructions.clear();
        self.instructions.shrink_to_fit();

        self.current_instruction = 0;
        self.instruction_offset = 0;
        self.output_written = 0;
        self.target_size = 0;
        self.prepared = false;
    }
}

impl Default for PatchApplier {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse patch header and return JSON with metadata and instructions.
/// This is a lightweight function for the TypeScript-based applier.
/// Returns JSON string with structure:
/// {
///   "sourceSize": number,
///   "sourceHash": string (hex),
///   "targetSize": number,
///   "instructions": [
///     { "type": "copy", "offset": number, "length": number } |
///     { "type": "insert", "patchOffset": number, "length": number }
///   ]
/// }
#[wasm_bindgen]
pub fn parse_patch_header(patch_data: &[u8]) -> Result<String, JsError> {
    let metadata = PatchMetadata::parse(patch_data)
        .map_err(|e| JsError::new(&format!("Failed to parse patch: {}", e)))?;

    // Build JSON manually (no serde dependency needed)
    let mut json = String::from("{");
    json.push_str(&format!("\"sourceSize\":{},", metadata.source_size));
    json.push_str(&format!(
        "\"sourceHash\":\"{:016x}\",",
        metadata.source_hash
    ));
    json.push_str(&format!("\"targetSize\":{},", metadata.target_size));
    json.push_str("\"instructions\":[");

    for (i, instr) in metadata.instructions.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        match instr {
            ParsedInstruction::Copy { offset, length } => {
                json.push_str(&format!(
                    "{{\"type\":\"copy\",\"offset\":{},\"length\":{}}}",
                    offset, length
                ));
            }
            ParsedInstruction::Insert {
                patch_offset,
                length,
            } => {
                json.push_str(&format!(
                    "{{\"type\":\"insert\",\"patchOffset\":{},\"length\":{}}}",
                    patch_offset, length
                ));
            }
        }
    }

    json.push_str("]}");
    Ok(json)
}

/// Parse ONLY the patch header (33 bytes) without parsing instructions.
/// Returns JSON: { "sourceSize": number, "sourceHash": string, "targetSize": number, "headerSize": 33 }
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

/// Get the library version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// WASM-bindable streaming hash builder.
/// Use this to calculate hash incrementally from JavaScript without BigInt allocations.
#[wasm_bindgen]
pub struct WasmHashBuilder {
    inner: HashBuilder,
}

#[wasm_bindgen]
impl WasmHashBuilder {
    /// Create a new hash builder
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: HashBuilder::new(),
        }
    }

    /// Update the hash with a chunk of data
    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    /// Finalize and return the hash as a hex string
    pub fn finalize(&self) -> String {
        format!("{:016x}", self.inner.finalize())
    }

    /// Finalize and return the hash as a u64 (for comparison)
    pub fn finalize_u64(&self) -> u64 {
        self.inner.finalize()
    }
}

/// Calculate hash of data
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
