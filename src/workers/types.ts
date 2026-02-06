/**
 * Worker message and response types for Patchly web worker communication.
 */

// ============================================================================
// Worker Messages (main thread -> worker)
// ============================================================================

/** Initialize WASM module. */
export interface InitMessage {
  type: 'init';
}

/** Create a patch from source and target files. */
export interface CreatePatchMessage {
  type: 'createPatch';
  sourceFile: File;
  targetFile: File;
  outputName: string;
}

/** Apply a patch to a source file. */
export interface ApplyPatchMessage {
  type: 'applyPatch';
  sourceFile: File;
  patchFile: File;
  outputName: string;
}

/** Get WASM library version. */
export interface GetVersionMessage {
  type: 'getVersion';
}

/** Hash a file. */
export interface HashFileMessage {
  type: 'hashFile';
  file: File;
}

/** Union of all worker input messages. */
export type WorkerMessage = 
  | InitMessage
  | CreatePatchMessage
  | ApplyPatchMessage
  | GetVersionMessage
  | HashFileMessage;

// ============================================================================
// Worker Responses (worker -> main thread)
// ============================================================================

/** WASM initialized and ready. */
export interface ReadyResponse {
  type: 'ready';
}

/** Operation progress update. */
export interface ProgressResponse {
  type: 'progress';
  stage: string;
  percent: number;
  detail?: string;
}

/** Operation completed successfully. */
export interface CompleteResponse {
  type: 'complete';
  outputName: string;
  size: number;
}

/** Operation failed. */
export interface ErrorResponse {
  type: 'error';
  message: string;
}

/** Source and target files are identical. */
export interface IdenticalResponse {
  type: 'identical';
}

/** Version response. */
export interface VersionResponse {
  type: 'version';
  version: string;
}

/** Hash result. */
export interface HashResponse {
  type: 'hash';
  hash: string;
}

/** Union of all worker output responses. */
export type WorkerResponse =
  | ReadyResponse
  | ProgressResponse
  | CompleteResponse
  | ErrorResponse
  | IdenticalResponse
  | VersionResponse
  | HashResponse;

// ============================================================================
// Callback Types
// ============================================================================

/** Callback for progress updates. */
export type ProgressCallback = (stage: string, percent: number, detail?: string) => void;

/** Callback for operation completion. */
export type CompleteCallback = (outputName: string, size: number) => void;

/** Callback for errors. */
export type ErrorCallback = (message: string) => void;