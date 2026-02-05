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
│   ├─ index.css
│   │
│   ├─ components/
│   │   ├─ FilePicker.tsx
│   │   ├─ ModeSelector.tsx      # Generate Patch / Apply Patch
│   │   ├─ ProgressBar.tsx
│   │   ├─ ActionButtons.tsx
│   │   └─ ErrorBanner.tsx
│   │
│   ├─ pages/
│   │   └─ Home.tsx
│   │
│   ├─ hooks/
│   │   ├─ usePatchWorker.ts
│   │   └─ useFileDiff.ts
│   │
│   ├─ worker/
│   │   ├─ diff.worker.ts
│   │   ├─ messages.ts
│   │   └─ types.ts
│   │
│   ├─ wasm/
│   │   └─ patch_wasm.ts
│   │
│   ├─ utils/
│   │   ├─ download.ts
│   │   ├─ file.ts
│   │   └─ constants.ts
│   │
│   └─ types/
│       └─ index.ts
│
├─ rust/
│   ├─ Cargo.toml
│   ├─ Cargo.lock
│   └─ src/
│       ├─ lib.rs
│       ├─ diff/
│       │   ├─ mod.rs
│       │   ├─ rolling_hash.rs
│       │   ├─ matcher.rs
│       │   └─ patch.rs
│       │
│       ├─ format/
│       │   ├─ mod.rs
│       │   └─ patch_format.rs
│       │
│       └─ utils/
│           └─ buffer.rs
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
Check is files are identical (skip if same)
  ↓
Rush compares binary chunks using rolling hash
  ↓
Patch instructions (COPY/INSERT) are generated
  ↓
Patch is written to OPFS storage
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
- [ ] True streaming architecture for GB-scale files
- [ ] Stream source to block index only
- [ ] Stream target directly to patch instructions
- [ ] Stream patch output directly to OPFS
- [ ] Remove WASM memory limit dependency

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
