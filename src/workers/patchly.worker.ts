import { formatBytes } from '../utils/bytes';
import { createOpfsFile, streamFileToOpfsNoHash, getSyncAccessHandle, safeDeleteOpfsFile } from '../utils/opfs';
import init, {PatchBuilder, version, hash_data, parse_patch_header_only, WasmHashBuilder} from '../wams/patchly_wasm.js';
import type { WorkerMessage, WorkerResponse } from './types';

// Chunk size for writing to OPFS (1MB)
const WRITE_CHUNK_SIZE = 1024 * 1024;

let wasmInitialized = false;

/// Send message to main thread
function send(msg: WorkerResponse) {
  self.postMessage(msg);
}

/// Initialize WASM
async function initWasm() {
  if (wasmInitialized) return;

  await init();
  wasmInitialized = true;
  send ({ type: 'ready' });
}

/// Read file in chunks and feed to callback
async function readFileChunked(
  file: File,
  onChunk: (chunk: Uint8Array) => void,
  onProgress?: (percent: number) => void
): Promise<void> {
  const reader = file.stream().getReader();
  let bytesRead = 0;

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    onChunk(value);
    bytesRead += value.length;

    if (onProgress) {
      onProgress((bytesRead / file.size) * 100);
    }
  }
}

/// Create patch operation
async function createPatch(sourceFile: File, targetFile: File, outputName: string) {
  try {
    const builder = new PatchBuilder();

    // Read and index source file
    send({ type: 'progress', stage: 'Indexing source', percent: 0 });
    await readFileChunked(sourceFile, (chunk) => {
      builder.add_source_chunk(chunk);
      send({
        type: 'progress',
        stage: 'Indexing source',
        percent: (builder.source_size() / sourceFile.size) * 40,
        detail: formatBytes(builder.source_size()) 
      });
    });

    // Finalize source
    builder.finalize_source();
    send({ type: 'progress', stage: 'Source indexed', percent: 40 });

    // Set target size for header
    builder.set_target_size(BigInt(targetFile.size));

    // Open output file for streaming
    const writable = await createOpfsFile(outputName);
    let totalWritten = 0;

    // Process target file
    const reader = targetFile.stream().getReader();
    
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      // Process this chunk
      builder.add_target_chunk(value);

      // Flush any available output
      while (builder.has_output() && builder.pending_output_size() >= WRITE_CHUNK_SIZE) {
        const patchChunk = builder.flush_output(WRITE_CHUNK_SIZE);
        if (patchChunk.length === 0) break;
        
        await writable.write(new Blob([patchChunk.slice()]));
        totalWritten += patchChunk.length;
      }

      // Report progress
      const percent = 40 + (builder.target_size() / targetFile.size) * 50;
      send({
        type: 'progress',
        stage: 'Processing target',
        percent,
        detail: formatBytes(builder.target_size())
      });
    }

    // Check if files are identical
    if (builder.are_files_identical()) {
      await writable.close();
      builder.reset();
      send({ type: 'identical' });
      return;
    }

    // Finalize target processing
    send({ type: 'progress', stage: 'Finalizing', percent: 90 });
    builder.finalize_target();

    // Flush all remaining output
    while (builder.has_output()) {
      const patchChunk = builder.flush_output(WRITE_CHUNK_SIZE);
      if (patchChunk.length === 0) break;
      
      await writable.write(new Blob([patchChunk.slice()]));
      totalWritten += patchChunk.length;
    }

    await writable.close();

    send({ type: 'progress', stage: 'Complete', percent: 100 });
    send({ type: 'complete', outputName, size: totalWritten });

    builder.reset();
  } catch (err) {
    send({ type: 'error', message: `Create patch failed: ${err}`});
  }
}

