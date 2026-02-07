import { IconAlert } from "./Icons";

interface ErrorActionProps {
  message: string | null;
  onReset: () => void;
}

function ErrorAction({ message, onReset }: ErrorActionProps) {
  return (
    <div
      key="error"
      className="flex flex-col items-center py-4 animate-in shake duration-500 fill-mode-both"
    >
      <div className="w-14 h-14 md:w-16 md:h-16 bg-red-500/10 rounded-2xl flex items-center justify-center mb-4 md:mb-6 border border-red-500/20">
        <IconAlert className="w-7 h-7 md:w-8 md:h-8 text-red-500" />
      </div>
      <p className="text-xs md:text-sm text-red-400 font-bold mb-6 px-4 md:px-10 text-center leading-relaxed">
        {message}
      </p>
      <button
        onClick={onReset}
        className="px-6 md:px-8 py-3 bg-red-500/10 text-red-400 rounded-xl text-[9px] md:text-[10px] font-black uppercase tracking-widest hover:bg-red-500/20 transition-all border border-red-500/20"
      >
        Back
      </button>
    </div>
  );
}

export { ErrorAction };
