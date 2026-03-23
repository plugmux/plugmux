import logoWhite from "@assets/logo-white.svg";

export function Titlebar() {
  return (
    <div
      data-tauri-drag-region
      className="flex h-12 shrink-0 items-center justify-between bg-primary pl-20 pr-3 select-none"
    >
      <img src={logoWhite} alt="plugmux desktop" className="h-6" />
      <button className="rounded bg-primary-foreground px-3 py-1 text-xs font-medium text-primary hover:bg-primary-foreground/90">
        Sign in
      </button>
    </div>
  );
}
