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
│   ├─ workers/
│   │   ├─ index.ts               # Worker initialization & API
│   │   ├─ types.ts               # Message types for worker communication
│   │   └─ patchly.worker.ts      # Web Worker with WASM integration
│   │
│   ├─ wams/                      # WASM generated output (wasm-pack)
│   │   ├─ package.json
│   │   ├─ patchly_wasm.js
│   │   ├─ patchly_wasm.d.ts
│   │   ├─ patchly_wasm_bg.wasm
│   │   └─ patchly_wasm_bg.wasm.d.ts
│   │
│   ├─ utils/
│   │   ├─ bytes.ts               # Byte formatting utilities
│   │   └─ opfs.ts                # OPFS streaming utilities
│   │
│   └─ types/
│       └─ opfs.d.ts              # FileSystemSyncAccessHandle types
│
├─ rust/
│   ├─ Cargo.toml
│   ├─ Cargo.lock
│   └─ src/
│       ├─ lib.rs                 # WASM bindings & exports
│       ├─ diff/
│       │   ├─ mod.rs
│       │   ├─ rolling_hash.rs    # O(1) rolling hash for chunk matching
│       │   ├─ block_index.rs     # Memory-efficient hash→offset index
│       │   └─ streaming_diff.rs  # Streaming diff generator
│       │
│       ├─ format/
│       │   ├─ mod.rs
│       │   └─ patch_format.rs    # Patch serialization & FNV-1a hashing
│
└─ scripts/
    └─ build-wasm.sh
```

---

## Key Features

- Generate **binary patch** from two versions of a file
- Apply patch to reconstruct the new file
- Works with **any file type** (binary-level diffing)
- Fully client-side (no upload, no server)
- Supports **GB-scale files** with low memory usage (~400MB peak for 1.8GB data)
- **Streaming architecture** – never loads full files into memory
- **OPFS-based storage** for efficient random access
- **WASM-accelerated** hashing and processing
- Smooth UI using Web Worker (non-blocking main thread)
- Fully static deployment

---

## Tech Stack

### Core Engine (Rust → WASM)

- **Rust** – Core diff/patch engine
- **WebAssembly** – Browser runtime
- **Rolling Hash** – O(1) chunk matching (Adler-32 variant)
- **FNV-1a Hash** – File verification (64-bit)
- **Binary Delta Encoding** – COPY/INSERT instruction format

### Frontend

- **React v19**
- **Vite** – Build tooling & HMR
- **Tailwind CSS** – Styling

### Browser APIs

- **Web Worker** – Background WASM processing
- **OPFS (Origin Private File System)** – Persistent temp storage
- **FileSystemSyncAccessHandle** – Random access reads for patch application
- **Streams API** – Chunked file reading

### Tooling

- **Bun** – Package manager & runtime
- **wasm-pack** – Rust → WASM compilation

### Deployment

- **Cloudflare Pages** – Static site hosting (planned)

---

## How It Works

### Create Patch

```text
┌─────────────────────────────────────────────────────────────────┐
│                        CREATE PATCH FLOW                         │
└─────────────────────────────────────────────────────────────────┘

User selects source file (old version) + target file (new version)
                              ↓
              ┌───────────────────────────────────┐
              │        Web Worker + WASM          │
              └───────────────────────────────────┘
                              ↓
        Source file streamed in chunks (64KB)
                              ↓
        ┌─────────────────────────────────────────┐
        │        BlockIndex (Hash Table)          │
        │   Rolling hash → offset mapping         │
        │   ~5-10% of source size in memory       │
        └─────────────────────────────────────────┘
                              ↓
        Target file streamed in chunks (64KB)
                              ↓
        For each chunk:
          - Compute rolling hash
          - Match against BlockIndex
          - Generate COPY (if match) or INSERT (if new)
                              ↓
        ┌─────────────────────────────────────────┐
        │         Stream to OPFS Output           │
        │    No instruction accumulation          │
        └─────────────────────────────────────────┘
                              ↓
        Check if files identical → skip if same
                              ↓
        User downloads .patch file

Memory Usage: ~50-100MB for GB-scale files
```

### Apply Patch

```text
┌─────────────────────────────────────────────────────────────────┐
│                        APPLY PATCH FLOW                          │
└─────────────────────────────────────────────────────────────────┘

User selects source file (original) + patch file (.patch)
                              ↓
              ┌───────────────────────────────────┐
              │        Web Worker + WASM          │
              └───────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  Step 1: Stream patch file to OPFS temp (no hash needed)        │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  Step 2: Parse 33-byte header via WASM                          │
