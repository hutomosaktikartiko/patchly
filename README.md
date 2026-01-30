# AllCrypt - Client-side File Encryption

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
User selects old file
  ↓
User selects new file
  ↓
Web Worker loads Rust WASM
  ↓
File is read in chunks (1 MB)
  ↓
Rush compares binary chunks
  ↓
Patch intructions are generated
  ↓
Patch file (.patch) is downloaded
```

### Apply Patch

```text
User selects old file
  ↓
User selects patch file
  ↓
Web Worker loads Rust WASM
  ↓
Patch instructions are applied
  ↓
New file is constructed
  ↓
Result file is downloaded
```

## Feature Checklist

### Core Functionality

- [] Select old file
- [] Select new file
- [] Generate patch file
- [] Download patch file
- [] Apply patch to old file
- [] Download reconstructed file
- [] Byte-for-byte output verification

### Diff Engine

- [] Chunk-based file reading
- [] Rolling hash implementation
- [] Binary block matching
- [] Insert/copy patch instructions
- [] Deterministic patch output

### Web Worker

- [] Worker setup
- [] WASM loading inside worker
- [] Main thread ↔ Worker messaging
- [] Progress reporting
- [] Auto-cancel on reload / close

### Performance Optimization

- [] Streaming input processing
- [] Chunk-based diffing
- [] Streaming output to disk (OPFS)
- [] Transferable buffers (zero-copy)

### Safety & Validation

- [] Old file hash verification before patch apply
- [] Patch format validation
- [] Mismatch detection (wrong based file)
- [] Corrupted patch detection

### UI/UX

- [] File picker
- [ ] Mode selector (Generate / Apply)
- [] Progress bar
- [] Error messages
- [] Patch size vs full size comparison

### Build & Deploy

- [] Vite + WASM integration
- [] Bun-based local dev
- [] Production build
- [] Deploy to Cloudflare Pages
