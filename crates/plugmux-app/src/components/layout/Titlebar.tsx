import logoWhite from "@assets/logo-white.svg";

export function Titlebar() {
  return (
    <div
      data-tauri-drag-region
      className="flex h-[63px] shrink-0 items-center justify-between bg-primary pl-20 pr-3 select-none"
    >
      <div className="flex items-center gap-0">
        <img src={logoWhite} alt="plugmux desktop" className="h-8" />
        <span className="rounded-full bg-white/20 px-2 py-0.5 text-[10px] font-semibold tracking-wide text-white">
          BETA
        </span>
      </div>
      <button className="rounded-xl bg-primary-foreground px-5 py-2 text-sm font-medium text-primary hover:bg-primary-foreground/90">
        Sign in
      </button>
    </div>
  );
}
