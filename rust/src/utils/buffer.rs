//! Buffer utilities for chunked/streaming processing.
//!
//! Provides memory-efficient handling of large data by working
//! with chunks instead of loading everything into a single buffer.

/// Default chunk size for splitting data (1MB)
pub const DEFAULT_BUFFER_CHUNK_SIZE: usize = 1024 * 1024;

/// A buffer that stores data in chunks for memory-efficient processing.
///
/// Instad of storing all data in a single contiguous Vec<u8>
/// ChunkBuffer stores data as a list of chunks. This allows:
/// - Incremental appending without reallocation
/// - Memory-efficient processing of large files
/// - Easy integration with streaming APIs
#[derive(Debug, Clone)]
pub struct ChunkBuffer {
    chunks: Vec<Vec<u8>>,
    total_size: usize,
}

impl ChunkBuffer {
    /// Create a new empty ChunkBuffer.
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            total_size: 0,
        }
    }

    /// Create a ChunkBuffer with pre-allocated capacity for chunks.
    ///
    /// # Arguments
    /// * `chunk_capacity` - Expected number of chunks
    pub fn with_capacity(chunk_capacity: usize) -> Self {
        Self {
            chunks: Vec::with_capacity(chunk_capacity),
            total_size: 0,
        }
    }

    /// Append a chunk of data.
    ///
    /// # Arguments
    /// * `data` - Chunk to append (takes ownership)
    pub fn push(&mut self, data: Vec<u8>) {
        self.total_size += data.len();
        self.chunks.push(data);
    }

    /// Append data by copying from a slice
    ///
    /// # Arguments
    /// * `data` - Data slice to copy and append
    pub fn push_slice(&mut self, data: &[u8]) {
        self.total_size += data.len();
        self.chunks.push(data.to_vec());
    }

    /// Get total size of all data in buffer.
    pub fn total_size(&self) -> usize {
        self.total_size
    }

    /// Get number of chunks
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.total_size == 0
    }

    /// Get a reference to a specific chunk.
    ///
    /// # Arguments
    /// * `index` - Chunk index
    pub fn get_chunk(&self, index: usize) -> Option<&[u8]> {
        self.chunks.get(index).map(|c| c.as_slice())
    }

    /// Iterate over all chunks
    pub fn iter_chunks(&self) -> impl Iterator<Item = &[u8]> {
        self.chunks.iter().map(|c| c.as_slice())
    }

    /// Merge all chunks into a single configuous Vec<u8>.
    ///
    /// Note: This allocates memory for the entire data.
    pub fn merge(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.total_size);
        for chunk in &self.chunks {
            result.extend_from_slice(chunk);
        }
        result
    }

    /// Clear all data from buffer
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.total_size = 0;
    }

    /// Read bytes at a specific offset acress chunks.
    ///
    /// # Arguments
    /// * `offset` - Starting byte offset
    /// * `length` - Number of bytes to read
    ///
    /// # Returns
    /// Vec containing the requested bytes, or None if out of bounds
    pub fn read_at(&self, offset: usize, length: usize) -> Option<Vec<u8>> {
        if offset + length > self.total_size {
            return None;
        }

        let mut result = Vec::with_capacity(length);
        let mut current_offset = 0;
        let mut remaining = length;
        let mut start_offset = offset;

        for chunk in &self.chunks {
            let chunk_len = chunk.len();

            // Skip chunks before out start offset
            if current_offset + chunk_len <= start_offset {
                current_offset += chunk_len;
                continue;
            }

            // Calculate where to start reading in this chunk
            let chunk_start = if start_offset > current_offset {
                start_offset - current_offset
            } else {
                0
            };

            // Calculate how many bytes to read from this chunk
            let bytes_available = chunk_len - chunk_start;
            let bytes_to_read = remaining.min(bytes_available);

            result.extend_from_slice(&chunk[chunk_start..chunk_start + bytes_to_read]);
            remaining -= bytes_to_read;
            start_offset = current_offset + chunk_len;
            current_offset += chunk_len;

            if remaining == 0 {
                break;
            }
        }

        Some(result)
    }
}

impl Default for ChunkBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Split a large data slice into smaller chunks.
///
/// # Arguments
/// * `data` - Data to split
/// * `chunk_size` - Maximum size of each chunk
///
/// # Returns
/// Vector of chunks
pub fn split_into_chunks(data: &[u8], chunk_size: usize) -> Vec<Vec<u8>> {
    if chunk_size == 0 {
        return vec![data.to_vec()];
    }

    data.chunks(chunk_size).map(|c| c.to_vec()).collect()
}

