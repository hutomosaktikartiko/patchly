/**
 * Extended OPFS types not included in TypeScript's lib.dom.d.ts.
 * These are needed for web worker file system access.
 */

interface FileSystemDirectoryHandle {
  /** Iterates over all entries in the directory. */
  entries(): AsyncIterableIterator<[string, FileSystemHandle]>;
  /** Iterates over all entry names in the directory. */
  keys(): AsyncIterableIterator<string>;
  /** Iterates over all handles in the directory. */
  values(): AsyncIterableIterator<FileSystemHandle>;
}

interface FileSystemFileHandle {
  /** Creates a synchronous access handle for direct file operations. */
  createSyncAccessHandle(): Promise<FileSystemSyncAccessHandle>;
}

/**
 * Synchronous access handle for direct file reads/writes.
 * Only available in web workers.
 */
interface FileSystemSyncAccessHandle {
  /**
   * Reads data from the file into a buffer.
   * @param buffer - Buffer to read into.
   * @param options - Optional read position.
   * @returns Number of bytes read.
   */
  read(buffer: ArrayBuffer | ArrayBufferView, options?: { at?: number }): number;
  
  /**
   * Writes data from a buffer to the file.
   * @param buffer - Buffer to write from.
   * @param options - Optional write position.
   * @returns Number of bytes written.
   */
  write(buffer: ArrayBuffer | ArrayBufferView, options?: { at?: number }): number;
  
  /**
   * Truncates or extends the file to the specified size.
   * @param size - New file size in bytes.
   */
  truncate(size: number): void;
  
  /** Returns the current file size in bytes. */
  getSize(): number;
  
  /** Flushes any pending writes to disk. */
  flush(): void;
  
  /** Closes the handle and releases the file lock. */
  close(): void;
}