│          → sourceSize, sourceHash, targetSize                   │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  Step 3: Stream source file to OPFS temp                        │
│          → StreamingHasher for FNV-1a (native u64, zero alloc)  │
│          → Validate hash matches patch header                   │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  Step 4: Apply instructions using FileSystemSyncAccessHandle    │
│                                                                  │
│    For each instruction:                                         │
│      COPY:   Read from source OPFS → Write to output buffer     │
│      INSERT: Read from patch OPFS → Write to output buffer      │
│                                                                  │
│    Buffer pooling: 1MB output + 64KB read buffer (reused)       │
│    Batched writes: Flush at 1MB intervals                       │
└─────────────────────────────────────────────────────────────────┘
                              ↓
        Cleanup temp files (_source.tmp, _patch.tmp)
                              ↓
        User downloads reconstructed file

Memory Usage: ~400MB peak for 1.8GB total data (~22% overhead)
Speed: ~8 seconds for 1.8GB data
```

---

## Patch File Format

```text
┌─────────────────────────────────────────────────────────────────┐
│                         PATCH HEADER (33 bytes)                  │
├──────────────┬──────────────────────────────────────────────────┤
│ Magic        │ "PTCH" (4 bytes)                                 │
│ Version      │ 0x01 (1 byte)                                    │
│ Source Size  │ u64 little-endian (8 bytes)                      │
│ Source Hash  │ FNV-1a 64-bit (8 bytes)                          │
│ Target Size  │ u64 little-endian (8 bytes)                      │
│ Reserved     │ 4 bytes (future use)                             │
└──────────────┴──────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                        INSTRUCTIONS (variable)                   │
├──────────────┬──────────────────────────────────────────────────┤
│ COPY         │ 0x01 + offset (u64) + length (u32) = 13 bytes    │
│ INSERT       │ 0x02 + length (u32) + data (N bytes)             │
└──────────────┴──────────────────────────────────────────────────┘
```

---

## Feature Checklist

### Core Functionality

- [x] Select old file
- [x] Select new file
- [x] Generate patch file
- [x] Download patch file
- [x] Apply patch to old file
- [x] Download reconstructed file
- [x] Byte-for-byte output verification

### Diff Engine (Rust/WASM)

- [x] Chunk-based file reading (4KB chunks)
- [x] Rolling hash implementation (Adler-32 variant)
- [x] Binary block matching via BlockIndex
- [x] COPY/INSERT instruction generation
- [x] Deterministic patch output
- [x] Streaming architecture (no full file in memory)
- [x] StreamingHasher for zero-allocation hashing

### Patch Application

- [x] OPFS-based temp file storage
- [x] FileSystemSyncAccessHandle for random access
- [x] Buffer pooling (1MB output + 64KB read)
- [x] Batched writes to OPFS
- [x] WASM-accelerated FNV-1a hash validation
- [x] Time-based progress reporting (100ms intervals)

### Web Worker

- [x] Worker setup with WASM loading
- [x] Main thread ↔ Worker messaging
- [x] Progress reporting with stage info
- [x] Auto-cancel on reload/close
- [x] Error handling with cleanup

### Memory Optimization

- [x] Streaming input processing
- [x] OPFS for large file handling
- [x] Buffer reuse (no per-instruction allocation)
- [x] No Blob wrapper allocations
- [x] WASM hash instead of JS BigInt
- [x] Skip hash for patch file streaming
- [x] Memory cleanup to 0 after completion

### Safety & Validation

- [x] Source file hash verification (FNV-1a)
- [x] Source file size validation
- [x] Patch header magic validation
- [x] Mismatch detection (wrong base file)
- [x] Corrupted patch detection
- [x] OPFS temp file cleanup on error

### UI/UX

- [x] File picker for source/target/patch
- [x] Mode selector (Generate / Apply)
- [x] Progress bar with percentage
- [x] Stage display (Reading, Processing, Writing)
- [x] Error messages with details
- [x] Patch size vs full size comparison
- [x] Speed display

### Build & Deploy

- [ ] Deploy to Cloudflare Pages

---

## Performance Characteristics

| Operation    | Memory Peak | Speed          | Notes                     |
| ------------ | ----------- | -------------- | ------------------------- |
| Create Patch | ~50-100MB   | ~70 seconds/GB | BlockIndex in WASM memory |
| Apply Patch  | ~400MB      | ~8 seconds/GB  | 3 concurrent file streams |

_Tested with 283MB source + 848MB target files_

---

## Build Commands

```bash
# Install dependencies
bun install

# Build WASM (from rust/ directory)
wasm-pack build --target web --out-dir ../src/wams

# Development server
bun run dev

# Production build
bun run build
```
