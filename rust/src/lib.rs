//! Patchly WASM - Binary diff & patch engine for WebAssembly
//!
//! Provides streaming APIs for memory-efficient processing of large files.

pub mod diff;
pub mod format;
pub mod utils;

use wasm_bindgen::prelude::*;

use crate::diff::block_index::BlockIndex;
use crate::diff::patch::apply_patch;
use crate::diff::streaming_diff::StreamingDiff;
use crate::format::patch_format::{calculate_hash, HashBuilder, Patch};
use crate::utils::buffer::ChunkBuffer;

/// Default chunk size for diff matching (4KB)
const DEFAULT_CHUNK_SIZE: usize = 4 * 1024;

/// Default output chunk size for streaming (1MB)
const DEFAULT_OUTPUT_CHUNK_SIZE: usize = 1024 * 1024;

/// Streaming patch build
///
/// # Memory Usage
/// - Source: O(blocks) - use BlockIndex
/// - Target: Processed incrementally via StreamingDiff
///
/// # Usage Flow
/// 1. Call add_source_chunk() for all source data
/// 2. Call finalize_source() when done with source
/// 3. Call add_target_chunk() for all target data
/// 4. Call finalize() to get the patch
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
    // Streaming diff processor (created after finalize_source)
    diff: Option<StreamingDiff>,
    // Whether source has been finalized
    source_finalized: bool,
    // Chunk size for matching
    chunk_size: usize,
}

