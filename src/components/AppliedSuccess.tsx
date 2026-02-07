import { formatSize } from "../utils/bytes";

interface AppliedSuccessProps {
  source: File | null;
  outputSize: number;
}

function AppliedSuccess({ source, outputSize }: AppliedSuccessProps) {
  return (
    <>
      <div className="bg-slate-950/50 rounded-2xl p-4 md:p-5 border border-slate-800 animate-in fade-in zoom-in-95 duration-500 delay-200 fill-mode-both">
        <p className="text-[8px] md:text-[9px] font-black text-slate-500 uppercase tracking-widest mb-1">
          Base File
        </p>
        <p className="text-sm md:text-base font-mono font-bold text-slate-300">
          {formatSize(source?.size ?? 0)}
        </p>
      </div>
      <div className="bg-slate-950/50 rounded-2xl p-5 border border-slate-800 animate-in fade-in zoom-in-95 duration-500 delay-300 fill-mode-both">
        <p className="text-[8px] md:text-[9px] font-black text-slate-500 uppercase tracking-widest mb-1">
          Merged Result
        </p>
        <p className="text-sm md:text-base font-mono font-bold text-indigo-400">
          {formatSize(outputSize)}
        </p>
      </div>
    </>
  );
}

export { AppliedSuccess };