/// Create a ChunkBuffer from a data slice by splitting it.
///
/// # Arguments
/// * `data` - Data to convert
/// * `chunk_size` - Size of each chunk
pub fn chunk_buffer_from_slice(data: &[u8], chunk_size: usize) -> ChunkBuffer {
    let mut buffer = ChunkBuffer::new();
    for chunk in data.chunks(chunk_size) {
        buffer.push_slice(chunk);
    }
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer = ChunkBuffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.total_size(), 0);
        assert_eq!(buffer.chunk_count(), 0);
    }

    #[test]
    fn test_push_and_size() {
        let mut buffer = ChunkBuffer::new();
        buffer.push(vec![1, 2, 3, 4, 5]);
        buffer.push(vec![6, 7, 8]);

        assert_eq!(buffer.total_size(), 8);
        assert_eq!(buffer.chunk_count(), 2);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_push_slice() {
        let mut buffer = ChunkBuffer::new();
        buffer.push_slice(b"hello");
        buffer.push_slice(b" world");

        assert_eq!(buffer.total_size(), 11);
        assert_eq!(buffer.chunk_count(), 2);
    }

    #[test]
    fn test_get_chunk() {
        let mut buffer = ChunkBuffer::new();
        buffer.push(vec![1, 2, 3]);
        buffer.push(vec![4, 5, 6]);

        assert_eq!(buffer.get_chunk(0), Some(&[1u8, 2, 3][..]));
        assert_eq!(buffer.get_chunk(1), Some(&[4u8, 5, 6][..]));
        assert_eq!(buffer.get_chunk(2), None);
    }

    #[test]
    fn test_iter_chunks() {
        let mut buffer = ChunkBuffer::new();
        buffer.push(vec![1, 2]);
        buffer.push(vec![3, 4]);
        buffer.push(vec![5, 6]);

        let chunks: Vec<&[u8]> = buffer.iter_chunks().collect();
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], &[1, 2]);
        assert_eq!(chunks[1], &[3, 4]);
        assert_eq!(chunks[2], &[5, 6]);
    }

    #[test]
    fn test_merge() {
        let mut buffer = ChunkBuffer::new();
        buffer.push(vec![1, 2, 3]);
        buffer.push(vec![4, 5]);
        buffer.push(vec![6, 7, 8, 9]);

        let merged = buffer.merge();
        assert_eq!(merged, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_merge_empty() {
        let buffer = ChunkBuffer::new();
        let merged = buffer.merge();
        assert!(merged.is_empty());
    }

    #[test]
    fn test_read_at_single_chunk() {
        let mut buffer = ChunkBuffer::new();
        buffer.push(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        assert_eq!(buffer.read_at(0, 3), Some(vec![0, 1, 2]));
        assert_eq!(buffer.read_at(3, 4), Some(vec![3, 4, 5, 6]));
        assert_eq!(buffer.read_at(7, 3), Some(vec![7, 8, 9]));
    }

    #[test]
    fn test_read_at_across_chunks() {
        let mut buffer = ChunkBuffer::new();
        buffer.push(vec![0, 1, 2, 3]);
        buffer.push(vec![4, 5, 6, 7]);
        buffer.push(vec![8, 9]);

        // Read across first and second chunk
        assert_eq!(buffer.read_at(2, 4), Some(vec![2, 3, 4, 5]));

        // Read across all there chunks
        assert_eq!(buffer.read_at(3, 5), Some(vec![3, 4, 5, 6, 7]));

        // Read entire buffer
        assert_eq!(
            buffer.read_at(0, 10),
            Some(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
        );
    }

    #[test]
    fn test_read_at_out_bounds() {
        let mut buffer = ChunkBuffer::new();
        buffer.push(vec![1, 2, 3, 4, 5]);

        assert_eq!(buffer.read_at(0, 10), None); // Too long
        assert_eq!(buffer.read_at(5, 1), None); // Start at end
        assert_eq!(buffer.read_at(10, 1), None); // Way past end
    }

    #[test]
    fn test_read_at_empty_buffer() {
        let buffer = ChunkBuffer::new();
        assert_eq!(buffer.read_at(0, 1), None);
    }

    #[test]
    fn test_split_into_chunks() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let chunks = split_into_chunks(&data, 3);

        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0], vec![1, 2, 3]);
        assert_eq!(chunks[1], vec![4, 5, 6]);
        assert_eq!(chunks[2], vec![7, 8, 9]);
        assert_eq!(chunks[3], vec![10]);
    }

    #[test]
    fn test_split_into_chunks_exact() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let chunks = split_into_chunks(&data, 2);

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], vec![1, 2]);
        assert_eq!(chunks[1], vec![3, 4]);
        assert_eq!(chunks[2], vec![5, 6]);
    }

    #[test]
    fn test_split_info_chunks_larger_than_data() {
        let data = vec![1, 2, 3];
        let chunks = split_into_chunks(&data, 10);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], vec![1, 2, 3]);
    }

    #[test]
    fn test_chunk_buffer_from_slice() {
        let data = b"hello world, this is a test";
        let buffer = chunk_buffer_from_slice(data, 5);

        assert_eq!(buffer.total_size, data.len());
        assert_eq!(buffer.chunk_count(), 6); // 27 : 5 = 5 full + 1 partial

        // Verify merge gives back original
        assert_eq!(buffer.merge(), data.to_vec());
    }

    #[test]
    fn test_with_capacity() {
        let buffer = ChunkBuffer::with_capacity(10);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_default() {
        let buffer: ChunkBuffer = Default::default();
        assert!(buffer.is_empty());
    }
}
