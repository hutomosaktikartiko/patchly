/**
 * Patchly Web Worker
 *
 * Handles patch creation and application in a separate thread.
 * Uses OPFS for memory-efficient streaming of large files.
 */

import { formatSize } from "../utils/bytes";
import {
  createOpfsFile,
  streamFileToOpfs,
  getSyncAccessHandle,
  safeDeleteOpfsFile,
} from "../utils/opfs";
import init, {
  PatchBuilder,
  version,
  hash_data,
  parse_patch_header_only,
  StreamingHasher,
} from "../wams/patchly_wasm.js";
import type { WorkerMessage, WorkerResponse } from "./types";

// ============================================================================
// Constants
// ============================================================================

/** Chunk size for batched OPFS writes (1MB). */
const WRITE_CHUNK_SIZE = 1024 * 1024;

/** Size of reusable read buffer for small reads (64KB). */
const SMALL_READ_SIZE = 64 * 1024;

/** Patch header size in bytes. */
const HEADER_SIZE = 33;

/** COPY instruction type marker. */
const TYPE_COPY = 0x01;

/** INSERT instruction type marker. */
const TYPE_INSERT = 0x02;

/** Progress update interval in milliseconds. */
const PROGRESS_INTERVAL_MS = 100;

/** Temp file names for OPFS operations. */
const TEMP_FILES = {
  SOURCE: "_source.tmp",
  PATCH: "_patch.tmp",
} as const;

// ============================================================================
// State
// ============================================================================

let wasmInitialized = false;

// ============================================================================
// Utilities
// ============================================================================

/** Sends a message to the main thread. */
function send(msg: WorkerResponse): void {
  self.postMessage(msg);
}

/** Initializes the WASM module. */
async function initWasm(): Promise<void> {
  if (wasmInitialized) return;

  await init();
  wasmInitialized = true;
  send({ type: "ready" });
}

/**
 * Reads a file in chunks and feeds each chunk to a callback.
 *
 * @param file - File to read.
 * @param onChunk - Callback for each chunk.
 * @param onProgress - Optional progress callback (0-100).
 */
async function readFileChunked(
  file: File,
  onChunk: (chunk: Uint8Array) => void,
  onProgress?: (percent: number) => void,
): Promise<void> {
  const reader = file.stream().getReader();
  let bytesRead = 0;

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    onChunk(value);
    bytesRead += value.length;

    onProgress?.((bytesRead / file.size) * 100);
  }
}

// ============================================================================
// Patch Creation
// ============================================================================

/**
 * Creates a binary patch from source and target files.
 *
 * @param sourceFile - Original file.
 * @param targetFile - Modified file.
 * @param outputName - Output filename in OPFS.
 */
async function createPatch(
  sourceFile: File,
  targetFile: File,
  outputName: string,
): Promise<void> {
  try {
    const builder = new PatchBuilder();

    // Phase 1: Index source file (0-40%)
    send({ type: "progress", stage: "Indexing source", percent: 0 });

    await readFileChunked(sourceFile, (chunk) => {
      builder.add_source_chunk(chunk);
      send({
        type: "progress",
        stage: "Indexing source",
        percent: (builder.source_size() / sourceFile.size) * 40,
        detail: formatSize(builder.source_size()),
      });
    });

    builder.finalize_source();
    send({ type: "progress", stage: "Source indexed", percent: 40 });

    // Set target size for header
    builder.set_target_size(BigInt(targetFile.size));

    // Phase 2: Process target file (40-90%)
    const writable = await createOpfsFile(outputName);
    let totalWritten = 0;
    const reader = targetFile.stream().getReader();

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      builder.add_target_chunk(value);

      // Flush output when buffer is large enough
      while (
        builder.has_output() &&
        builder.pending_output_size() >= WRITE_CHUNK_SIZE
      ) {
        const patchChunk = builder.flush_output(WRITE_CHUNK_SIZE);
        if (patchChunk.length === 0) break;

        await writable.write(new Blob([patchChunk.slice()]));
        totalWritten += patchChunk.length;
      }

      send({
        type: "progress",
        stage: "Processing target",
        percent: 40 + (builder.target_size() / targetFile.size) * 50,
        detail: formatSize(builder.target_size()),
      });
    }

    // Check for identical files
    if (builder.are_files_identical()) {
      await writable.close();
      builder.reset();
      send({ type: "identical" });
      return;
    }

    // Phase 3: Finalize (90-100%)
    send({ type: "progress", stage: "Finalizing", percent: 90 });
    builder.finalize_target();

    // Flush remaining output
    while (builder.has_output()) {
      const patchChunk = builder.flush_output(WRITE_CHUNK_SIZE);
      if (patchChunk.length === 0) break;

      await writable.write(new Blob([patchChunk.slice()]));
      totalWritten += patchChunk.length;
    }

    await writable.close();
    builder.reset();

    send({ type: "progress", stage: "Complete", percent: 100 });
    send({ type: "complete", outputName, size: totalWritten });
  } catch (err) {
    send({ type: "error", message: `Create patch failed: ${err}` });
  }
}

