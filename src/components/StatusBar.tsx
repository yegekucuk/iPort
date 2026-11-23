import type { FilterChip, SnapshotCounts } from "../types/port";
import { formatRelativeAge } from "../utils/format";

interface StatusBarProps {
  counts: SnapshotCounts;
  activeChip: FilterChip;
  timestampMs: number;
  error: string | null;
  onToggleSystem: () => void;
}

export function StatusBar({
  counts,
  activeChip,
  timestampMs,
  error,
  onToggleSystem,
}: StatusBarProps) {
  const showingSystem = activeChip === "system";

  return (
    <footer className="shrink-0 h-[30px] px-3.5 bg-[var(--headerbar-bg-color)] border-t border-[var(--border-color)] flex items-center justify-between gap-4 text-[11.5px] text-[var(--muted-fg-color)]">
      <div className="flex items-center gap-2.5 min-w-0">
        <span className="inline-flex items-center gap-1.5 text-[var(--window-fg-color)]">
          <span
            className="w-2 h-2 rounded-full"
            style={{
              backgroundColor: error
                ? "var(--destructive-bg-color)"
                : "var(--success-color)",
            }}
            aria-hidden="true"
          />
          <span className="font-mono-tabular">
            {counts.totalVisible} active{" "}
            {counts.totalVisible === 1 ? "port" : "ports"}
          </span>
        </span>

        <span aria-hidden="true" className="opacity-40">
          ·
        </span>

        <button
          type="button"
          onClick={onToggleSystem}
          className="hover:text-[var(--window-fg-color)] underline-offset-2 hover:underline cursor-pointer"
        >
          {showingSystem
            ? `Showing ${counts.system} OS ports (Return to All)`
            : `${counts.system} OS ports hidden`}
        </button>

        {error && (
          <span className="text-[var(--destructive-bg-color)] truncate">
            · {error}
          </span>
        )}
      </div>

      <div className="flex items-center gap-3 shrink-0">
        <span className="hidden md:inline text-[var(--subtle-fg-color)]">
          <kbd className="font-mono-tabular">/</kbd> Search ·{" "}
          <kbd className="font-mono-tabular">↑↓</kbd> Select ·{" "}
          <kbd className="font-mono-tabular">o</kbd> Open ·{" "}
          <kbd className="font-mono-tabular">c</kbd> Copy ·{" "}
          <kbd className="font-mono-tabular">x</kbd> Stop
        </span>
        <span className="font-mono-tabular text-[var(--muted-fg-color)]">
          Updated {formatRelativeAge(timestampMs)}
        </span>
      </div>
    </footer>
  );
}
