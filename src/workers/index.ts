import { getOpfsFile, getOpfsRoot } from "../utils/opfs";
import type { CompleteCallback, ProgressCallback, ErrorCallback } from "./types";

export class PatchlyWorker {
  private worker: Worker;
  private onProgress?: ProgressCallback;
  private onComplete?: CompleteCallback;
  private onError?: ErrorCallback;
  private onIdentical?: () => void;
  private readyPromise: Promise<void>;
  private readyResolve?: () => void;

  constructor() {
    this.worker = new Worker(
      new URL('./patchly.worker.ts', import.meta.url),
      { type: "module" }
    );

    this.readyPromise = new Promise((resolve) => {
      this.readyResolve = resolve;
    });

    this.worker.onmessage = this.handleMessage.bind(this);
    this.worker.onerror = (err) => {
      console.error('Worker error:', err);
      this.onError?.(`Worker error: ${err.message}`);
    };

    // Initialize WASM
    this.worker.postMessage({ type: 'init' });
  }

  private handleMessage(event: MessageEvent) {
    const msg = event.data;

    switch (msg.type) {
      case 'ready':
        this.readyResolve?.();
        break;
      case 'progress':
        this.onProgress?.(msg.stage, msg.percent, msg.detail);
        break;
      case 'complete':
        this.onComplete?.(msg.outputName, msg.size);
        break;
      case 'error':
        this.onError?.(msg.message);
        break;
      case 'identical':
        this.onIdentical?.();
        break;
      case 'version':
        console.log('Pathly WASM version:', msg.version);
        break;
      case 'hash':
        console.log('File hash:', msg.hash);
        break;
    }
  }

  async waitReady(): Promise<void> {
    return this.readyPromise;
  }

  setCallbacks(
    onProgress?: ProgressCallback,
    onComplete?: CompleteCallback,
    onError?: ErrorCallback,
    onIdentical?: () => void,
  ) {
    this.onProgress = onProgress;
    this.onComplete = onComplete;
    this.onError = onError;
    this.onIdentical = onIdentical;
  }

  createPatch(sourceFile: File, targetFile: File, outputName: string) {
    this.worker.postMessage({
      type: 'createPatch',
      sourceFile,
      targetFile,
      outputName,
    });
  }

  applyPatch(sourceFile: File, patchFile: File, outputName: string) {
    this.worker.postMessage({
      type: 'applyPatch',
      sourceFile,
      patchFile,
      outputName
    });
  }

  getVersion() {
    this.worker.postMessage({ type: 'getVersion' });
  }

  terminate() {
    this.worker.terminate();
  }
}

/** Common MIME types by extension. */
const MIME_TYPES: Record<string, string> = {
  // Video
  '.mp4': 'video/mp4',
  '.webm': 'video/webm',
  '.mkv': 'video/x-matroska',
  '.avi': 'video/x-msvideo',
  '.mov': 'video/quicktime',
  // Audio
  '.mp3': 'audio/mpeg',
  '.wav': 'audio/wav',
  '.ogg': 'audio/ogg',
  '.flac': 'audio/flac',
  // Images
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.gif': 'image/gif',
  '.webp': 'image/webp',
  // Documents
  '.pdf': 'application/pdf',
  '.zip': 'application/zip',
  '.json': 'application/json',
  // Default
  '.patch': 'application/octet-stream',
};

/** Gets MIME type from filename extension. */
function getMimeType(filename: string): string {
  const ext = filename.substring(filename.lastIndexOf('.')).toLowerCase();
  return MIME_TYPES[ext] || 'application/octet-stream';
}

/** Downloads a file from OPFS with correct MIME type. */
export async function downloadFromOpfs(fileName: string): Promise<void> {
  const file = await getOpfsFile(fileName);
  
  // Get MIME type from extension (OPFS files have empty type)
  const mimeType = getMimeType(fileName);
  
  // Use File.slice() to create new File with correct type (avoids loading entire file into memory)
  const typedFile = file.slice(0, file.size, mimeType);
  
  const url = URL.createObjectURL(typedFile);
  const a = document.createElement("a");
  a.href = url;
  a.download = fileName;
  a.click();
  
  // Delay revoke to ensure download starts
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export async function listOpfsFiles(): Promise<string[]> {
  const root = await getOpfsRoot();
  const files: string[] = [];

  for await (const [name] of root.entries()) {
    files.push(name);
  }

  return files;
}