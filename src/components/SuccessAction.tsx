import type React from "react";
import { IconCheck, IconDownload } from "./Icons";

interface SuccessActionProps extends React.PropsWithChildren {
  onDownload: () => void;
  onReset: () => void;
}

function SuccessAction({ children, onDownload, onReset }: SuccessActionProps) {
  return (
    <div
      key="done"
      className="w-full animate-in fade-in slide-in-from-bottom-6 duration-700 ease-out-quint fill-mode-both"
    >
      <div className="flex flex-col lg:flex-row items-center justify-between gap-6 md:gap-10">
        {/* Result Stats */}
        <div className="flex-1 space-y-3 md:space-y-4 w-full">
          <div className="flex items-center gap-3 text-emerald-500 mb-2 md:mb-4 animate-in fade-in slide-in-from-left-4 duration-500 delay-100 fill-mode-both">
            <div className="w-8 h-8 md:w-10 md:h-10 rounded-xl bg-emerald-500/10 flex items-center justify-center border border-emerald-500/20 shrink-0">
              <IconCheck className="w-5 h-5 md:w-6 md:h-6" />
            </div>
            <h3 className="text-lg md:text-xl font-black uppercase tracking-tight text-white">
              Success
            </h3>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 md:gap-4">
            {children}
          </div>
        </div>

        {/* Final Actions */}
        <div className="flex flex-col gap-3 md:gap-4 w-full md:w-auto md:min-w-[220px] animate-in fade-in slide-in-from-right-4 duration-500 delay-500 fill-mode-both">
          <button
            onClick={onDownload}
            className="w-full px-6 md:px-8 py-4 md:py-5 bg-emerald-600 hover:bg-emerald-500 text-white rounded-2xl font-black uppercase tracking-widest text-[10px] md:text-xs transition-all flex items-center justify-center gap-3 shadow-lg shadow-emerald-600/20 active:scale-95"
          >
            <IconDownload className="w-4 h-4 md:w-5 md:h-5" />
            Save File
          </button>
          <button
            onClick={onReset}
            className="w-full px-6 md:px-8 py-3 md:py-4 bg-slate-800 hover:bg-slate-700 text-slate-400 rounded-2xl font-bold text-[10px] md:text-xs transition-all border border-slate-700/50"
          >
            Repeat Process
          </button>
        </div>
      </div>
    </div>
  );
}

export { SuccessAction };
