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

/// OPFS utilities for main thread
export async function downloadFromOpfs(fileName: string): Promise<void> {
  const file = await getOpfsFile(fileName);

  // Create download link
  const url = URL.createObjectURL(file);
  const a = document.createElement("a");
  a.href = url;
  a.download = fileName;
  a.click();
  URL.revokeObjectURL(url);
}

export async function listOpfsFiles(): Promise<string[]> {
  const root = await getOpfsRoot();
  const files: string[] = [];

  for await (const [name] of root.entries()) {
    files.push(name);
  }

  return files;
}