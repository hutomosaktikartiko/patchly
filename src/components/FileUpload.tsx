import { useRef, type ReactNode } from "react";
import { formatSize } from "../utils/bytes";

interface FileUploadProps {
  title: string;
  icon: ReactNode;
  label: string;
  description?: string;
  file: File | null;
  accept?: string;
  onChange: (file: File | null) => void;
}

function FileUpload({
  title,
  icon,
  label,
  description = "Click or drag file here",
  file,
  accept,
  onChange,
}: FileUploadProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  const handleClear = (e: React.MouseEvent) => {
    e.stopPropagation();
    e.preventDefault();
    // Reset input value so the same file can be re-selected
    if (inputRef.current) {
      inputRef.current.value = "";
    }
    onChange(null);
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0] || null;
    onChange(selectedFile);
    // Reset input value after selection so re-selecting same file works
    e.target.value = "";
  };

  return (
    <div className="group">
      <label className="block text-[10px] font-black uppercase tracking-widest text-slate-500 mb-2 md:mb-3 ml-1">
        {title}
      </label>
      <div
        className={`h-56 md:h-64 relative border-2 border-dashed rounded-3xl p-6 md:p-8 flex flex-col items-center justify-center transition-all duration-300 transform ${
          file
            ? "border-indigo-500/50 bg-indigo-500/5"
            : "border-slate-800 bg-slate-900/40 hover:border-slate-700"
        }`}
      >
        <input
          ref={inputRef}
          type="file"
          accept={accept}
          onChange={handleChange}
          className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
        />

        {/* Clear button */}
        {file && (
          <button
            onClick={handleClear}
            className="absolute top-3 right-3 w-7 h-7 md:w-8 md:h-8 rounded-lg bg-slate-800/80 hover:bg-red-500/20 border border-slate-700 hover:border-red-500/50 flex items-center justify-center transition-all z-10 group/clear"
            title="Remove file"
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              className="text-slate-400 group-hover/clear:text-red-400 transition-colors"
            >
              <path
                d="M18 6L6 18M6 6l12 12"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </button>
        )}

        <div
          className={`w-12 h-12 md:w-14 md:h-14 mb-3 md:mb-4 rounded-2xl flex items-center justify-center transition-all ${file ? "bg-indigo-500/20 text-indigo-400 shadow-[0_0_20_rgba(99,102,241,0.15)]" : "bg-slate-800/50 text-slate-600"}`}
        >
          {icon}
        </div>
        <div className="max-w-full px-2 text-center">
          <p className="text-xs md:text-sm font-bold text-slate-200 truncate">
            {file ? file.name : label}
          </p>
          {file ? (
            <span className="inline-block text-[9px] md:text-[10px] font-mono text-slate-500 mt-2 px-2 py-0.5 bg-slate-800/50 rounded uppercase tracking-tight">
              {formatSize(file.size)}
            </span>
          ) : (
            <span className="text-[9px] md:text-[10px] text-slate-600 mt-2 block">
              {description}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

export { FileUpload };