// ============================================================================
// Patch Application
// ============================================================================

/** Parsed patch header information. */
interface PatchHeader {
  sourceSize: number;
  sourceHash: string;
  targetSize: number;
  headerSize: number;
}

/**
 * Applies a binary patch to a source file.
 *
 * Uses OPFS for memory-efficient streaming instead of loading
 * everything into WASM memory.
 *
 * @param sourceFile - Original file to patch.
 * @param patchFile - Patch file to apply.
 * @param outputName - Output filename in OPFS.
 */
async function applyPatch(
  sourceFile: File,
  patchFile: File,
  outputName: string,
): Promise<void> {
  try {
    // Phase 1: Stream patch file to OPFS (0-10%)
    send({ type: "progress", stage: "Reading patch", percent: 0 });

    await streamFileToOpfs(patchFile, TEMP_FILES.PATCH, (bytes, total) => {
      send({
        type: "progress",
        stage: "Reading patch",
        percent: (bytes / total) * 10,
        detail: formatSize(bytes),
      });
    });

    // Phase 2: Parse patch header (10%)
    send({ type: "progress", stage: "Parsing header", percent: 10 });

    const patchHandle = await getSyncAccessHandle(TEMP_FILES.PATCH);
    const headerBuffer = new Uint8Array(HEADER_SIZE);
    patchHandle.read(headerBuffer, { at: 0 });

    const headerInfo: PatchHeader = JSON.parse(
      parse_patch_header_only(headerBuffer),
    );

    // Validate source file size
    if (sourceFile.size !== headerInfo.sourceSize) {
      patchHandle.close();
      send({
        type: "error",
        message: `Source size mismatch. Expected ${formatSize(headerInfo.sourceSize)}, got ${formatSize(sourceFile.size)}`,
      });
      return;
    }

    // Phase 3: Stream source to OPFS with hash validation (12-40%)
    send({ type: "progress", stage: "Reading source", percent: 12 });

    const hashBuilder = new StreamingHasher();
    const sourceWritable = await createOpfsFile(TEMP_FILES.SOURCE);
    const sourceReader = sourceFile.stream().getReader();
    let sourceBytesWritten = 0;

    try {
      while (true) {
        const { done, value } = await sourceReader.read();
        if (done) break;

        hashBuilder.update(value);
        await sourceWritable.write(value);
        sourceBytesWritten += value.length;

        send({
          type: "progress",
          stage: "Reading source",
          percent: 12 + (sourceBytesWritten / sourceFile.size) * 28,
          detail: `${formatSize(sourceBytesWritten)} / ${formatSize(sourceFile.size)}`,
        });
      }
    } finally {
      await sourceWritable.close();
    }

    // Validate source hash
    send({ type: "progress", stage: "Validating source", percent: 40 });

    const computedHash = hashBuilder.finalize();
    if (computedHash !== headerInfo.sourceHash) {
      patchHandle.close();
      hashBuilder.free();
      send({
        type: "error",
        message: `Source hash mismatch. Expected ${headerInfo.sourceHash}, got ${computedHash}`,
      });
      return;
    }
    hashBuilder.free();

    // Phase 4: Apply patch instructions (45-95%)
    send({ type: "progress", stage: "Applying patch", percent: 45 });

    const sourceHandle = await getSyncAccessHandle(TEMP_FILES.SOURCE);
    const outputWritable = await createOpfsFile(outputName);

    let bytesWritten = 0;
    let patchOffset = HEADER_SIZE;
    const patchFileSize = patchHandle.getSize();

    // Reusable buffers to reduce allocations
    const instrHeaderBuffer = new Uint8Array(13);
    const outputBuffer = new Uint8Array(WRITE_CHUNK_SIZE);
    let outputBufferPos = 0;
    const smallReadBuffer = new Uint8Array(SMALL_READ_SIZE);

    const flushOutputBuffer = async (): Promise<void> => {
      if (outputBufferPos > 0) {
        await outputWritable.write(outputBuffer.slice(0, outputBufferPos));
        bytesWritten += outputBufferPos;
        outputBufferPos = 0;
      }
    };

    const writeToOutput = async (data: Uint8Array): Promise<void> => {
      let dataOffset = 0;
      while (dataOffset < data.length) {
        const spaceInBuffer = WRITE_CHUNK_SIZE - outputBufferPos;
        const bytesToCopy = Math.min(spaceInBuffer, data.length - dataOffset);

        outputBuffer.set(
          data.subarray(dataOffset, dataOffset + bytesToCopy),
          outputBufferPos,
        );
        outputBufferPos += bytesToCopy;
        dataOffset += bytesToCopy;

        if (outputBufferPos >= WRITE_CHUNK_SIZE) {
          await flushOutputBuffer();
        }
      }
    };

    let lastProgressUpdate = Date.now();

    try {
      while (patchOffset < patchFileSize) {
        // Read instruction type
        patchHandle.read(instrHeaderBuffer.subarray(0, 1), { at: patchOffset });
        const instrType = instrHeaderBuffer[0];
        patchOffset += 1;

        if (instrType === TYPE_COPY) {
          // COPY: offset(8) + length(4)
          patchHandle.read(instrHeaderBuffer.subarray(0, 12), {
            at: patchOffset,
          });
          patchOffset += 12;

          const copyOffset = Number(
            new DataView(instrHeaderBuffer.buffer).getBigUint64(0, true),
          );
          const copyLength = new DataView(instrHeaderBuffer.buffer).getUint32(
            8,
            true,
          );

          let remaining = copyLength;
          let srcOffset = copyOffset;
          while (remaining > 0) {
            const chunkSize = Math.min(remaining, SMALL_READ_SIZE);
            // Read into reusable buffer
            sourceHandle.read(smallReadBuffer.subarray(0, chunkSize), {
              at: srcOffset,
            });
            // IMPORTANT: Use slice() to copy data, not subarray() which is just a view
            // Otherwise the buffer may be overwritten before writeToOutput finishes
            await writeToOutput(smallReadBuffer.slice(0, chunkSize));
            remaining -= chunkSize;
            srcOffset += chunkSize;
          }
        } else if (instrType === TYPE_INSERT) {
          // INSERT: length(4) + data
          patchHandle.read(instrHeaderBuffer.subarray(0, 4), {
            at: patchOffset,
          });
          patchOffset += 4;

          const insertLength = new DataView(instrHeaderBuffer.buffer).getUint32(
            0,
            true,
          );

          let remaining = insertLength;
          let insertOffset = patchOffset;
          while (remaining > 0) {
            const chunkSize = Math.min(remaining, SMALL_READ_SIZE);
            // Read into reusable buffer
            patchHandle.read(smallReadBuffer.subarray(0, chunkSize), {
              at: insertOffset,
            });
            // IMPORTANT: Use slice() to copy data, not subarray() which is just a view
            await writeToOutput(smallReadBuffer.slice(0, chunkSize));
            remaining -= chunkSize;
            insertOffset += chunkSize;
          }
          patchOffset += insertLength;
        } else {
          throw new Error(`Unknown instruction type: ${instrType}`);
        }

        // Throttled progress updates
        const now = Date.now();
        if (now - lastProgressUpdate >= PROGRESS_INTERVAL_MS) {
          const totalWritten = bytesWritten + outputBufferPos;
          send({
            type: "progress",
            stage: "Writing output",
            percent: 45 + (totalWritten / headerInfo.targetSize) * 50,
            detail: `${formatSize(totalWritten)} / ${formatSize(headerInfo.targetSize)}`,
          });
          lastProgressUpdate = now;
        }
      }

      await flushOutputBuffer();
    } finally {
      sourceHandle.close();
      patchHandle.close();
      await outputWritable.close();
    }

    // Phase 5: Cleanup (98-100%)
    send({ type: "progress", stage: "Cleaning up", percent: 98 });
    await safeDeleteOpfsFile(TEMP_FILES.SOURCE);
    await safeDeleteOpfsFile(TEMP_FILES.PATCH);

    send({ type: "progress", stage: "Complete", percent: 100 });
    send({ type: "complete", outputName, size: headerInfo.targetSize });
  } catch (err) {
    // Cleanup on error
    await safeDeleteOpfsFile(TEMP_FILES.SOURCE);
    await safeDeleteOpfsFile(TEMP_FILES.PATCH);
    send({ type: "error", message: `Apply patch failed: ${err}` });
  }
}

// ============================================================================
// File Hashing
// ============================================================================

/**
 * Computes the hash of a file.
 *
 * @param file - File to hash.
 */
async function hashFile(file: File): Promise<void> {
  try {
    const data = new Uint8Array(await file.arrayBuffer());
    const hash = hash_data(data);
    send({ type: "hash", hash });
  } catch (err) {
    send({ type: "error", message: `Hash failed: ${err}` });
  }
}

// ============================================================================
// Message Handler
// ============================================================================

/** Handles messages from the main thread. */
self.onmessage = async (event: MessageEvent<WorkerMessage>): Promise<void> => {
  const msg = event.data;

  switch (msg.type) {
    case "init":
      await initWasm();
      break;

    case "createPatch":
      await createPatch(msg.sourceFile, msg.targetFile, msg.outputName);
      break;

    case "applyPatch":
      await applyPatch(msg.sourceFile, msg.patchFile, msg.outputName);
      break;

    case "getVersion":
      if (!wasmInitialized) await initWasm();
      send({ type: "version", version: version() });
      break;

    case "hashFile":
      if (!wasmInitialized) await initWasm();
      await hashFile(msg.file);
      break;
  }
};
