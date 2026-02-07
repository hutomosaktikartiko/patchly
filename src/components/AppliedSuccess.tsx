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
          Source File
        </p>
        <p className="text-sm md:text-base font-mono font-bold text-slate-300">
          {formatSize(source?.size ?? 0)}
        </p>
      </div>
      <div className="bg-slate-950/50 rounded-2xl p-5 border border-slate-800 animate-in fade-in zoom-in-95 duration-500 delay-300 fill-mode-both">
        <p className="text-[8px] md:text-[9px] font-black text-slate-500 uppercase tracking-widest mb-1">
          Patched Output
        </p>
        <p className="text-sm md:text-base font-mono font-bold text-indigo-400">
          {formatSize(outputSize)}
        </p>
      </div>
      <div className="col-span-1 sm:col-span-2 bg-emerald-500/5 rounded-2xl p-4 md:p-5 border border-emerald-500/20 flex items-center gap-3 md:gap-4 shadow-inner animate-in fade-in slide-in-from-bottom-4 duration-500 delay-400 fill-mode-both">
        <div className="w-1.5 h-1.5 md:w-2 md:h-2 rounded-full bg-emerald-400 animate-pulse shadow-[0_0_8px_rgba(52,211,153,0.6)] shrink-0" />
        <span className="text-[9px] md:text-xs font-bold text-emerald-400/80 uppercase tracking-widest leading-tight">
          Byte-for-Byte Verified
        </span>
      </div>
    </>
  );
}

export { AppliedSuccess };