/// Apply patch operation using OPFS streaming (memory-efficient)
/// This approach streams source and patch to temp files, then reads from OPFS
/// for each instruction instead of holding everything in WASM memory.
async function applyPatch(sourceFile: File, patchFile: File, outputName: string) {
  const SOURCE_TEMP = '_source.tmp';
  const PATCH_TEMP = '_patch.tmp';
  
  // Instruction type markers (same as Rust)
  const TYPE_COPY = 0x01;
  const TYPE_INSERT = 0x02;
  
  try {
    // Step 1: Stream patch file to OPFS temp (no hash needed for patch)
    send({ type: 'progress', stage: 'Reading patch', percent: 0 });
    await streamFileToOpfsNoHash(patchFile, PATCH_TEMP, (bytes, total) => {
      send({
        type: 'progress',
        stage: 'Reading patch',
        percent: (bytes / total) * 10,
        detail: formatBytes(bytes)
      });
    });
    
    // Step 2: Parse patch header ONLY (33 bytes) - not the entire file!
    send({ type: 'progress', stage: 'Parsing header', percent: 10 });
    const patchHandle = await getSyncAccessHandle(PATCH_TEMP);
    const headerBuffer = new Uint8Array(33);
    patchHandle.read(headerBuffer, { at: 0 });
    
    // Parse header using WASM
    const headerInfoJson = parse_patch_header_only(headerBuffer);
    const headerInfo: { sourceSize: number; sourceHash: string; targetSize: number; headerSize: number } = JSON.parse(headerInfoJson);
    
    // Validate source file size early
    if (sourceFile.size !== headerInfo.sourceSize) {
      patchHandle.close();
      send({
        type: 'error',
        message: `Source file size mismatch. Expected ${formatBytes(headerInfo.sourceSize)}, got ${formatBytes(sourceFile.size)}`
      });
      return;
    }
    
    // Step 3: Stream source file to OPFS temp with WASM hash (memory efficient!)
    send({ type: 'progress', stage: 'Reading source', percent: 12 });
    
    // Use WASM HashBuilder instead of JS BigInt - much more memory efficient
    const hashBuilder = new WasmHashBuilder();
    const sourceWritable = await createOpfsFile(SOURCE_TEMP);
    const sourceReader = sourceFile.stream().getReader();
    let sourceBytesWritten = 0;
    
    try {
      while (true) {
        const { done, value } = await sourceReader.read();
        if (done) break;
        
        // Update hash using WASM (native u64, no BigInt allocations!)
        hashBuilder.update(value);
        
        // Write to OPFS
        await sourceWritable.write(value);
        sourceBytesWritten += value.length;
        
        send({
          type: 'progress',
          stage: 'Reading source',
          percent: 12 + (sourceBytesWritten / sourceFile.size) * 28,
          detail: `${formatBytes(sourceBytesWritten)} / ${formatBytes(sourceFile.size)}`
        });
      }
    } finally {
      await sourceWritable.close();
    }
    
    // Validate source hash using WASM result
    send({ type: 'progress', stage: 'Validating source', percent: 40 });
    const computedHashHex = hashBuilder.finalize();
    if (computedHashHex !== headerInfo.sourceHash) {
      patchHandle.close();
      hashBuilder.free(); // Free WASM memory
      send({
        type: 'error',
        message: `Source hash mismatch. Expected ${headerInfo.sourceHash}, got ${computedHashHex}`
      });
      return;
    }
    hashBuilder.free(); // Free WASM memory
    
    // Step 4: Apply patch by streaming through instructions
    send({ type: 'progress', stage: 'Applying patch', percent: 45 });
    
    const sourceHandle = await getSyncAccessHandle(SOURCE_TEMP);
    const outputWritable = await createOpfsFile(outputName);
    
    let bytesWritten = 0;
    let patchOffset = 33; // Start after header
    const patchFileSize = patchHandle.getSize();
    
    // Buffer for reading instruction headers (reused)
    const instrHeaderBuffer = new Uint8Array(13);
    
    // Output buffer for batching writes (1MB) - REUSED to reduce allocations
    const OUTPUT_BUFFER_SIZE = 1024 * 1024;
    const outputBuffer = new Uint8Array(OUTPUT_BUFFER_SIZE);
    let outputBufferPos = 0;
    
    // Reusable read buffer for small reads (64KB)
    const SMALL_READ_SIZE = 64 * 1024;
    const smallReadBuffer = new Uint8Array(SMALL_READ_SIZE);
    
    // Helper to flush output buffer to OPFS
    const flushOutputBuffer = async () => {
      if (outputBufferPos > 0) {
        // Write subarray directly - no Blob wrapper needed!
        await outputWritable.write(outputBuffer.slice(0, outputBufferPos));
        bytesWritten += outputBufferPos;
        outputBufferPos = 0;
      }
    };
    
    // Helper to write data to output buffer, flushing when full
    const writeToOutput = async (data: Uint8Array) => {
      let dataOffset = 0;
      while (dataOffset < data.length) {
        const spaceInBuffer = OUTPUT_BUFFER_SIZE - outputBufferPos;
        const bytesToCopy = Math.min(spaceInBuffer, data.length - dataOffset);
        
        outputBuffer.set(data.subarray(dataOffset, dataOffset + bytesToCopy), outputBufferPos);
        outputBufferPos += bytesToCopy;
        dataOffset += bytesToCopy;
        
        if (outputBufferPos >= OUTPUT_BUFFER_SIZE) {
          await flushOutputBuffer();
        }
      }
    };
    
    let lastProgressUpdate = Date.now();
    const PROGRESS_INTERVAL = 100; // Update every 100ms
    
    try {
      while (patchOffset < patchFileSize) {
        // Read instruction type
        patchHandle.read(instrHeaderBuffer.subarray(0, 1), { at: patchOffset });
        const instrType = instrHeaderBuffer[0];
        patchOffset += 1;
        
        if (instrType === TYPE_COPY) {
          // Read COPY instruction: offset(8) + length(4) = 12 bytes
          patchHandle.read(instrHeaderBuffer.subarray(0, 12), { at: patchOffset });
          patchOffset += 12;
          
          const copyOffset = Number(new DataView(instrHeaderBuffer.buffer).getBigUint64(0, true));
          const copyLength = new DataView(instrHeaderBuffer.buffer).getUint32(8, true);
          
          // Read from source in chunks using reusable buffer
          let remaining = copyLength;
          let srcOffset = copyOffset;
          while (remaining > 0) {
            const chunkSize = Math.min(remaining, SMALL_READ_SIZE);
            const chunk = smallReadBuffer.subarray(0, chunkSize);
            sourceHandle.read(chunk, { at: srcOffset });
            await writeToOutput(chunk);
            remaining -= chunkSize;
            srcOffset += chunkSize;
          }
          
        } else if (instrType === TYPE_INSERT) {
          // Read INSERT instruction: length(4) = 4 bytes
          patchHandle.read(instrHeaderBuffer.subarray(0, 4), { at: patchOffset });
          patchOffset += 4;
          
          const insertLength = new DataView(instrHeaderBuffer.buffer).getUint32(0, true);
          
          // Read INSERT data in chunks using reusable buffer
          let remaining = insertLength;
          let insertOffset = patchOffset;
          while (remaining > 0) {
            const chunkSize = Math.min(remaining, SMALL_READ_SIZE);
            const chunk = smallReadBuffer.subarray(0, chunkSize);
            patchHandle.read(chunk, { at: insertOffset });
            await writeToOutput(chunk);
            remaining -= chunkSize;
            insertOffset += chunkSize;
          }
          patchOffset += insertLength;
          
        } else {
          throw new Error(`Unknown instruction type: ${instrType}`);
        }
        
        // Report progress periodically (time-based, not per-instruction)
        const now = Date.now();
        if (now - lastProgressUpdate >= PROGRESS_INTERVAL) {
          const totalWritten = bytesWritten + outputBufferPos;
          const percent = 45 + (totalWritten / headerInfo.targetSize) * 50;
          send({
            type: 'progress',
            stage: 'Writing output',
            percent,
            detail: `${formatBytes(totalWritten)} / ${formatBytes(headerInfo.targetSize)}`
          });
          lastProgressUpdate = now;
        }
      }
      
      // Flush any remaining data
      await flushOutputBuffer();
      
    } finally {
      sourceHandle.close();
      patchHandle.close();
      await outputWritable.close();
    }
    
    // Step 5: Cleanup temp files
    send({ type: 'progress', stage: 'Cleaning up', percent: 98 });
    await safeDeleteOpfsFile(SOURCE_TEMP);
    await safeDeleteOpfsFile(PATCH_TEMP);
    
    send({ type: 'progress', stage: 'Complete', percent: 100 });
    send({ type: 'complete', outputName, size: headerInfo.targetSize });
    
  } catch (err) {
    // Cleanup on error
    await safeDeleteOpfsFile(SOURCE_TEMP);
    await safeDeleteOpfsFile(PATCH_TEMP);
    send({ type: 'error', message: `Apply patch failed: ${err}` });
  }
}

/// Hash file operation
async function hashFile(file: File) {
  try {
    const data = new Uint8Array(await file.arrayBuffer());
    const hash = hash_data(data);
    send({ type: 'hash', hash });
  } catch (err) {
    send({ type: 'error', message: `Hash failed: ${err}`});
  }
}

/// Handle messages from main thread
self.onmessage = async (event: MessageEvent<WorkerMessage>) => {
  const msg = event.data;

  switch (msg.type) {
    case 'init':
      await initWasm();
      break;
    case 'createPatch':
      await createPatch(msg.sourceFile, msg.targetFile, msg.outputName);
      break;
    case 'applyPatch':
      await applyPatch(msg.sourceFile, msg.patchFile, msg.outputName);
      break;
    case 'getVersion':
      if (!wasmInitialized) await initWasm();
      send({ type: 'version', version: version() });
      break;
    case 'hashFile':
      if (!wasmInitialized) await initWasm();
      await hashFile(msg.file);
      break;
  }
}