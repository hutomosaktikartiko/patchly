//! Binary patch file format utilities.
//!
//! Provides constants, hashing utilities, and header parsing for the PTCH format.
//!
//! ## Format Structure
//!
//! Header (33 bytes):
//!   - Magic: "PTCH" (4 bytes)
//!   - Version: u8 (1 byte)
//!   - Chunk size: u32 LE (4 bytes)
//!   - Source size: u64 LE (8 bytes)
//!   - Source hash: u64 LE (8 bytes)
//!   - Target size: u64 LE (8 bytes)
//!
//! Instructions (variable):
//!   - COPY: 0x01 + offset(u64 LE) + length(u32 LE)
//!   - INSERT: 0x02 + length(u32 LE) + data

use std::io::{self, Read, Write};

/// Magic bytes to identify patch files.
pub const MAGIC: &[u8; 4] = b"PTCH";

/// Current format version.
pub const VERSION: u8 = 1;

/// Header size in bytes.
pub const HEADER_SIZE: usize = 33;

/// Instruction type marker for COPY.
pub const TYPE_COPY: u8 = 0x01;

/// Instruction type marker for INSERT.
pub const TYPE_INSERT: u8 = 0x02;

/// FNV-1a hash offset basis.
const FNV_OFFSET: u64 = 0xcbf29ce484222325;

/// FNV-1a hash prime.
const FNV_PRIME: u64 = 0x100000001b3;

/// Validation error when source file doesn't match patch requirements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Source file size doesn't match expected size.
    SizeMismatch { expected: u64, actual: u64 },
    /// Source file hash doesn't match expected hash.
    HashMismatch { expected: u64, actual: u64 },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::SizeMismatch { expected, actual } => {
                write!(
                    f,
                    "Source size mismatch: expected {} bytes, got {} bytes",
                    expected, actual
                )
            }
            ValidationError::HashMismatch { expected, actual } => {
                write!(
                    f,
                    "Source hash mismatch: expected {:016x}, got {:016x}",
                    expected, actual
                )
            }
        }
    }
}

/// Calculates a 64-bit FNV-1a hash of data.
pub fn calculate_hash(data: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Incremental hash builder for streaming data.
///
/// Use this when data arrives in chunks and you need to compute
/// a running hash without buffering the entire content.
pub struct HashBuilder {
    /// Current hash state.
    hash: u64,
}

impl HashBuilder {
    /// Creates a new hash builder.
    pub fn new() -> Self {
        Self { hash: FNV_OFFSET }
    }

    /// Updates the hash with additional data.
    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            self.hash ^= byte as u64;
            self.hash = self.hash.wrapping_mul(FNV_PRIME)
        }
    }

    /// Finalizes and returns the hash value.
    pub fn finalize(&self) -> u64 {
        self.hash
    }
}

impl Default for HashBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializes a patch header to bytes.
///
/// # Returns
///
/// A 33-byte header.
pub fn serialize_header(
    chunk_size: u32,
    source_size: u64,
    source_hash: u64,
    target_size: u64,
) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::with_capacity(HEADER_SIZE);

    buffer.write_all(MAGIC)?;
    buffer.write_all(&[VERSION])?;
    buffer.write_all(&chunk_size.to_le_bytes())?;
    buffer.write_all(&source_size.to_le_bytes())?;
    buffer.write_all(&source_hash.to_le_bytes())?;
    buffer.write_all(&target_size.to_le_bytes())?;

    Ok(buffer)
}

/// Parsed patch header information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchHeader {
    /// Chunk size used during diff generation.
    pub chunk_size: u32,
    /// Size of the original source file.
    pub source_size: u64,
    /// Hash of the original source file.
    pub source_hash: u64,
    /// Size of the target file after patching.
    pub target_size: u64,
}

