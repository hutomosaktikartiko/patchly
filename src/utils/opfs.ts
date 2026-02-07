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
async function createOpfsFile(
  filename: string,
): Promise<FileSystemWritableFileStream> {
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
 * Safely deletes a file, ignoring errors if file doesn't exist.
 *
 * @param filename - Name of the file to delete.
 * @returns true if deleted, false if file not found.
 */
async function safeDeleteOpfsFile(filename: string): Promise<boolean> {
  try {
    await deleteOpfsFile(filename);
    return true;
  } catch {
    return false;
  }
}

/**
 * Lists all files currently in OPFS.
 *
 * @returns Array of filenames.
 */
async function listOpfsFiles(): Promise<string[]> {
  const root = await getOpfsRoot();
  const files: string[] = [];

  for await (const [name] of root.entries()) {
    files.push(name);
  }

  return files;
}

/**
 * Clears all files from OPFS.
 * Useful for cleanup or debugging.
 *
 * @returns Number of files deleted.
 */
async function clearAllOpfsFiles(): Promise<number> {
  const root = await getOpfsRoot();
  let count = 0;

  for await (const [name] of root.entries()) {
    try {
      await root.removeEntry(name);
      count++;
    } catch {
      // Ignore errors for individual files
    }
  }

  return count;
}

/**
 * Gets a sync access handle for random access reads/writes.
 *
 * **Important**: Must be closed after use!
 *
 * @param filename - Name of the file to access.
 * @returns Sync access handle for the file.
 */
async function getSyncAccessHandle(
  filename: string,
): Promise<FileSystemSyncAccessHandle> {
  const root = await getOpfsRoot();
  const fileHandle = await root.getFileHandle(filename, { create: true });
  return await fileHandle.createSyncAccessHandle();
}

/** Progress callback for streaming operations. */
type StreamProgressCallback = (
  bytesWritten: number,
  totalBytes: number,
) => void;

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
  onProgress?: StreamProgressCallback,
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

/** Common MIME types by extension. */
const MIME_TYPES: Record<string, string> = {
  // Video
  ".mp4": "video/mp4",
  ".webm": "video/webm",
  ".mkv": "video/x-matroska",
  ".avi": "video/x-msvideo",
  ".mov": "video/quicktime",
  // Audio
  ".mp3": "audio/mpeg",
  ".wav": "audio/wav",
  ".ogg": "audio/ogg",
  ".flac": "audio/flac",
  // Images
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".gif": "image/gif",
  ".webp": "image/webp",
  // Documents
  ".pdf": "application/pdf",
  ".zip": "application/zip",
  ".json": "application/json",
  // Default
  ".patch": "application/octet-stream",
};

/** Gets MIME type from filename extension. */
function getMimeType(filename: string): string {
  const ext = filename.substring(filename.lastIndexOf(".")).toLowerCase();
  return MIME_TYPES[ext] || "application/octet-stream";
}

/**
 * Downloads a file from OPFS with correct MIME type.
 *
 * @param filename - Name of the file in OPFS to download.
 */
async function downloadFromOpfs(filename: string): Promise<void> {
  const file = await getOpfsFile(filename);

  // Get MIME type from extension (OPFS files have empty type)
  const mimeType = getMimeType(filename);

  // Use File.slice() to create new File with correct type (avoids loading entire file into memory)
  const typedFile = file.slice(0, file.size, mimeType);

  const url = URL.createObjectURL(typedFile);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();

  // Delay revoke to ensure download starts
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export {
  // Core operations
  getOpfsRoot,
  createOpfsFile,
  getOpfsFile,
  deleteOpfsFile,
  safeDeleteOpfsFile,
  // List & cleanup
  listOpfsFiles,
  clearAllOpfsFiles,
  // Sync access
  getSyncAccessHandle,
  // Streaming
  streamFileToOpfs,
  streamFileToOpfs as streamFileToOpfsNoHash, // Backwards compatibility
  // Download
  downloadFromOpfs,
  // Types
  type StreamProgressCallback,
};
