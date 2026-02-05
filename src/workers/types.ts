export type WorkerMessage = 
  | { type: 'init'}
  | { type: 'createPatch'; sourceFile: File; targetFile: File; outputName: string }
  | { type: 'applyPatch'; sourceFile: File; patchFile: File; outputName: string }
  | { type: 'getVersion' }
  | { type: 'hashFile'; file: File };

export type WorkerResponse =
  | { type: 'ready' }
  | { type: 'progress'; stage: string; percent: number; detail?: string }
  | { type: 'complete'; outputName: string; size: number }
  | { type: 'error'; message: string }
  | { type: 'identical' }
  | { type: 'version'; version: string }
  | { type: 'hash'; hash: string };

export type ProgressCallback = (stage: string, percent: number, detail?: string) => void;
export type CompleteCallback = (outputName: string, size: number) => void;
export type ErrorCallback = (message: string) => void;