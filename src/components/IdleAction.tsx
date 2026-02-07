import type { ReactNode } from "react";

interface IdleActionProps {
  disabled: boolean;
  label: string;
  icon: ReactNode;
  onClick: () => void;
}

function IdleAction({ disabled, label, icon, onClick }: IdleActionProps) {
  return (
    <div
      key="idle"
      className="flex flex-col items-center animate-in fade-in zoom-in-95 duration-500 fill-mode-both ease-out-quint"
    >
      <button
        disabled={disabled}
        onClick={onClick}
        className="group relative px-8 md:px-12 py-4 md:py-5 bg-indigo-600 disabled:bg-slate-800 disabled:text-slate-600 text-white rounded-2xl font-black uppercase tracking-widest text-[10px] md:text-xs transition-all hover:scale-[1.02] active:scale-[0.98] shadow-lg shadow-indigo-600/20 disabled:shadow-none flex items-center gap-3 md:gap-4 overflow-hidden"
      >
        {icon}
        <span>{label}</span>
        <div className="absolute inset-0 bg-linear-to-r from-transparent via-white/10 to-transparent -translate-x-full group-hover:animate-[shimmer_1.5s_infinite]" />
      </button>
      {disabled && (
        <p className="mt-4 text-[9px] md:text-[10px] text-slate-500 font-medium uppercase tracking-widest animate-in fade-in duration-500">
          Select files to begin
        </p>
      )}
    </div>
  );
}

export { IdleAction };
