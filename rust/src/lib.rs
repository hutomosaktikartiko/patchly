pub mod diff;
pub mod format;
pub mod utils;

use wasm_bindgen::prelude::*;

use crate::diff::patch::{apply_patch, generate_patch};
use crate::format::patch_format::Patch;

/// Create a binary patch from source and target files.
///
/// # Arguments
/// * `source` - Original file bytes
/// * `target` - New file bytes
///
/// # Returns
/// Serialized patch data as bytes
#[wasm_bindgen]
pub fn create_patch(source: &[u8], target: &[u8]) -> Result<Vec<u8>, JsError> {
    let patch = generate_patch(source, target);
    patch
        .serialize()
        .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

/// Create a patch with custom chunk size.
///
/// # Arguments
/// * `source` - Original file bytes
/// * `target` - New file bytes
/// * `chunk_size` - Size of chunks for matching (default 4096)
#[wasm_bindgen]
pub fn create_patch_with_options(
    source: &[u8],
    target: &[u8],
    chunk_size: usize,
) -> Result<Vec<u8>, JsError> {
    let patch = diff::patch::generate_patch_with_chunk_size(source, target, chunk_size);
    patch
        .serialize()
        .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}

/// Apply a patch to source file to reconstruct target file.
///
/// # Arguments
/// * `source` - Original file bytes
/// * `patch_data` - Serialize patch bytes
///
/// # Returns
/// Reconstructured target file bytes
#[wasm_bindgen]
pub fn apply_patch_wasm(source: &[u8], patch_data: &[u8]) -> Result<Vec<u8>, JsError> {
    // Deserialize patch
    let patch = Patch::deserialize(patch_data)
        .map_err(|e| JsError::new(&format!("Invalid patch data: {}", e)))?;

    // Apply patch
    apply_patch(source, &patch).map_err(|e| JsError::new(&format!("Patch apply error: {}", e)))
}

/// Get information about a patch without applying it.
///
/// returns a JSON string with patch statistics.
#[wasm_bindgen]
pub fn get_patch_info(patch_data: &[u8]) -> Result<String, JsError> {
    let patch = Patch::deserialize(patch_data)
        .map_err(|e| JsError::new(&format!("Invalid patch data: {}", e)))?;

    let stats = patch.stats();

    // Simple JSON output (no serde dependency needed)
    Ok(format!(
        r#"{{"chunkSize":{},"targetSize":{},"copyCount":{},"copyBytes":{},"insertCount":{},"insertBytes":{},"totalInstructions":{}}}"#,
        patch.chunk_size,
        patch.target_size,
        stats.copy_count,
        stats.copy_bytes,
        stats.insert_count,
        stats.insert_bytes,
        patch.instruction_count()
    ))
}

/// Get the librarti version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_apply_patch() {
        let source = b"hello world, this is original";
        let target = b"hello rust!, this is modified";

        // Create patch
        let patch_data = create_patch(source, target).unwrap();

        // Apply patch
        let result = apply_patch_wasm(source, &patch_data).unwrap();

        assert_eq!(result, target);
    }

    #[test]
    fn test_get_patch_info() {
        let source = b"aaaabbbbcccc";
        let target = b"aaaaNEWWcccc";

        let patch_data = create_patch(source, target).unwrap();
        let info = get_patch_info(&patch_data).unwrap();

        // Should be valid JSON-ish string
        assert!(info.contains("chunkSize"));
        assert!(info.contains("targetSize"));
        assert!(info.contains("copyCount"));
    }

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
        assert!(v.contains("0.1"));
    }
}
