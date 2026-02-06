/**
 * OPFS (Origin Private File System) utilities for streaming file operations.
 * Provides efficient file handling in web workers without memory pressure.
 */

/** Returns the OPFS root directory handle. */
async function getOpfsRoot(): Promise<FileSystemDirectoryHandle> {
  return await navigator.storage.getDirectory();
}

/**
 * Creates a new file in OPFS and returns a writable stream.
 * 
 * @param filename - Name of the file to create.
 * @returns Writable stream for the file.
 */
async function createOpfsFile(filename: string): Promise<FileSystemWritableFileStream> {
  const root = await getOpfsRoot();
  const fileHandle = await root.getFileHandle(filename, { create: true });
  return await fileHandle.createWritable();
}

/**
 * Gets an existing file from OPFS.
 * 
 * @param filename - Name of the file to retrieve.
 * @returns The file object.
 * @throws If file doesn't exist.
 */
async function getOpfsFile(filename: string): Promise<File> {
  const root = await getOpfsRoot();
  const fileHandle = await root.getFileHandle(filename);
  return await fileHandle.getFile();
}

/**
 * Deletes a file from OPFS.
 * 
 * @param filename - Name of the file to delete.
 * @throws If file doesn't exist.
 */
async function deleteOpfsFile(filename: string): Promise<void> {
  const root = await getOpfsRoot();
  return root.removeEntry(filename);
}

/**
 * Gets a sync access handle for random access reads/writes.
 * 
 * **Important**: Must be closed after use!
 * 
 * @param filename - Name of the file to access.
 * @returns Sync access handle for the file.
 */
async function getSyncAccessHandle(filename: string): Promise<FileSystemSyncAccessHandle> {
  const root = await getOpfsRoot();
  const fileHandle = await root.getFileHandle(filename, { create: true });
  return await fileHandle.createSyncAccessHandle();
}

/** Progress callback for streaming operations. */
type StreamProgressCallback = (bytesWritten: number, totalBytes: number) => void;

/**
 * Streams a File object to OPFS without computing hash.
 * Use when hash validation is not needed (e.g., patch files).
 * 
 * @param file - Source file to stream.
 * @param filename - Destination filename in OPFS.
 * @param onProgress - Optional progress callback.
 * @returns Object containing the total bytes written.
 */
async function streamFileToOpfs(
  file: File,
  filename: string,
  onProgress?: StreamProgressCallback
): Promise<{ size: number }> {
  const writable = await createOpfsFile(filename);
  const reader = file.stream().getReader();
  let bytesWritten = 0;
  
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      
      await writable.write(value);
      bytesWritten += value.length;
      
      onProgress?.(bytesWritten, file.size);
    }
  } finally {
    await writable.close();
  }
  
  return { size: bytesWritten };
}

/**
 * Safely deletes a file, ignoring errors if file doesn't exist.
 * 
 * @param filename - Name of the file to delete.
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
  getSyncAccessHandle,
  streamFileToOpfs,
  // Backwards compatibility alias
  streamFileToOpfs as streamFileToOpfsNoHash,
  type StreamProgressCallback
};