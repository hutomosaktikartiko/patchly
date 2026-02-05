//! Patchly WASM - Binary diff & patch engine for WebAssembly
//!
//! Provides streaming APIs for memory-efficient processing of large files.

pub mod diff;
pub mod format;
pub mod utils;

use wasm_bindgen::prelude::*;

use crate::diff::patch::{apply_patch, generate_patch_with_chunk_size};
use crate::format::patch_format::{calculate_hash, HashBuilder, Patch};
use crate::utils::buffer::ChunkBuffer;

/// Default chunk size for diff matching (4KB)
const DEFAULT_CHUNK_SIZE: usize = 4 * 1024;

/// Default output chunk size for streaming (1MB)
const DEFAULT_OUTPUT_CHUNK_SIZE: usize = 1024 * 1024;

/// Builder for creating patches from streamed file chunks.
#[wasm_bindgen]
pub struct PatchBuilder {
    source_buffer: ChunkBuffer,
    target_buffer: ChunkBuffer,
    source_hasher: HashBuilder,
    target_hasher: HashBuilder,
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
            source_buffer: ChunkBuffer::new(),
            target_buffer: ChunkBuffer::new(),
            source_hasher: HashBuilder::new(),
            target_hasher: HashBuilder::new(),
            chunk_size,
        }
    }

    /// Add a chunk of source (old file) data.
    #[wasm_bindgen]
    pub fn add_source_chunk(&mut self, chunk: &[u8]) {
        self.source_hasher.update(chunk);
        self.source_buffer.push_slice(chunk);
    }

    /// Add a chunk of target (new file) data.
    #[wasm_bindgen]
    pub fn add_target_chunk(&mut self, chunk: &[u8]) {
        self.target_hasher.update(chunk);
        self.target_buffer.push_slice(chunk);
    }

    /// Get current source size (bytes received so far).
    #[wasm_bindgen]
    pub fn source_size(&self) -> usize {
        self.source_buffer.total_size()
    }

    /// Get current target size (bytes received so far).
    #[wasm_bindgen]
    pub fn target_size(&self) -> usize {
        self.target_buffer.total_size()
    }

    /// Get progress as percentage (0-100) based on expected sizes.
    /// Returns source progress if target_expected is 0.
    #[wasm_bindgen]
    pub fn progress(&self, source_expected: usize, target_expected: usize) -> f64 {
        let total_expected = source_expected + target_expected;
        if total_expected == 0 {
            return 100.0;
        }
        let total_received = self.source_buffer.total_size() + self.target_buffer.total_size();
        (total_received as f64 / total_expected as f64 * 100.0).min(100.0)
    }

    /// Finalize and generate the patch.
    /// returns serialized patch data.
    #[wasm_bindgen]
    pub fn finalize(&self) -> Result<Vec<u8>, JsError> {
        let source = self.source_buffer.merge();
        let target = self.target_buffer.merge();

        let patch = generate_patch_with_chunk_size(&source, &target, self.chunk_size);

        patch
            .serialize()
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }

    /// Get patch info without without full serialization (for perview).
    /// Returns JSON string with stats
    #[wasm_bindgen]
    pub fn get_preview_info(&self) -> String {
        let source = self.source_buffer.merge();
        let target = self.target_buffer.merge();

        let patch = generate_patch_with_chunk_size(&source, &target, self.chunk_size);
        let stats = patch.stats();

        format!(
            r#"{{"sourceSize":{},"targetSize":{},"chunkSize":{},"copyCount":{},"copyBytes":{},"insertCount":{},"insertBytes":{},"estimatedPatchSize":{}}}"#,
            patch.source_size,
            patch.target_size,
            patch.chunk_size,
            stats.copy_count,
            stats.copy_bytes,
            stats.insert_count,
            stats.insert_bytes,
            stats.insert_bytes + (stats.copy_count * 13) as u64 + 33 // Rough estimate
        )
    }

    /// Check if source and target files are indentical.
    /// Files are identical if both size AND hash match
    #[wasm_bindgen]
    pub fn are_files_identical(&self) -> bool {
        let same_size = self.source_buffer.total_size() == self.target_buffer.total_size();
        let same_hash = self.source_hasher.finalize() == self.target_hasher.finalize();

        same_size && same_hash
    }

    /// Reset the builder for reuse.
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.source_buffer.clear();
        self.target_buffer.clear();
        self.source_hasher = HashBuilder::new();
        self.target_hasher = HashBuilder::new();
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

    /// Get progress as percentage (0-100)
    #[wasm_bindgen]
    pub fn progress(&self, source_expected: usize) -> f64 {
        if source_expected == 0 {
            return if self.patch_loaded { 100.0 } else { 0.0 };
        }
        (self.source_buffer.total_size() as f64 / source_expected as f64 * 100.0).min(100.0)
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
    fn test_identical_files_detection() {
        let data = b"identical content here";

        let mut builder = PatchBuilder::new();
        builder.add_source_chunk(data);
        builder.add_target_chunk(data);

        assert!(builder.are_files_identical());
    }

    #[test]
    fn test_different_files_detection() {
        let source = b"original content";
        let target = b"modified content";

        let mut builder = PatchBuilder::new();
        builder.add_source_chunk(source);
        builder.add_target_chunk(target);

        assert!(!builder.are_files_identical());
    }

    #[test]
    fn test_same_size_different_content() {
        let source = b"aaaa";
        let target = b"bbbb";

        let mut builder = PatchBuilder::new();
        builder.add_source_chunk(source);
        builder.add_target_chunk(target);

        // Same size but different content = not identical
        assert!(!builder.are_files_identical());
    }

    #[test]
    fn test_patch_applier_streaming_output() {
        let source = b"original file content here!!!!";
        let target = b"modified file content here!!!!";

        // Create patch
        let mut builder = PatchBuilder::new();
        builder.add_source_chunk(source);
        builder.add_target_chunk(target);
        let patch_data = builder.finalize().unwrap();

        // Apply with streaming output
        let mut applier = PatchApplier::new();
        applier.add_source_chunk(source);
        applier.set_patch(&patch_data);
        applier.prepare().unwrap();

        assert!(applier.has_more_output());
        assert_eq!(applier.total_output_size(), target.len());

        // Read in small chunks
        let mut result = Vec::new();
        while applier.has_more_output() {
            let chunk = applier.next_output_chunk(10);
            result.extend_from_slice(&chunk);
        }

        assert_eq!(result, target);
        assert!(!applier.has_more_output());
    }

    #[test]
    fn test_patch_builder_and_applier_roundtrip() {
        let source = b"correct source file!";
        let target = b"target file content!";

        // Create patch
        let mut builder = PatchBuilder::new();
        builder.add_source_chunk(source);
        builder.add_target_chunk(target);
        let patch_data = builder.finalize().unwrap();

        // Apply with correct source
        let mut applier = PatchApplier::new();
        applier.add_source_chunk(source);
        applier.set_patch(&patch_data);
        applier.prepare().unwrap();

        let mut result = Vec::new();
        while applier.has_more_output() {
            result.extend(applier.next_output_chunk(10));
        }

        assert_eq!(result, target);
    }

    #[test]
    fn test_progress_tracking() {
        let mut builder = PatchBuilder::new();

        // Initially 0%
        assert_eq!(builder.progress(100, 100), 0.0);

        // Add half of source
        builder.add_source_chunk(&[0u8; 50]);
        assert_eq!(builder.progress(100, 100), 25.0);

        // Add all of source
        builder.add_source_chunk(&[0u8; 50]);
        assert_eq!(builder.progress(100, 100), 50.0);

        // Add all of target
        builder.add_target_chunk(&[0u8; 100]);
        assert_eq!(builder.progress(100, 100), 100.0);
    }

    #[test]
    fn test_expected_sizes() {
        let source = b"source data here";
        let target = b"target data here!!";

        let mut builder = PatchBuilder::new();
        builder.add_source_chunk(source);
        builder.add_target_chunk(target);
        let patch_data = builder.finalize().unwrap();

        let mut applier = PatchApplier::new();
        applier.set_patch(&patch_data);

        assert_eq!(applier.expected_source_size().unwrap(), source.len() as u64);
        assert_eq!(applier.expected_target_size().unwrap(), target.len() as u64);
    }

    #[test]
    fn test_get_patch_info() {
        let source = b"aaaabbbbcccc";
        let target = b"aaaaNEWWcccc";

        let mut builder = PatchBuilder::new();
        builder.add_source_chunk(source);
        builder.add_target_chunk(target);
        let patch_data = builder.finalize().unwrap();

        let mut applier = PatchApplier::new();
        applier.set_patch(&patch_data);

        let info = applier.get_patch_info().unwrap();
        assert!(info.contains("sourceSize"));
        assert!(info.contains("targetSize"));
        assert!(info.contains("copyCount"));
        assert!(info.contains("sourceHash"));
    }

    #[test]
    fn test_builder_reset() {
        let mut builder = PatchBuilder::new();
        builder.add_source_chunk(b"test data");
        builder.add_target_chunk(b"test data");

        assert!(builder.source_size() > 0);

        builder.reset();

        assert_eq!(builder.source_size(), 0);
        assert_eq!(builder.target_size(), 0);
    }

    #[test]
    fn test_applier_reset() {
        let mut applier = PatchApplier::new();
        applier.add_source_chunk(b"test");
        applier.set_patch(b"PTCH\x02...");

        assert!(applier.source_size() > 0);
        assert!(applier.is_patch_loaded());

        applier.reset();

        assert_eq!(applier.source_size(), 0);
        assert!(!applier.is_patch_loaded());
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

        assert_eq!(hash.len(), 16); // 64-bit = 16 hex chars
        assert_eq!(hash, hash_data(data)); // Consistent
    }

    #[test]
    fn test_output_progress() {
        let source = b"source";
        let target = b"target content that is longer";

        let mut builder = PatchBuilder::new();
        builder.add_source_chunk(source);
        builder.add_target_chunk(target);
        let patch_data = builder.finalize().unwrap();

        let mut applier = PatchApplier::new();
        applier.add_source_chunk(source);
        applier.set_patch(&patch_data);
        applier.prepare().unwrap();

        assert_eq!(applier.output_progress(), 0.0);

        // Read half
        let half = applier.total_output_size() / 2;
        applier.next_output_chunk(half);

        assert!(applier.output_progress() > 40.0);
        assert!(applier.output_progress() < 60.0);

        // Read rest
        while applier.has_more_output() {
            applier.next_output_chunk(100);
        }

        assert_eq!(applier.output_progress(), 100.0);
    }
}
