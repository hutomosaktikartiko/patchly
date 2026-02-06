//! Binary patch file format (serialize/deserialize).
//!
//! Format structure:
//! - Header: magic(4) + version(1) + chunk_size(4) + source_size(8) + source_hash(8) + target_size(8)
//! - Instructions: sequence of COPY or INSERT operations

use std::io::{self, Read, Write};

/// Magic bytes to identify patch files: "PTCH"
const MAGIC: &[u8; 4] = b"PTCH";

/// Current format version
const VERSION: u8 = 1;

/// Instructions type markers
const TYPE_COPY: u8 = 0x01;
const TYPE_INSERT: u8 = 0x02;

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

/// A single patch instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// Copy bytes from source file.
    Copy { offset: u64, length: u32 },

    /// Insert new bytes (not present in source)
    Insert { data: Vec<u8> },
}

/// Parsed instruction that references data by offset (zero-copy).
/// Used by PatchApplier to avoid copying INSERT data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedInstruction {
    /// Copy bytes from source file
    Copy { offset: u64, length: u32 },

    /// Insert new bytes - references position in original patch data.
    Insert { patch_offset: usize, length: u32 },
}

/// Patch metadat without instruction data (for zero-copy parsing).
#[derive(Debug, Clone)]
pub struct PatchMetadata {
    pub chunk_size: u32,
    pub source_size: u64,
    pub source_hash: u64,
    pub target_size: u64,
    pub instructions: Vec<ParsedInstruction>,
}

impl PatchMetadata {
    /// Parse patch data without copying INSERT data.
    /// Returns metadata with instructions that reference the original data.
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        let mut cursor = io::Cursor::new(data);

        // Read and verify magic
        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid patch file: bad magic bytes",
            ));
        }

        // Read version
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

        // Parse instructions WITHOUT copying INSERT data
        let mut instructions = Vec::new();
        loop {
            let mut type_byte = [0u8; 1];
            match cursor.read_exact(&mut type_byte) {
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }

            match type_byte[0] {
                TYPE_COPY => {
                    let mut offset_bytes = [0u8; 8];
                    let mut length_bytes = [0u8; 4];
                    cursor.read_exact(&mut offset_bytes)?;
                    cursor.read_exact(&mut length_bytes)?;

                    instructions.push(ParsedInstruction::Copy {
                        offset: u64::from_le_bytes(offset_bytes),
                        length: u32::from_le_bytes(length_bytes),
                    });
                }
                TYPE_INSERT => {
                    let mut length_bytes = [0u8; 4];
                    cursor.read_exact(&mut length_bytes)?;
                    let length = u32::from_le_bytes(length_bytes);

                    // Store offset instead of copying data!
                    let patch_offset = cursor.position() as usize;

                    // Skip over the data (don't read it)
                    cursor.set_position(cursor.position() + length as u64);

                    instructions.push(ParsedInstruction::Insert {
                        patch_offset,
                        length,
                    });
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Unknown instruction type: {}", type_byte[0]),
                    ));
                }
            }
        }

        Ok(Self {
            chunk_size,
            source_size,
            source_hash,
            target_size,
            instructions,
        })
    }

    /// Validate source file
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

/// Complete patch containing all instructions to transform source -> target.
#[derive(Debug, Clone)]
pub struct Patch {
    /// Chunk size used during diff generation
    pub chunk_size: u32,
    /// Size of source file (for validation)
    pub source_size: u64,
    /// Hash of source file (for validation)
    pub source_hash: u64,
    /// Expected size of target file after applying patch
    pub target_size: u64,
    /// List of instructions in order
    pub instructions: Vec<Instruction>,
}

impl Patch {
    /// Create a new empty patch.
    pub fn new(chunk_size: u32, source_size: u64, source_hash: u64, target_size: u64) -> Self {
        Self {
            chunk_size,
            source_size,
            source_hash,
            target_size,
            instructions: Vec::new(),
        }
    }

    /// Add a COPY instruction.
    pub fn add_copy(&mut self, offset: u64, length: u32) {
        self.instructions.push(Instruction::Copy { offset, length });
    }

    /// Add an INSERT instruction.
    pub fn add_insert(&mut self, data: Vec<u8>) {
        if !data.is_empty() {
            self.instructions.push(Instruction::Insert { data });
        }
    }

