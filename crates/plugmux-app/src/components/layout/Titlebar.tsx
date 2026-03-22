import logoWhite from "@/assets/logo-white.svg";

export function Titlebar() {
  return (
    <div
      data-tauri-drag-region
      className="flex h-12 shrink-0 items-center justify-between bg-[#7A67D1] pl-20 pr-3 select-none"
    >
      <img src={logoWhite} alt="plugmux desktop" className="h-6" />
      <button className="rounded bg-white px-3 py-1 text-xs font-medium text-[#7A67D1] hover:bg-white/90">
        Sign in
      </button>
    </div>
  );
}
