import type { RefObject } from "react";
import {
  Monitor,
  Moon,
  RefreshCw,
  Search,
  Sun,
  X,
} from "lucide-react";
import type { FilterChip, SnapshotCounts, ThemeMode } from "../types/port";

interface TopBarProps {
  searchInputRef: RefObject<HTMLInputElement | null>;
  searchQuery: string;
  onSearchChange: (q: string) => void;
  activeChip: FilterChip;
  onChipChange: (chip: FilterChip) => void;
  counts: SnapshotCounts;
  theme: ThemeMode;
  onCycleTheme: () => void;
  isRefreshing: boolean;
  onRefresh: () => void;
  previewMode: boolean;
}

export function TopBar({
  searchInputRef,
  searchQuery,
  onSearchChange,
  activeChip,
  onChipChange,
  counts,
  theme,
  onCycleTheme,
  isRefreshing,
  onRefresh,
  previewMode,
}: TopBarProps) {
  const chips: Array<{ id: FilterChip; label: string; count: number }> = [
    { id: "all", label: "All", count: counts.totalVisible },
    { id: "dev", label: "Dev", count: counts.dev },
    { id: "docker", label: "Docker", count: counts.docker },
    { id: "service", label: "Services", count: counts.services },
    { id: "tunnel", label: "Tunnels", count: counts.tunnels },
    { id: "third_party", label: "3rd-Party", count: counts.thirdParty },
    { id: "system", label: "System", count: counts.system },
  ];

  const themeLabel =
    theme === "system" ? "System" : theme === "light" ? "Light" : "Dark";

  return (
    <header className="shrink-0 bg-[var(--headerbar-bg-color)] border-b border-[var(--border-color)]">
      {/* Primary Toolbar Row */}
      <div className="px-3.5 py-2 flex items-center gap-3">
        <div className="flex items-center gap-2 shrink-0">
          <span
            aria-hidden="true"
            className="w-6 h-6 rounded-[7px] bg-[var(--accent-bg-color)] text-[var(--accent-fg-color)] flex items-center justify-center shadow-xs"
          >
            <svg viewBox="0 0 24 24" className="w-3.5 h-3.5" fill="none" stroke="currentColor" strokeWidth="2.2">
              <circle cx="12" cy="12" r="7" />
              <circle cx="12" cy="12" r="2.2" fill="currentColor" stroke="none" />
            </svg>
          </span>
          <span className="font-semibold text-[14px] tracking-[-0.01em] text-[var(--window-fg-color)]">
            iPort
          </span>
          {previewMode && (
            <span className="text-[11px] px-1.5 py-0.5 rounded-[5px] bg-[var(--entry-bg-color)] text-[var(--muted-fg-color)] border border-[var(--border-color)]">
              Preview
            </span>
          )}
        </div>

        {/* GtkSearchEntry */}
        <div className="flex-1 max-w-xl mx-auto relative flex items-center">
          <Search
            className="w-3.5 h-3.5 text-[var(--muted-fg-color)] absolute left-2.5 pointer-events-none"
            aria-hidden="true"
          />
          <input
            ref={searchInputRef}
            type="text"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="Search port, process, framework, project, path…"
            aria-label="Search ports"
            className="w-full h-[32px] pl-8 pr-12 rounded-[8px] bg-[var(--entry-bg-color)] focus:bg-[var(--entry-focus-bg)] border border-[var(--border-color)] text-[13px] text-[var(--window-fg-color)] placeholder:text-[var(--subtle-fg-color)] transition-colors"
          />
          {searchQuery ? (
            <button
              type="button"
              onClick={() => onSearchChange("")}
              title="Clear search (Esc)"
              aria-label="Clear search"
              className="adw-flat-btn absolute right-1.5 w-6 h-6 flex items-center justify-center text-[var(--muted-fg-color)]"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          ) : (
            <kbd className="font-mono-tabular absolute right-2 px-1.5 py-0.5 text-[10.5px] leading-none rounded-[4px] border border-[var(--border-color)] text-[var(--subtle-fg-color)] pointer-events-none">
              /
            </kbd>
          )}
        </div>

        {/* GNOME Flat Toolbar Actions */}
        <div className="flex items-center gap-1.5 shrink-0">
          <button
            type="button"
            onClick={onRefresh}
            title="Refresh ports now (r)"
            aria-label="Refresh ports"
            className="adw-flat-btn h-[30px] w-[32px] flex items-center justify-center text-[var(--window-fg-color)]"
          >
            <RefreshCw
              className={`w-3.5 h-3.5 ${isRefreshing ? "animate-spin" : ""}`}
            />
          </button>

          <button
            type="button"
            onClick={onCycleTheme}
            title={`Theme: ${themeLabel} (click to switch)`}
            className="adw-flat-btn h-[30px] px-2.5 flex items-center gap-1.5 text-[12px] text-[var(--window-fg-color)]"
          >
            {theme === "system" ? (
              <Monitor className="w-3.5 h-3.5 text-[var(--muted-fg-color)]" />
            ) : theme === "light" ? (
              <Sun className="w-3.5 h-3.5 text-[var(--muted-fg-color)]" />
            ) : (
              <Moon className="w-3.5 h-3.5 text-[var(--muted-fg-color)]" />
            )}
            <span>{themeLabel}</span>
          </button>
        </div>
      </div>

      {/* Category Filter Row */}
      <nav
        aria-label="Port categories"
        className="px-3.5 pb-2 flex items-center gap-1 overflow-x-auto"
      >
        {chips.map((chip) => {
          const active = activeChip === chip.id;
          return (
            <button
              key={chip.id}
              type="button"
              onClick={() => onChipChange(chip.id)}
              aria-pressed={active}
              className={`h-[26px] px-2.5 rounded-[6px] text-[12px] flex items-center gap-1.5 shrink-0 border transition-colors cursor-pointer ${
                active
                  ? "bg-[var(--accent-soft-bg)] text-[var(--accent-bg-color)] border-[var(--accent-soft-border)] font-medium"
                  : "bg-transparent text-[var(--muted-fg-color)] border-transparent hover:bg-[var(--hover-bg-color)] hover:text-[var(--window-fg-color)]"
              }`}
            >
              <span>{chip.label}</span>
              <span
                className={`font-mono-tabular text-[11px] px-1 rounded-[4px] ${
                  active
                    ? "text-[var(--accent-bg-color)]"
                    : "text-[var(--subtle-fg-color)]"
                }`}
              >
                {chip.count}
              </span>
            </button>
          );
        })}
      </nav>
    </header>
  );
}
