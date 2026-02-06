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

    /// Release unused capacity to reduce memory footprint.
    /// Call this after clear() when you want to free memory.
    pub fn shrink_to_fit(&mut self) {
        self.chunks.shrink_to_fit();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer = ChunkBuffer::new();
        assert!(buffer.total_size == 0);
        assert_eq!(buffer.total_size(), 0);
        assert_eq!(buffer.chunks.len(), 0);
    }

    #[test]
    fn test_push_and_size() {
        let mut buffer = ChunkBuffer::new();
        buffer.push(vec![1, 2, 3, 4, 5]);
        buffer.push(vec![6, 7, 8]);

        assert_eq!(buffer.total_size(), 8);
        assert_eq!(buffer.chunks.len(), 2);
        assert!(!(buffer.total_size == 0));
    }

    #[test]
    fn test_push_slice() {
        let mut buffer = ChunkBuffer::new();
        buffer.push_slice(b"hello");
        buffer.push_slice(b" world");

        assert_eq!(buffer.total_size(), 11);
        assert_eq!(buffer.chunks.len(), 2);
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
    fn test_default() {
        let buffer: ChunkBuffer = Default::default();
        assert!(buffer.total_size == 0);
    }
}