impl PatchHeader {
    /// Parses a header from bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - At least 33 bytes of header data.
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        if data.len() < HEADER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Header too small: {} bytes (need {})",
                    data.len(),
                    HEADER_SIZE
                ),
            ));
        }

        let mut cursor = io::Cursor::new(data);

        // Validate magic bytes
        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid patch file: bad magic bytes",
            ));
        }

        // Validate version
        let mut version = [0u8; 1];
        cursor.read_exact(&mut version)?;
        if version[0] != VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported patch version: {} (expected {})",
                    version[0], VERSION
                ),
            ));
        }

        // Read chunk_size
        let mut chunk_size_bytes = [0u8; 4];
        cursor.read_exact(&mut chunk_size_bytes)?;
        let chunk_size = u32::from_le_bytes(chunk_size_bytes);

        // Read source_size
        let mut source_size_bytes = [0u8; 8];
        cursor.read_exact(&mut source_size_bytes)?;
        let source_size = u64::from_le_bytes(source_size_bytes);

        // Read source_hash
        let mut source_hash_bytes = [0u8; 8];
        cursor.read_exact(&mut source_hash_bytes)?;
        let source_hash = u64::from_le_bytes(source_hash_bytes);

        // Read target_size
        let mut target_size_bytes = [0u8; 8];
        cursor.read_exact(&mut target_size_bytes)?;
        let target_size = u64::from_le_bytes(target_size_bytes);

        Ok(Self {
            chunk_size,
            source_size,
            source_hash,
            target_size,
        })
    }

    /// Validates that a source file matches this header's requirements.
    pub fn validate_source(
        &self,
        source_size: u64,
        source_hash: u64,
    ) -> Result<(), ValidationError> {
        if source_size != self.source_size {
            return Err(ValidationError::SizeMismatch {
                expected: self.source_size,
                actual: source_size,
            });
        }
        if source_hash != self.source_hash {
            return Err(ValidationError::HashMismatch {
                expected: self.source_hash,
                actual: source_hash,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash() {
        let data = b"hello world";
        let hash = calculate_hash(data);

        // Same data should produce same hash
        assert_eq!(hash, calculate_hash(data));

        // Different data should produce different hash
        assert_ne!(hash, calculate_hash(b"hello world!"));
    }

    #[test]
    fn test_hash_builder() {
        let data = b"hello world";

        // Full hash
        let full_hash = calculate_hash(data);

        // Incremental hash
        let mut builder = HashBuilder::new();
        builder.update(b"hello");
        builder.update(b" ");
        builder.update(b"world");
        let incremental_hash = builder.finalize();

        assert_eq!(full_hash, incremental_hash);
    }

    #[test]
    fn test_hash_builder_default() {
        let builder = HashBuilder::default();
        assert_eq!(builder.finalize(), FNV_OFFSET);
    }

    #[test]
    fn test_serialize_header() {
        let header = serialize_header(4096, 1000, 0xABCD, 2000).unwrap();

        assert_eq!(header.len(), HEADER_SIZE);
        assert_eq!(&header[0..4], MAGIC);
        assert_eq!(header[4], VERSION);
    }

    #[test]
    fn test_header_roundtrip() {
        let original = serialize_header(4096, 12345, 0xDEADBEEF, 67890).unwrap();
        let parsed = PatchHeader::parse(&original).unwrap();

        assert_eq!(parsed.chunk_size, 4096);
        assert_eq!(parsed.source_size, 12345);
        assert_eq!(parsed.source_hash, 0xDEADBEEF);
        assert_eq!(parsed.target_size, 67890);
    }

    #[test]
    fn test_header_invalid_magic() {
        let bad_data = b"BADM\x01\x00\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let result = PatchHeader::parse(bad_data);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("magic"));
    }

    #[test]
    fn test_header_invalid_version() {
        let bad_data = b"PTCH\x99\x00\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let result = PatchHeader::parse(bad_data);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("version"));
    }

    #[test]
    fn test_header_too_small() {
        let result = PatchHeader::parse(&[0u8; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_source_success() {
        let header = PatchHeader {
            chunk_size: 4096,
            source_size: 100,
            source_hash: 0xABCD,
            target_size: 200,
        };
        assert!(header.validate_source(100, 0xABCD).is_ok());
    }

    #[test]
    fn test_validate_source_size_mismatch() {
        let header = PatchHeader {
            chunk_size: 4096,
            source_size: 100,
            source_hash: 0xABCD,
            target_size: 200,
        };
        let result = header.validate_source(50, 0xABCD);

        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::SizeMismatch { expected, actual } => {
                assert_eq!(expected, 100);
                assert_eq!(actual, 50);
            }
            _ => panic!("Expected SizeMismatch"),
        }
    }

    #[test]
    fn test_validate_source_hash_mismatch() {
        let header = PatchHeader {
            chunk_size: 4096,
            source_size: 100,
            source_hash: 0xABCD,
            target_size: 200,
        };
        let result = header.validate_source(100, 0x1234);

        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::HashMismatch { expected, actual } => {
                assert_eq!(expected, 0xABCD);
                assert_eq!(actual, 0x1234);
            }
            _ => panic!("Expected HashMismatch"),
        }
    }

    #[test]
    fn test_validation_error_display() {
        let size_err = ValidationError::SizeMismatch {
            expected: 100,
            actual: 50,
        };
        assert!(size_err.to_string().contains("100"));
        assert!(size_err.to_string().contains("50"));

        let hash_err = ValidationError::HashMismatch {
            expected: 0xABCD,
            actual: 0x1234,
        };
        assert!(hash_err.to_string().contains("abcd"));
        assert!(hash_err.to_string().contains("1234"));
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAGIC, b"PTCH");
        assert_eq!(VERSION, 1);
        assert_eq!(HEADER_SIZE, 33);
        assert_eq!(TYPE_COPY, 0x01);
        assert_eq!(TYPE_INSERT, 0x02);
    }
}