#[wasm_bindgen]
impl PatchBuilder {
    /// Create a new PatchBuilder with default chunk size
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::with_chunk_size(DEFAULT_CHUNK_SIZE)
    }

    /// Create a new PatchBuilder with custom chunk size.
    #[wasm_bindgen]
    pub fn with_chunk_size(chunk_size: usize) -> Self {
        Self {
            source_index: BlockIndex::with_block_size(chunk_size),
            source_hasher: HashBuilder::new(),
            target_hasher: HashBuilder::new(),
            source_size: 0,
            target_size: 0,
            diff: None,
            source_finalized: false,
            chunk_size,
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

    /// Add a chunk of target (new file) data.
    #[wasm_bindgen]
    pub fn add_target_chunk(&mut self, chunk: &[u8]) {
        if !self.source_finalized {
            self.finalize_source();
        }

        self.target_hasher.update(chunk);
        self.target_size += chunk.len() as u64;

        if let Some(diff) = &mut self.diff {
            diff.process_target_chunk(chunk);
        }
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

    /// Check if source and target files are indentical.
    /// Files are identical if both size AND hash match
    #[wasm_bindgen]
    pub fn are_files_identical(&self) -> bool {
        let same_size = self.source_size == self.target_size;
        let same_hash = self.source_hasher.finalize() == self.target_hasher.finalize();

        same_size && same_hash
    }

    /// Finalize and generate the patch.
    /// returns serialized patch data.
    #[wasm_bindgen]
    pub fn finalize(&mut self) -> Result<Vec<u8>, JsError> {
        // Ensure source is finalized
        if !self.source_finalized {
            self.finalize_source();
        }

        // Take the diff and finalize it
        let diff = self
            .diff
            .take()
            .ok_or_else(|| JsError::new("No diff processor available"))?;

        let instructions = diff.finalize();

        // Create patch with metadata
        let source_hash = self.source_hasher.finalize();
        let mut patch = Patch::new(
            self.chunk_size as u32,
            self.source_size,
            source_hash,
            self.target_size,
        );
        patch.instructions = instructions;

        patch
            .serialize()
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }

    /// Get progress as percentage (0-100) based on expected sizes.
    #[wasm_bindgen]
    pub fn progress(&self, source_expected: usize, target_expected: usize) -> f64 {
        let total_expected = source_expected + target_expected;
        if total_expected == 0 {
            return 100.0;
        }
        let total_received = self.source_size + self.target_size;
        (total_received as f64 / total_expected as f64 * 100.0).min(100.0)
    }

    /// Reset the builder for reuse.
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.source_index = BlockIndex::with_block_size(self.chunk_size);
        self.source_hasher = HashBuilder::new();
        self.target_hasher = HashBuilder::new();
        self.source_size = 0;
        self.target_size = 0;
        self.diff = None;
        self.source_finalized = false;
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
    source_buffer: ChunkBuffer,
    source_hasher: HashBuilder,
    patch_data: Vec<u8>,
    patch_loaded: bool,
    output_buffer: Option<Vec<u8>>,
    output_position: usize,
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
            output_buffer: None,
            output_position: 0,
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

    /// Check if patch has been loaded.
    #[wasm_bindgen]
    pub fn is_patch_loaded(&self) -> bool {
        self.patch_loaded
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

        let source = self.source_buffer.merge();
        let patch = Patch::deserialize(&self.patch_data)
            .map_err(|e| JsError::new(&format!("Invalid patch: {}", e)))?;

        let output = apply_patch(&source, &patch)
            .map_err(|e| JsError::new(&format!("Apply error: {}", e)))?;

        self.output_buffer = Some(output);
        self.output_position = 0;
        self.prepared = true;

        Ok(())
    }

    /// Check if there's more output to read
    #[wasm_bindgen]
    pub fn has_more_output(&self) -> bool {
        match &self.output_buffer {
            Some(buf) => self.output_position < buf.len(),
            None => false,
        }
    }

    /// Get output progress as percentage (0-100).
    #[wasm_bindgen]
    pub fn output_progress(&self) -> f64 {
        match &self.output_buffer {
            Some(buf) if !buf.is_empty() => {
                (self.output_position as f64 / buf.len() as f64 * 100.0).min(100.0)
            }
            _ => 0.0,
        }
    }

    /// Get next chunk of output for streaming to OPFS.
    #[wasm_bindgen]
    pub fn next_output_chunk(&mut self, max_chunk_size: usize) -> Vec<u8> {
        let chunk_size = if max_chunk_size == 0 {
            DEFAULT_OUTPUT_CHUNK_SIZE
        } else {
            max_chunk_size
        };

        match &self.output_buffer {
            Some(buf) => {
                if self.output_position > buf.len() {
                    return Vec::new();
                }

                let end = (self.output_position + chunk_size).min(buf.len());
                let chunk = buf[self.output_position..end].to_vec();
                self.output_position = end;
                chunk
            }
            None => Vec::new(),
        }
    }

    /// Get total output size
    #[wasm_bindgen]
    pub fn total_output_size(&self) -> usize {
        match &self.output_buffer {
            Some(buf) => buf.len(),
            None => 0,
        }
    }

    /// Get remaining bytes to output
    #[wasm_bindgen]
    pub fn remaining_output_size(&self) -> usize {
        match &self.output_buffer {
            Some(buf) => buf.len().saturating_sub(self.output_position),
            None => 0,
        }
    }

    /// Reset the applier for reuse.
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.source_buffer.clear();
        self.source_hasher = HashBuilder::new();
        self.patch_data.clear();
        self.patch_loaded = false;
        self.output_buffer = None;
        self.output_position = 0;
        self.prepared = false;
    }

    /// Get patch info as JSON string.
    #[wasm_bindgen]
    pub fn get_patch_info(&self) -> Result<String, JsError> {
        if !self.patch_loaded {
            return Err(JsError::new("Patch not loaded"));
        }

        let patch = Patch::deserialize(&self.patch_data)
            .map_err(|e| JsError::new(&format!("Invalid patch: {}", e)))?;

        let stats = patch.stats();

        Ok(format!(
            r#"{{"sourceSize":{},"sourceHash":{},"targetSize":{},"chunkSize":{},"copyCount":{},"copyBytes":{},"insertCount":{},"insertBytes":{},"totalInstructions":{}}}"#,
            patch.source_size,
            format!("{:016x}", patch.source_hash),
            patch.target_size,
            patch.chunk_size,
            stats.copy_count,
            stats.copy_bytes,
            stats.insert_count,
            stats.insert_bytes,
            patch.instruction_count(),
        ))
    }
}

impl Default for PatchApplier {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the library version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
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
    fn test_streaming_patch_builder_basic() {
        let source = b"aaaabbbbccccdddd";
        let target = b"aaaaNEWWccccdddd";

        let mut builder = PatchBuilder::with_chunk_size(4);
        builder.add_source_chunk(source);
        builder.finalize_source();
        builder.add_target_chunk(target);

        let patch_data = builder.finalize().unwrap();
        assert!(!patch_data.is_empty());

        // Verify patch can be applied
        let mut applier = PatchApplier::new();
        applier.add_source_chunk(source);
        applier.set_patch(&patch_data);
        applier.prepare().unwrap();

        let mut result = Vec::new();
        while applier.has_more_output() {
            result.extend(applier.next_output_chunk(100));
        }

        assert_eq!(result, target);
    }

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
    fn test_chunked_source_input() {
        let source = b"aaaabbbbccccdddd";
        let target = b"aaaabbbbccccdddd";

        let mut builder = PatchBuilder::with_chunk_size(4);

        // Add source in chunks
        builder.add_source_chunk(b"aaaa");
        builder.add_source_chunk(b"bbbb");
        builder.add_source_chunk(b"cccc");
        builder.add_source_chunk(b"dddd");
        builder.finalize_source();

        // Add target in chunks
        builder.add_target_chunk(b"aaaa");
        builder.add_target_chunk(b"bbbb");
        builder.add_target_chunk(b"cccc");
        builder.add_target_chunk(b"dddd");

        let patch_data = builder.finalize().unwrap();

        // Verify roundtrip
        let mut applier = PatchApplier::new();
        applier.add_source_chunk(source);
        applier.set_patch(&patch_data);
        applier.prepare().unwrap();

        let mut result = Vec::new();
        while applier.has_more_output() {
            result.extend(applier.next_output_chunk(100));
        }

        assert_eq!(result, target);
    }

    #[test]
    fn test_auto_finalize_source() {
        // If finalize_source() is not called, add_target_chunk should auto-finalize
        let source = b"test source";
        let target = b"test target";

        let mut builder = PatchBuilder::new();
        builder.add_source_chunk(source);
        // Skipping finalize_source() intentionally
        builder.add_target_chunk(target);

        let patch_data = builder.finalize().unwrap();
        assert!(!patch_data.is_empty());
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
    fn test_progress() {
        let mut builder = PatchBuilder::new();

        assert_eq!(builder.progress(100, 100), 0.0);

        builder.add_source_chunk(&[0u8; 50]);
        assert_eq!(builder.progress(100, 100), 25.0);

        builder.add_source_chunk(&[0u8; 50]);
        builder.finalize_source();
        assert_eq!(builder.progress(100, 100), 50.0);

        builder.add_target_chunk(&[0u8; 100]);
        assert_eq!(builder.progress(100, 100), 100.0);
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