    /// Serialize patch to binary format.
    pub fn serialize(&self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Write header
        buffer.write_all(MAGIC)?;
        buffer.write_all(&[VERSION])?;
        buffer.write_all(&self.chunk_size.to_le_bytes())?;
        buffer.write_all(&self.source_size.to_le_bytes())?;
        buffer.write_all(&self.source_hash.to_le_bytes())?;
        buffer.write_all(&self.target_size.to_le_bytes())?;

        // Write instructions
        for instruction in &self.instructions {
            match instruction {
                Instruction::Copy { offset, length } => {
                    buffer.write_all(&[TYPE_COPY])?;
                    buffer.write_all(&offset.to_le_bytes())?;
                    buffer.write_all(&length.to_le_bytes())?;
                }
                Instruction::Insert { data } => {
                    buffer.write_all(&[TYPE_INSERT])?;
                    buffer.write_all(&(data.len() as u32).to_le_bytes())?;
                    buffer.write_all(data)?;
                }
            }
        }

        Ok(buffer)
    }

    /// Deserialize patch from binary format.
    pub fn deserialize(data: &[u8]) -> io::Result<Self> {
        let mut cursor = io::Cursor::new(data);

        // Read and verify magic
        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid patch file: bad magic bytes",
            ));
        }

        // Read version
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

        // Read instructions
        let mut instructions = Vec::new();
        loop {
            let mut type_byte = [0u8; 1];
            match cursor.read_exact(&mut type_byte) {
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }

            match type_byte[0] {
                TYPE_COPY => {
                    let mut offset_bytes = [0u8; 8];
                    let mut length_bytes = [0u8; 4];
                    cursor.read_exact(&mut offset_bytes)?;
                    cursor.read_exact(&mut length_bytes)?;

                    instructions.push(Instruction::Copy {
                        offset: u64::from_le_bytes(offset_bytes),
                        length: u32::from_le_bytes(length_bytes),
                    });
                }
                TYPE_INSERT => {
                    let mut length_bytes = [0u8; 4];
                    cursor.read_exact(&mut length_bytes)?;
                    let length = u32::from_le_bytes(length_bytes) as usize;

                    let mut data = vec![0u8; length];
                    cursor.read_exact(&mut data)?;

                    instructions.push(Instruction::Insert { data });
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Unknown instruction type: {}", type_byte[0]),
                    ));
                }
            }
        }

        Ok(Self {
            chunk_size,
            source_size,
            source_hash,
            target_size,
            instructions,
        })
    }

    /// Get total number of instructions.
    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }

    /// Calculate statistics about the patch.
    pub fn stats(&self) -> PatchStats {
        let mut copy_count = 0;
        let mut copy_bytes = 0u64;
        let mut insert_count = 0;
        let mut insert_bytes = 0u64;

        for inst in &self.instructions {
            match inst {
                Instruction::Copy { length, .. } => {
                    copy_count += 1;
                    copy_bytes += *length as u64;
                }
                Instruction::Insert { data } => {
                    insert_count += 1;
                    insert_bytes += data.len() as u64;
                }
            }
        }

        PatchStats {
            copy_count,
            copy_bytes,
            insert_count,
            insert_bytes,
        }
    }

    /// Validate that source file matches expected hash and size.
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

/// Statistics about a patch
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchStats {
    pub copy_count: usize,
    pub copy_bytes: u64,
    pub insert_count: usize,
    pub insert_bytes: u64,
}

/// Validation errors when source file doesn't match patch requirements
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    SizeMismatch { expected: u64, actual: u64 },
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

/// Calculate a simple 64-bit hash of data (FNV-1a variant)
/// Used for secure file verification.
pub fn calculate_hash(data: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Calculate hash incrementally from chunks.
pub struct HashBuilder {
    hash: u64,
}

impl HashBuilder {
    pub fn new() -> Self {
        Self { hash: FNV_OFFSET }
    }

    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            self.hash ^= byte as u64;
            self.hash = self.hash.wrapping_mul(FNV_PRIME)
        }
    }

    pub fn finalize(&self) -> u64 {
        self.hash
    }
}

