import { useState, useRef, useCallback, useEffect } from "react";
import { PatchlyWorker, downloadFromOpfs } from "./workers";
import { formatSize } from "./utils/bytes";
import { Background } from "./components/Background";
import { Header, type Mode } from "./components/Header";
import { FileUpload } from "./components/FileUpload";
import {
  IconFilePlus,
  IconFileUp,
  IconSwap,
  IconZap,
} from "./components/Icons";
import { IdleAction } from "./components/IdleAction";
import { ProcessingAction } from "./components/ProcessingAction";
import { SuccessAction } from "./components/SuccessAction";
import { ErrorAction } from "./components/ErrorAction";
import { CreatedSuccess } from "./components/CreatedSuccess";
import { AppliedSuccess } from "./components/AppliedSuccess";
import { Footer } from "./components/Footer";

type Status = "idle" | "processing" | "success" | "error";

function App() {
  const [mode, setMode] = useState<Mode>("create");
  const [status, setStatus] = useState<Status>("idle");
  const [progress, setProgress] = useState(0);
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
    console.log(`[${new Date().toLocaleTimeString()}] ${msg}`);
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
        addLog(`Done! Output: ${name} (${formatSize(size)})`);
      },
      // on error
      (message) => {
        setStatus("error");
        setError(message);
        addLog(`ERROR: ${message}`);
      },
      // on identical
      () => {
        setStatus("error");
        setError("Source and target file are identical. No patch needed");
        addLog("Files are identical - no patch created");
      },
    );

    worker.waitReady().then(() => {
      addLog("WASM module ready");
    });

    workerRef.current = worker;

    return () => worker.terminate();
  }, [addLog]);

  const handleStart = () => {
    if (mode === "create") {
      if (!sourceFile || !targetFile) {
        setError("Please select both source and target files");
        return;
      }

      setStatus("processing");
      setProgress(0);
      setError(null);
      addLog(`Creating patch: ${sourceFile.name} -> ${targetFile.name}`);
      workerRef.current?.createPatch(
        sourceFile,
        targetFile,
        `${targetFile.name}.patch`,
      );
    } else {
      if (!sourceFile || !patchFile) {
        setError("Please select both source and patch files");
        return;
      }

      setStatus("processing");
      setProgress(0);
      setError(null);
      addLog(`Applying patch to: ${sourceFile.name}`);
      const outName = patchFile.name.replace(/\.patch$/, "") || "output";
      workerRef.current?.applyPatch(sourceFile, patchFile, outName);
    }
  };

  const handleDownload = async () => {
    if (!outputName) return;

    await downloadFromOpfs(outputName);
    addLog(`Downloaded: ${outputName}`);
  };

  const handleReset = () => {
    setStatus("idle");
    setProgress(0);
    setError(null);
    setOutputName(null);
    setOutputSize(0);
    setSourceFile(null);
    setTargetFile(null);
    setPatchFile(null);
  };

  return (
    <div className="min-h-screen bg-slate-950 text-slate-200 font-sans selection:bg-indigo-500/30 overflow-x-hidden">
      {/* Decorative Background */}
      <Background />
      <main className="relative z-10 max-w-4xl mx-auto px-4 md:px-6 py-8 md:py-12">
        {/* Header */}
        <Header mode={mode} setMode={setMode} />

        {/* Main Workspace */}
        <div
          key={mode}
          className="animate-in fade-in slide-in-from-bottom-4 duration-500"
        >
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 md:gap-6 mb-8">
            {/* Source File */}
            <FileUpload
              title="Base File"
              icon={<IconFileUp />}
              label="Select base file"
              file={sourceFile}
              onChange={setSourceFile}
            />

            {/* Source or Patch File */}
            <FileUpload
              title={mode === "create" ? "Target File" : "Patch File"}
              icon={mode === "create" ? <IconFilePlus /> : <IconZap />}
              label={
                mode === "create" ? "Select target file" : "Select .patch file"
              }
              file={mode === "create" ? targetFile : patchFile}
              onChange={mode === "create" ? setTargetFile : setPatchFile}
            />
          </div>

          {/* Action Hub */}
          <div className="min-h-[280px] md:min-h-[320px] bg-slate-900/80 border border-slate-800/50 rounded-3xl p-6 md:p-8 backdrop-blur-xl shadow-2xl flex items-center justify-center relative overflow-hidden transition-all duration-500">
            {status === "idle" && (
              <IdleAction
                disabled={
                  !sourceFile || (mode === "create" ? !targetFile : !patchFile)
                }
                label={mode === "create" ? "Generate Patch" : "Apply Patch"}
                icon={mode == "create" ? <IconSwap /> : <IconZap />}
                onClick={handleStart}
              />
            )}

            {status === "processing" && (
              <ProcessingAction progress={progress} />
            )}

            {status === "success" && (
              <SuccessAction onDownload={handleDownload} onReset={handleReset}>
                {mode === "create" ? (
                  <CreatedSuccess target={targetFile} outputSize={outputSize} />
                ) : (
                  <AppliedSuccess source={sourceFile} outputSize={outputSize} />
                )}
              </SuccessAction>
            )}

            {status === "error" && (
              <ErrorAction message={error} onReset={handleReset} />
            )}
          </div>

          {/* Footer */}
          <Footer />
        </div>
      </main>
    </div>
  );
}

export default App;
