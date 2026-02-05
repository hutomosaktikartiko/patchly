import { useState, useRef, useCallback, useEffect } from "react";
import { PatchlyWorker, downloadFromOpfs } from "./workers";
import { formatBytes } from "./utils/bytes";

type Mode = "create" | "apply";
type Status = "idle" | "processing" | "success" | "error";

function App() {
  const [mode, setMode] = useState<Mode>("create");
  const [status, setStatus] = useState<Status>("idle");
  const [progress, setProgress] = useState(0);
  const [logs, setLogs] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  // Files
  const [sourceFile, setSourceFile] = useState<File | null>(null);
  const [targetFile, setTargetFile] = useState<File | null>(null);
  const [patchFile, setPatchFile] = useState<File | null>(null);

  // Output
  const [outputName, setOutputName] = useState<string | null>(null);
  const [outputSize, setOutputSize] = useState(0);

  // Worker ref
  const workerRef = useRef<PatchlyWorker | null>(null);

  // Add log helper
  const addLog = useCallback((msg: string) => {
    setLogs((prev) => [...prev, `[${new Date().toLocaleTimeString()}] ${msg}`]);
  }, []);

  // Initial worker
  useEffect(() => {
    const worker = new PatchlyWorker();

    worker.setCallbacks(
      // on progress
      (stage, percent, detail) => {
        setProgress(percent);
        addLog(`${stage}: ${percent.toFixed(1)}% ${detail || ""}`);
      },
      // on complete
      (name, size) => {
        setStatus("success");
        setOutputName(name);
        setOutputSize(size);
        setProgress(100);
        addLog(`Done! Output: ${name} (${formatBytes(size)})`);
      },
      // on error
      (message) => {
        setStatus("error");
        setError(message);
        addLog(`ERROR: ${message}`);
      }
    );

    worker.waitReady().then(() => {
      addLog("WASM module ready");
    });

    workerRef.current = worker;

    return () => worker.terminate();
  }, [addLog]);

  const handleStart = () => {
    if (mode === "create"){
      if (!sourceFile || !targetFile) {
        setError("Please select both source and target files");
        return;
      }

      setStatus("processing");
      setProgress(0);
      setLogs([]);
      setError(null);
      addLog(`Creating patch: ${sourceFile.name} -> ${targetFile.name}`);
      workerRef.current?.createPatch(sourceFile, targetFile, `${targetFile.name}.patch`);
    } else {
      if (!sourceFile || !patchFile) {
        setError("Please select both source and patch files");
        return;
      }

      setStatus("processing");
      setProgress(0);
      setLogs([]);
      setError(null);
      addLog(`Applying patch to: ${sourceFile.name}`);
      const outName = patchFile.name.replace(/\.patch$/, "") || "output";
      workerRef.current?.applyPatch(sourceFile, patchFile, outName);
    }
  }

  const handleDownload = async () => {
    if (!outputName) return;

    await downloadFromOpfs(outputName);
    addLog(`Downloaded: ${outputName}`);
  }

  const handleReset = () => {
    setStatus("idle");
    setProgress(0);
    setLogs([]);
    setError(null);
    setOutputName(null);
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
       <div className="max-w-2xl mx-auto">
        {/* Header */}
        <h1 className="text-3xl font-bold mb-2">Patchly</h1>
        <p className="text-gray-400 mb-8">Client-side binary diff & patch tool</p>

        {/* Mode Selector */}
        <div className="flex gap-2 mb-6">
          <button
            onClick={() => setMode("create")}
            className={`px-4 py-2 rounded ${
              mode === "create" ? "bg-blue-600" : "bg-gray-700"
            }`}
          >
            Create Patch
          </button>
          <button
            onClick={() => setMode("apply")}
            className={`px-4 py-2 rounded ${
              mode === "apply" ? "bg-blue-600" : "bg-gray-700"
            }`}
          >
            Apply Patch
          </button>
        </div>

        {/* File Inputs */}
        <div className="space-y-4 mb-6">
          {/* Source File (both modes) */}
          <div>
            <label className="block text-sm text-gray-400 mb-1">
              Source File {mode === "create" ? "(old version)" : "(original)"}
            </label>
            <input
              type="file"
              onChange={(e) => setSourceFile(e.target.files?.[0] || null)}
              className="w-full bg-gray-800 rounded p-2"
              disabled={status === "processing"}
            />
            {sourceFile && (
              <p className="text-sm text-gray-500 mt-1">
                {sourceFile.name} ({formatBytes(sourceFile.size)})
              </p>
            )}
          </div>

          {/* Target or Patch File */}
          {mode === "create" ? (
            <div>
              <label className="block text-sm text-gray-400 mb-1">
                Target File (new version)
              </label>
              <input
                type="file"
                onChange={(e) => setTargetFile(e.target.files?.[0] || null)}
                className="w-full bg-gray-800 rounded p-2"
                disabled={status === "processing"}
              />
              {targetFile && (
                <p className="text-sm text-gray-500 mt-1">
                  {targetFile.name} ({formatBytes(targetFile.size)})
                </p>
              )}
            </div>
          ) : (
            <div>
              <label className="block text-sm text-gray-400 mb-1">
                Patch File (.patch)
              </label>
              <input
                type="file"
                accept=".patch"
                onChange={(e) => setPatchFile(e.target.files?.[0] || null)}
                className="w-full bg-gray-800 rounded p-2"
                disabled={status === "processing"}
              />
              {patchFile && (
                <p className="text-sm text-gray-500 mt-1">
                  {patchFile.name} ({formatBytes(patchFile.size)})
                </p>
              )}
            </div>
          )}
        </div>

        {/* Action Button */}
        <button
          onClick={handleStart}
          disabled={status === "processing"}
          className="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-600 py-3 rounded font-semibold mb-6"
        >
          {status === "processing"
            ? "Processing..."
            : mode === "create"
            ? "Create Patch"
            : "Apply Patch"}
        </button>

        {/* Error Message */}
        {error && (
          <div className="bg-red-900/50 border border-red-500 text-red-200 p-3 rounded mb-4">
            {error}
          </div>
        )}

        {/* Progress Bar */}
        {status === "processing" && (
          <div className="mb-6">
            <div className="flex justify-between text-sm mb-1">
              <span>Progress</span>
              <span>{progress.toFixed(1)}%</span>
            </div>
            <div className="h-3 bg-gray-700 rounded overflow-hidden">
              <div
                className="h-full bg-blue-500 transition-all"
                style={{ width: `${progress}%` }}
              />
            </div>
          </div>
        )}

        {/* Success Result */}
        {status === "success" && outputName && (
          <div className="bg-green-900/50 border border-green-500 p-4 rounded mb-6">
            <p className="mb-2">
              Output: <strong>{outputName}</strong> ({formatBytes(outputSize)})
            </p>
            <div className="flex gap-2">
              <button
                onClick={handleDownload}
                className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded"
              >
                Download
              </button>
              <button
                onClick={handleReset}
                className="bg-gray-600 hover:bg-gray-700 px-4 py-2 rounded"
              >
                New Operation
              </button>
            </div>
          </div>
        )}

        {/* Logs */}
        {logs.length > 0 && (
          <div className="bg-gray-800 rounded p-4">
            <h3 className="text-sm font-semibold mb-2">Log</h3>
            <div className="text-xs font-mono text-gray-400 space-y-1 max-h-48 overflow-y-auto">
              {logs.map((log, i) => (
                <div key={i}>{log}</div>
              ))}
            </div>
          </div>
        )}
       </div>
    </div>
  )
}

export default App
