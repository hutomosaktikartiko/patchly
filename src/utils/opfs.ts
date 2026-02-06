async function getOpfsRoot(): Promise<FileSystemDirectoryHandle> {
  return await navigator.storage.getDirectory();
}

async function createOpfsFile(
  filename: string,
): Promise<FileSystemWritableFileStream> {
  const root = await getOpfsRoot();
  const fileHandle = await root.getFileHandle(filename, {create: true});

  return await fileHandle.createWritable();
}

async function getOpfsFile(filename: string): Promise<File> {
  const root = await getOpfsRoot();
  const fileHandle = await root.getFileHandle(filename);
  
  return await fileHandle.getFile();
}

async function deleteOpfsFile(filename: string): Promise<void> {
  const root = await getOpfsRoot();

  return root.removeEntry(filename);
}

function isOpfsAvailable(): boolean {
  return 'storage' in navigator && 'getDirectory' in navigator.storage;
}

/**
 * Get a sync access handle for random access reads/writes.
 * Must be closed after use!
 */
async function getSyncAccessHandle(filename: string): Promise<FileSystemSyncAccessHandle> {
  const root = await getOpfsRoot();
  const fileHandle = await root.getFileHandle(filename, { create: true });
  return await fileHandle.createSyncAccessHandle();
}

/**
 * Stream a File object to OPFS, returning a hash of the data.
 * Uses FNV-1a hash (same as Rust implementation).
 */
async function streamFileToOpfs(
  file: File,
  filename: string,
  onProgress?: (bytesWritten: number, totalBytes: number) => void
): Promise<{ hash: bigint; size: number }> {
  const writable = await createOpfsFile(filename);
  const reader = file.stream().getReader();
  
  // FNV-1a hash constants (same as Rust)
  const FNV_OFFSET = 0xcbf29ce484222325n;
  const FNV_PRIME = 0x100000001b3n;
  let hash = FNV_OFFSET;
  let bytesWritten = 0;
  
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      
      // Update hash (FNV-1a) - process in batches for better performance
      const len = value.length;
      for (let i = 0; i < len; i++) {
        hash ^= BigInt(value[i]);
        hash = BigInt.asUintN(64, hash * FNV_PRIME);
      }
      
      // Write Uint8Array directly - no Blob wrapper needed!
      await writable.write(value);
      bytesWritten += len;
      
      if (onProgress) {
        onProgress(bytesWritten, file.size);
      }
    }
  } finally {
    await writable.close();
  }
  
  return { hash, size: bytesWritten };
}

/**
 * Stream a File object to OPFS WITHOUT hashing.
 * Use when hash validation is not needed (e.g., patch files).
 * Much faster and uses less memory than streamFileToOpfs.
 */
async function streamFileToOpfsNoHash(
  file: File,
  filename: string,
  onProgress?: (bytesWritten: number, totalBytes: number) => void
): Promise<{ size: number }> {
  const writable = await createOpfsFile(filename);
  const reader = file.stream().getReader();
  let bytesWritten = 0;
  
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      
      // Write Uint8Array directly - no hash calculation!
      await writable.write(value);
      bytesWritten += value.length;
      
      if (onProgress) {
        onProgress(bytesWritten, file.size);
      }
    }
  } finally {
    await writable.close();
  }
  
  return { size: bytesWritten };
}

/**
 * Safe delete that ignores errors if file doesn't exist
 */
async function safeDeleteOpfsFile(filename: string): Promise<void> {
  try {
    await deleteOpfsFile(filename);
  } catch {
    // Ignore - file may not exist
  }
}

export { 
  getOpfsRoot, 
  createOpfsFile, 
  getOpfsFile, 
  deleteOpfsFile, 
  safeDeleteOpfsFile,
  isOpfsAvailable,
  getSyncAccessHandle,
  streamFileToOpfs,
  streamFileToOpfsNoHash
};