import { IconShield, IconSwap, IconZap } from "./Icons";

function Footer() {
  return (
    <div className="mt-8 md:mt-12 grid grid-cols-1 md:grid-cols-3 gap-4 md:gap-6 animate-in fade-in duration-1000 delay-300">
      <div className="flex items-start gap-4 p-5 md:p-6 rounded-3xl md:rounded-4xl bg-slate-900/20 border border-slate-800/40 hover:bg-slate-900/40 transition-all group">
        <div className="w-9 h-9 md:w-10 md:h-10 rounded-xl bg-slate-800/50 flex items-center justify-center shrink-0 group-hover:bg-indigo-500/10 transition-colors">
          <IconShield className="w-4 h-4 md:w-5 md:h-5 text-slate-500 group-hover:text-indigo-400" />
        </div>
        <div>
          <h4 className="text-[9px] md:text-[10px] font-black uppercase tracking-widest text-slate-400 mb-0.5 md:mb-1">
            Security
          </h4>
          <p className="text-[11px] md:text-xs text-slate-600 leading-tight">
            Data stays in your browser's memory
          </p>
        </div>
      </div>
      <div className="flex items-start gap-4 p-5 md:p-6 rounded-3xl md:rounded-4xl bg-slate-900/20 border border-slate-800/40 hover:bg-slate-900/40 transition-all group">
        <div className="w-9 h-9 md:w-10 md:h-10 rounded-xl bg-slate-800/50 flex items-center justify-center shrink-0 group-hover:bg-indigo-500/10 transition-colors">
          <IconZap className="w-4 h-4 md:w-5 md:h-5 text-slate-500 group-hover:text-indigo-400" />
        </div>
        <div>
          <h4 className="text-[9px] md:text-[10px] font-black uppercase tracking-widest text-slate-400 mb-0.5 md:mb-1">
            Technology
          </h4>
          <p className="text-[11px] md:text-xs text-slate-600 leading-tight">
            Rolling Hash based diffing engine
          </p>
        </div>
      </div>
      <div className="flex items-start gap-4 p-5 md:p-6 rounded-3xl md:rounded-4xl bg-slate-900/20 border border-slate-800/40 hover:bg-slate-900/40 transition-all group">
        <div className="w-9 h-9 md:w-10 md:h-10 rounded-xl bg-slate-800/50 flex items-center justify-center shrink-0 group-hover:bg-indigo-500/10 transition-colors">
          <IconSwap className="w-4 h-4 md:w-5 md:h-5 text-slate-500 group-hover:text-indigo-400" />
        </div>
        <div>
          <h4 className="text-[9px] md:text-[10px] font-black uppercase tracking-widest text-slate-400 mb-0.5 md:mb-1">
            Verification
          </h4>
          <p className="text-[11px] md:text-xs text-slate-600 leading-tight">
            Automatic checksum for every block
          </p>
        </div>
      </div>
    </div>
  );
}

export { Footer };
