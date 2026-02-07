import { IconGithub, IconShield } from "./Icons";
import { TabSwitcher } from "./TabSwitcher";

type Mode = "create" | "apply";
const GITHUB_URL = "https://github.com/hutomosaktikartiko/patchly";

interface HeaderProps {
  mode: Mode;
  setMode: (mode: Mode) => void;
}

function Header({ mode, setMode }: HeaderProps) {
  return (
    <header className="flex flex-col md:flex-row md:items-center justify-between gap-6 mb-8 md:mb-12 animate-in fade-in slide-in-from-top-4 duration-700">
      <div className="flex items-center gap-4">
        <img
          src="/logo.png"
          alt=""
          className="w-10 h-10 md:w-12 md:h-12 text-white rounded-xl flex items-center justify-center shadow-lg shadow-indigo-500/20 shrink-0"
        />
        <div>
          <h1 className="text-2xl md:text-3xl font-bold bg-clip-text text-transparent bg-linear-to-r from-white via-white to-slate-500 leading-tight">
            Patchly
          </h1>
          <div className="flex flex-wrap items-center gap-2 md:gap-3">
            <p className="text-slate-500 text-xs md:sm font-medium flex items-center gap-1.5 whitespace-nowrap">
              <IconShield className="w-3.5 h-3.5 md:w-4 md:h-4 text-emerald-500" />
              Client-side Binary Diff & Patch
            </p>
            <div className="hidden sm:block w-1 h-1 bg-slate-800 rounded-full" />
            <a
              href={GITHUB_URL}
              target="_blank"
              rel="noopener noreferrer"
              className="text-slate-500 hover:text-indigo-400 text-[9px] md:text-[10px] font-black uppercase tracking-widest transition-colors flex items-center gap-1.5"
            >
              <IconGithub className="w-3 h-3" />
              View Source
            </a>
          </div>
        </div>
      </div>

      {/* Tab Switcher */}
      <TabSwitcher mode={mode} setMode={setMode} />
    </header>
  );
}

export { type Mode, Header };
