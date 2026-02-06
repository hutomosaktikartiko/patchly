# Patchly – Client-side File Diff & Patch Engine

A **100% client-side binary diff & patch tool** built with **Rust compiled to WebAssembly**, running entirely in the browser.

> **No backend. No upload. No server-side processing.**

---

## Project Architecture

```text
patchly/
│
├─ README.md
├─ package.json
├─ bun.lockb
├─ tsconfig.json
├─ vite.config.ts
├─ tailwind.config.ts
├─ postcss.config.js
│
├─ public/
│   └─ favicon.svg
│
├─ src/
│   ├─ main.tsx
│   ├─ App.tsx
│   ├─ App.css
│   ├─ index.css
│   │
│   ├─ assets/
│   │   └─ react.svg
│   │
│   ├─ workers/
│   │   ├─ index.ts
│   │   ├─ types.ts
│   │   └─ patchly.worker.ts    # Web Worker for WASM operations
│   │
│   ├─ wams/                     # WASM generated output
│   │   ├─ package.json
│   │   ├─ patchly_wasm.js
│   │   ├─ patchly_wasm.d.ts
│   │   ├─ patchly_wasm_bg.wasm
│   │   └─ patchly_wasm_bg.wasm.d.ts
│   │
│   ├─ utils/
│   │   ├─ bytes.ts              # Byte formatting utilities
│   │   └─ opfs.ts               # OPFS storage utilities
│   │
│   └─ types/
│       └─ opfs.d.ts
│
├─ rust/
│   ├─ Cargo.toml
│   ├─ Cargo.lock
│   └─ src/
│       ├─ lib.rs
│       ├─ diff/
│       │   ├─ mod.rs
│       │   ├─ rolling_hash.rs   # O(1) rolling hash for chunk matching
│       │   ├─ block_index.rs    # Memory-efficient hash→offset index
│       │   └─ streaming_diff.rs # Streaming diff generator
│       │
│       ├─ format/
│       │   ├─ mod.rs
│       │   └─ patch_format.rs   # Patch serialization/deserialization
│       │
│       └─ utils/
│           ├─ mod.rs
│           └─ buffer.rs         # ChunkBuffer for streaming
│
└─ scripts/
    └─ build-wasm.sh

```

---

## Key Features

- Generate **binary patch** from two versions of a file
- Apply patch to reconstruct the new file
- Works with **any file type** (binary-level)
- Fully client-side (no upload, no server)
- Supports large files (GB-scale, device-dependent)
- Streaming & chunk-based processing
- Smooth UI using Web Worker (non-blocking)
- Fully static deployment

---

## Tech Stack

### Core

- **Rust**
- **WebAssembly (WASN)**: Browser runtime
- **Rolling Hash**: Efficient chunk mathing
- **Binary Delta Encoding**: Patch generation

### Frontend

- **React v19**
- **Vite**
- **Tailwind CSS**

### Tooling

- **Bun**
- **Web Worker**: Background processing
- **OPFS**: Persistance client-side storage

### Deployment

- **Cloudflare Pages**: Static site hosting

---

## How It Works

### Generate Patch

```text
App initialized Web Worker + WASM
  ↓
User selects source file (old version)
  ↓
User selects target file (new version)
  ↓
Files are read in streaming chunks
  ↓
Source file is indexed (BlockIndex)
  ↓
Target file is processed with interleaved output:
  - Read target chunk
  - Generate patch instructions (COPY/INSERT)
  - Flush to OPFS immediately
  ↓
Check if files are identical (skip if same)
  ↓
User dodnloads patch file (.patch)
```

### Apply Patch

```text
User selects source file (original file)
  ↓
User selects patch file (.patch)
  ↓
Patch metadata is validated
  ↓
Source file size is checked against patch header
  ↓
Source file is read and hash validated
  ↓
Patch instructions are applied
  ↓
Output is streamed to OPFS storage
  ↓
User downloads reconstructured file
```

## Feature Checklist

### Core Functionality

- [x] Select old file
- [x] Select new file
- [x] Generate patch file
- [x] Download patch file
- [x] Apply patch to old file
- [x] Download reconstructed file
- [x] Byte-for-byte output verification

### Diff Engine

- [x] Chunk-based file reading
- [x] Rolling hash implementation
- [x] Binary block matching
- [x] Insert/copy patch instructions
- [x] Deterministic patch output
- [x] Buffer utilities for chunked processing

### Web Worker

- [x] Worker setup
- [x] WASM loading inside worker
- [x] Main thread ↔ Worker messaging
- [x] Progress reporting
- [x] Auto-cancel on reload / close

### Performance Optimization

- [x] Streaming input processing
- [x] Chunk-based diffing
- [x] Streaming output to disk (OPFS)
- [x] Transferable buffers (zero-copy)
- [x] ChunkBuffer for memory-efficient processing
- [x] True streaming architecture for GB-scale files
- [x] Stream source to block index only (BlockIndex)
- [x] Stream target directly to patch output (no instruction accumulation)
- [x] Stream patch output directly to OPFS
- [x] Streaming PatchApplier (no full output in memory)

### Safety & Validation

- [x] Old file hash verification before patch apply
- [x] Patch format validation
- [x] Mismatch detection (wrong based file)
- [x] Corrupted patch detection

### UI/UX

- [x] File picker
- [x] Mode selector (Generate / Apply)
- [x] Progress bar
- [x] Error messages
- [x] Patch size vs full size comparison

### Build & Deploy

- [x] Vite + WASM integration
- [] Deploy to Cloudflare Pages
