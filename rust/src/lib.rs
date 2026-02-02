//! Patchly WASM - Binary diff & patch engine for WebAssembly
//!
//! Provides streaming APIs for memory-efficient processing of large files.

pub mod diff;
pub mod format;
pub mod utils;

use wasm_bindgen::prelude::*;

use crate::diff::patch::{apply_patch, generate_patch_with_chunk_size, PatchError};
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
    pub fn add_target_chunkk(&mut self, chunk: &[u8]) {
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
