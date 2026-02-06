import { formatBytes } from '../utils/bytes';
import { createOpfsFile } from '../utils/opfs';
import init, {PatchBuilder, PatchApplier, version, hash_data} from '../wams/patchly_wasm.js';
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
        
        const copy = new Uint8Array(patchChunk);
        await writable.write(new Blob([copy]));
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
      
      const copy = new Uint8Array(patchChunk);
      await writable.write(new Blob([copy]));
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

async function writeToOpfs(
  fileName: string,
  getData: () => Uint8Array | null,
  totalSize: number,
  onProgress?: (percent: number) => void
): Promise<void> {
  const writable = await createOpfsFile(fileName);

  let bytesWritten = 0;

  try {
    while (true) {
      const chunk = getData();
      if (!chunk || chunk.length === 0) break;

      const copy = new Uint8Array(chunk);
      await writable.write(new Blob([copy]));

      bytesWritten += chunk.length;

      if (onProgress && totalSize > 0) {
        onProgress((bytesWritten / totalSize) * 100);
      }
    }
  } finally {
    await writable.close();
  }
}

/// Apply patch operation
async function applyPatch(sourceFile: File, patchFile: File, outputName: string) {
  try {
    const applier = new PatchApplier();

    // Read patch file first to get expected sizes
    send({ type: 'progress', stage: 'Reading patch', percent: 0 });
    const patchData = new Uint8Array(await patchFile.arrayBuffer());
    applier.set_patch(patchData);

    const expectedSourceSize = Number(applier.expected_source_size());
    const expectedTargetSize = Number(applier.expected_target_size());

    // Validate source file size
    if (sourceFile.size !== expectedSourceSize) {
      send({
        type: 'error',
        message: `Source file size mismatch. Expected ${formatBytes(expectedSourceSize)}, got ${formatBytes(sourceFile.size)}`
      });
      return;
    }

    // Read source file
    send({ type: 'progress', stage: 'Reading source', percent: 10 });
    await readFileChunked(sourceFile, (chunk) => {
      applier.add_source_chunk(chunk);
      const percent = 10 + (applier.source_size() / sourceFile.size) * 30;
      send({
        type: 'progress',
        stage: 'Reading source',
        percent,
        detail: `${formatBytes(applier.source_size())} / ${formatBytes(sourceFile.size)}`
      });
    });

    // Validate source hash
    send({ type: 'progress', stage: 'Validating source', percent: 40 });
    try {
      applier.validate_source();
    } catch (err) {
      send({ type: 'error', message: `Source validation failed: ${err}` });
      return;
    }

    // Apply patch
    send({ type: 'progress', stage: 'Applying patch', percent: 45 });
    applier.prepare();

    // Write output to opfs using streaming
    send({ type: 'progress', stage: 'Writing output', percent: 50 });

    await writeToOpfs(
      outputName,
      () => {
        if (!applier.has_more_output()) return null;
        return applier.next_output_chunk(WRITE_CHUNK_SIZE)
      },
      expectedTargetSize,
      (percent) => {
        send({
          type: 'progress',
          stage: 'Writing output',
          percent: 50 + percent * 0.5,
          detail: `${formatBytes(expectedTargetSize - Number(applier.remaining_output_size()))} / ${formatBytes(expectedTargetSize)}`
        })
      }
    )

    send({ type: 'progress', stage: 'Complete', percent: 100 });
    send({ type: 'complete', outputName, size: expectedTargetSize });

    applier.reset();
  } catch (err) {
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