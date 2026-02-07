import { formatSize } from "../utils/bytes";
import { IconChart } from "./Icons";

interface CreatedSuccessProps {
  target: File | null;
  outputSize: number;
}

function CreatedSuccess({ target, outputSize }: CreatedSuccessProps) {
  return (
    <>
      <div className="bg-slate-950/50 rounded-2xl p-4 md:p-5 border border-slate-800 animate-in fade-in zoom-in-95 duration-500 delay-200 fill-mode-both">
        <p className="text-[8px] md:text-[9px] font-black text-slate-500 uppercase tracking-widest mb-1">
          Target File Size
        </p>
        <p className="text-sm md:text-base font-mono font-bold text-slate-300">
          {formatSize(target?.size ?? 0)}
        </p>
      </div>
      <div className="bg-slate-950/50 rounded-2xl p-4 md:p-5 border border-slate-800 animate-in fade-in zoom-in-95 duration-500 delay-300 fill-mode-both">
        <p className="text-[8px] md:text-[9px] font-black text-slate-500 uppercase tracking-widest mb-1">
          Patch Size
        </p>
        <p className="text-sm md:text-base font-mono font-bold text-emerald-400">
          {formatSize(outputSize)}
        </p>
      </div>
      <div className="col-span-1 sm:col-span-2 bg-indigo-500/5 rounded-2xl p-4 md:p-5 border border-indigo-500/20 flex items-center justify-between shadow-inner animate-in fade-in slide-in-from-bottom-4 duration-500 delay-400 fill-mode-both">
        <div className="flex items-center gap-3">
          <div className="w-7 h-7 md:w-8 md:h-8 rounded-lg bg-indigo-500/10 flex items-center justify-center shrink-0">
            <IconChart className="w-3.5 h-3.5 md:w-4 md:h-4 text-indigo-400" />
          </div>
          <span className="text-[10px] md:text-xs font-bold text-slate-300 uppercase tracking-wider">
            Efficiency
          </span>
        </div>
        <span className="text-xl md:text-2xl font-black text-indigo-400">
          -
          {Math.max(0, 100 - (outputSize / (target?.size ?? 0)) * 100).toFixed(
            1,
          )}
          %
        </span>
      </div>
    </>
  );
}

export { CreatedSuccess };