impl Default for HashBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_patch() {
        let patch = Patch::new(4096, 1000, 12345, 2000);
        assert_eq!(patch.chunk_size, 4096);
        assert_eq!(patch.source_size, 1000);
        assert_eq!(patch.source_hash, 12345);
        assert_eq!(patch.target_size, 2000);
        assert!(patch.instructions.is_empty());
    }

    #[test]
    fn test_add_copy() {
        let mut patch = Patch::new(4096, 100, 0, 100);
        patch.add_copy(100, 50);

        assert_eq!(patch.instructions.len(), 1);
        assert_eq!(
            patch.instructions[0],
            Instruction::Copy {
                offset: 100,
                length: 50
            }
        );
    }

    #[test]
    fn test_add_insert() {
        let mut patch = Patch::new(4096, 100, 0, 100);
        patch.add_insert(vec![1, 2, 3, 4]);

        assert_eq!(patch.instructions.len(), 1);
        assert_eq!(
            patch.instructions[0],
            Instruction::Insert {
                data: vec![1, 2, 3, 4]
            }
        );
    }

    #[test]
    fn test_add_empty_insert_ignored() {
        let mut patch = Patch::new(4096, 100, 0, 100);
        patch.add_insert(vec![]);

        assert!(patch.instructions.is_empty());
    }

    #[test]
    fn test_serialize_deserialize_empty() {
        let patch = Patch::new(4096, 12345, 0xABCD, 67890);
        let bytes = patch.serialize().unwrap();
        let restored = Patch::deserialize(&bytes).unwrap();

        assert_eq!(restored.chunk_size, 4096);
        assert_eq!(restored.source_size, 12345);
        assert_eq!(restored.source_hash, 0xABCD);
        assert_eq!(restored.target_size, 67890);
        assert!(restored.instructions.is_empty());
    }

    #[test]
    fn test_serialize_deserialize_with_instructions() {
        let mut patch = Patch::new(1024, 500, 0x1234, 5000);
        patch.add_copy(0, 100);
        patch.add_insert(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        patch.add_copy(200, 300);
        patch.add_insert(vec![1, 2, 3]);

        let bytes = patch.serialize().unwrap();
        let restored = Patch::deserialize(&bytes).unwrap();

        assert_eq!(restored.chunk_size, 1024);
        assert_eq!(restored.source_size, 500);
        assert_eq!(restored.source_hash, 0x1234);
        assert_eq!(restored.target_size, 5000);
        assert_eq!(restored.instructions.len(), 4);
        assert_eq!(restored.instructions, patch.instructions);
    }

    #[test]
    fn test_invalid_magic() {
        let bad_data = b"BADM\x02\x00\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let result = Patch::deserialize(bad_data);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("magic"));
    }

    #[test]
    fn test_invalid_version() {
        let bad_data = b"PTCH\x99\x00\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let result = Patch::deserialize(bad_data);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("version"));
    }

    #[test]
    fn test_stats() {
        let mut patch = Patch::new(1024, 500, 0, 5000);
        patch.add_copy(0, 100);
        patch.add_copy(200, 150);
        patch.add_insert(vec![1, 2, 3, 4, 5]);
        patch.add_insert(vec![6, 7, 8]);

        let stats = patch.stats();
        assert_eq!(stats.copy_count, 2);
        assert_eq!(stats.copy_bytes, 250);
        assert_eq!(stats.insert_count, 2);
        assert_eq!(stats.insert_bytes, 8);
    }

    #[test]
    fn test_header_size() {
        // Header should be: magic(4) + version(1) + chunk_size(4) + source_size(8) + source_hash(8) + target_size(8) = 33 bytes
        let patch = Patch::new(4096, 1000, 0, 1000);
        let bytes = patch.serialize().unwrap();
        assert_eq!(bytes.len(), 33);
    }

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
    fn test_validate_source_success() {
        let patch = Patch::new(4096, 100, 0xABCD, 200);
        assert!(patch.validate_source(100, 0xABCD).is_ok());
    }

    #[test]
    fn test_validate_source_size_mismatch() {
        let patch = Patch::new(4096, 100, 0xABCD, 200);
        let result = patch.validate_source(50, 0xABCD);

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
        let patch = Patch::new(4096, 100, 0xABCD, 200);
        let result = patch.validate_source(100, 0x1234);

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
    fn test_binary_data_in_insert() {
        let mut patch = Patch::new(1024, 256, 0, 256);
        // Full range of byte values
        let data: Vec<u8> = (0u8..=255).collect();
        patch.add_insert(data.clone());

        let bytes = patch.serialize().unwrap();
        let restored = Patch::deserialize(&bytes).unwrap();

        match &restored.instructions[0] {
            Instruction::Insert {
                data: restored_data,
            } => {
                assert_eq!(restored_data, &data);
            }
            _ => panic!("Expected INSERT instruction"),
        }
    }
}
