import { IconRefresh } from "./Icons";

interface ProcessingActionProps {
  progress: number;
}

function ProcessingAction({ progress }: ProcessingActionProps) {
  return (
    <div
      key="processing"
      className="w-full max-w-lg flex flex-col items-center animate-in fade-in slide-in-from-bottom-8 duration-500 ease-out-quint fill-mode-both px-2"
    >
      <div className="w-full mb-6 md:mb-8">
        <div className="flex justify-between items-end mb-3 md:mb-4 px-1">
          <div className="flex items-center gap-2 md:gap-3 animate-in fade-in duration-700 delay-150 fill-mode-both">
            <IconRefresh className="w-4 h-4 md:w-5 md:h-5 text-indigo-400 animate-spin" />
            <span className="text-xs md:text-sm font-black uppercase tracking-tight text-white">
              Processing Binary...
            </span>
          </div>
          <span className="text-lg md:text-xl font-black font-mono text-indigo-400 animate-in fade-in zoom-in-90 duration-500 delay-200 fill-mode-both">
            {progress.toFixed(1)}%
          </span>
        </div>

        {/* Linear Progress Chunky */}
        <div className="relative h-10 md:h-12 w-full bg-slate-950 rounded-2xl overflow-hidden border border-slate-800 shadow-inner animate-in scale-x-95 fade-in duration-500 fill-mode-both">
          <div
            className="h-full bg-linear-to-r from-indigo-700 via-indigo-500 to-blue-500 transition-all duration-300 ease-linear relative"
            style={{ width: `${progress}%` }}
          >
            <div className="absolute inset-0 bg-linear-to-r from-transparent via-white/10 to-transparent animate-[shimmer_2s_infinite]" />
          </div>
          <div
            className="absolute inset-0 opacity-10 pointer-events-none"
            style={{
              backgroundImage:
                "linear-gradient(45deg, #fff 25%, transparent 25%, transparent 50%, #fff 50%, #fff 75%, transparent 75%, transparent)",
              backgroundSize: "20px 20px",
            }}
          />
        </div>
      </div>
      <p className="text-[8px] md:text-[10px] text-slate-500 font-bold uppercase tracking-[0.2em] animate-pulse text-center">
        Do not close the browser while processing
      </p>
    </div>
  );
}

export { ProcessingAction };
