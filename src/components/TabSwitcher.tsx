import type { Mode } from "./Header";

interface TabSwitcherProps {
  mode: Mode;
  setMode: (mode: Mode) => void;
}

function TabSwitcher({ mode, setMode }: TabSwitcherProps) {
  return (
    <div className="relative flex bg-slate-900/50 p-1 rounded-xl border border-slate-800 self-stretch md:self-center">
      <div
        className="absolute top-1 bottom-1 left-1 transition-all duration-300 ease-out bg-indigo-600 rounded-lg shadow-lg shadow-indigo-600/20"
        style={{
          width: "calc(50% - 4px)",
          transform: `translateX(${mode === "create" ? "0%" : "100%"})`,
        }}
      />
      <button
        onClick={() => setMode("create")}
        className={`relative z-10 flex-1 px-4 md:px-6 py-2 rounded-lg text-xs md:text-sm font-semibold transition-colors duration-300 whitespace-nowrap ${
          mode === "create"
            ? "text-white"
            : "text-slate-400 hover:text-slate-200"
        }`}
      >
        Create Patch
      </button>
      <button
        onClick={() => setMode("apply")}
        className={`relative z-10 flex-1 px-4 md:px-6 py-2 rounded-lg text-xs md:text-sm font-semibold transition-colors duration-300 whitespace-nowrap ${
          mode === "apply"
            ? "text-white"
            : "text-slate-400 hover:text-slate-200"
        }`}
      >
        Apply Patch
      </button>
    </div>
  );
}

export { TabSwitcher };
