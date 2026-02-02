//! Binary patch file format (serialize/deserialize).
//!
//! Format structure:
//! - Header: magic(4) + version(1) + chunk_size(4) + target_size(8)
//! - Instructions: sequence of COPY or INSERT operations

use std::io::{self, Read, Write};

/// Magic bytes to identify patch files: "PTCH"
const MAGIC: &[u8; 4] = b"PTCH";

/// Current format version
const VERSION: u8 = 1;

/// Instructions type markers
const TYPE_COPY: u8 = 0x01;
const TYPE_INSERT: u8 = 0x02;

/// A single patch instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// Copy bytes from source file.
    /// (source_offset, length)
    Copy { offset: u64, length: u32 },

    /// Insert new bytes (not present in source)
    /// Contains the actual data to insert.
    Insert { data: Vec<u8> },
}

/// Complete patch containing all instructions to transform source -> target.
#[derive(Debug, Clone)]
pub struct Patch {
    /// Chunk size used during diff generation
    pub chunk_size: u32,
    /// Expected size of target file after applying patch
    pub target_size: u64,
    /// List of instructions in order
    pub instructions: Vec<Instruction>,
}

impl Patch {
    /// Create a new empty patch.
    pub fn new(chunk_size: u32, target_size: u64) -> Self {
        Self {
            chunk_size,
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
                format!("Unsupported patch version: {}", version[0]),
            ));
        }

        // Read chunk_size
        let mut chunk_size_bytes = [0u8; 4];
        cursor.read_exact(&mut chunk_size_bytes)?;
        let chunk_size = u32::from_le_bytes(chunk_size_bytes);

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

        for instruction in &self.instructions {
            match instruction {
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
}

/// Statistics about a patch
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchStats {
    pub copy_count: usize,
    pub copy_bytes: u64,
    pub insert_count: usize,
    pub insert_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_patch() {
        let patch = Patch::new(4096, 1000);
        assert_eq!(patch.chunk_size, 4096);
        assert_eq!(patch.target_size, 1000);
        assert!(patch.instructions.is_empty());
    }

    #[test]
    fn test_add_copy() {
        let mut patch = Patch::new(4096, 1000);
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
        let mut patch = Patch::new(4096, 1000);
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
        let mut patch = Patch::new(4096, 1000);
        patch.add_insert(vec![]);

        assert!(patch.instructions.is_empty());
    }

    #[test]
    fn test_serialize_deserialize_empty() {
        let patch = Patch::new(4096, 12345);
        let bytes = patch.serialize().unwrap();
        let restored = Patch::deserialize(&bytes).unwrap();

        assert_eq!(restored.chunk_size, 4096);
        assert_eq!(restored.target_size, 12345);
        assert!(restored.instructions.is_empty());
    }

    #[test]
    fn test_serialize_deserialize_with_instructions() {
        let mut patch = Patch::new(1024, 5000);
        patch.add_copy(0, 100);
        patch.add_insert(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        patch.add_copy(200, 300);
        patch.add_insert(vec![1, 2, 3]);

        let bytes = patch.serialize().unwrap();
        let restored = Patch::deserialize(&bytes).unwrap();

        assert_eq!(restored.chunk_size, 1024);
        assert_eq!(restored.target_size, 5000);
        assert_eq!(restored.instructions.len(), 4);
        assert_eq!(restored.instructions[0], patch.instructions[0]);
        assert_eq!(restored.instructions[1], patch.instructions[1]);
        assert_eq!(restored.instructions[2], patch.instructions[2]);
        assert_eq!(restored.instructions[3], patch.instructions[3]);
    }

    #[test]
    fn test_invalid_magic() {
        let bad_data = b"BADM\x01\x00\x00\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let result = Patch::deserialize(bad_data);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("magic"));
    }

    #[test]
    fn test_invalid_version() {
        let bad_data = b"PTCH\x99\x00\x00\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let result = Patch::deserialize(bad_data);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("version"));
    }

    #[test]
    fn test_stats() {
        let mut patch = Patch::new(1024, 5000);
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
        // Header should be: magic(4) + version(1) + chunk_size(4) + target_size(8) = 17 bytes
        let patch = Patch::new(4096, 1000);
        let bytes = patch.serialize().unwrap();
        assert_eq!(bytes.len(), 17);
    }

    #[test]
    fn test_binary_data_in_insert() {
        let mut patch = Patch::new(1024, 256);
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
