import { useEffect, type RefObject } from "react";
import type { PortEntry } from "../types/port";

interface UseKeyboardNavOptions {
  entries: PortEntry[];
  selectedId: string | null;
  searchQuery: string;
  searchInputRef: RefObject<HTMLInputElement | null>;
  onSelect: (id: string | null) => void;
  onClearSearch: () => void;
  onOpenBrowser: (entry: PortEntry) => void;
  onCopyUrl: (entry: PortEntry) => void;
  onStopProcess: (entry: PortEntry) => void;
  onRefresh: () => void;
}

function isEditableElement(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }
  const tag = target.tagName;
  return (
    tag === "INPUT" ||
    tag === "TEXTAREA" ||
    tag === "SELECT" ||
    target.isContentEditable
  );
}

export function useKeyboardNav({
  entries,
  selectedId,
  searchQuery,
  searchInputRef,
  onSelect,
  onClearSearch,
  onOpenBrowser,
  onCopyUrl,
  onStopProcess,
  onRefresh,
}: UseKeyboardNavOptions) {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.metaKey || event.ctrlKey || event.altKey) {
        return;
      }

      const editing = isEditableElement(event.target);

      if (event.key === "/" && !editing) {
        event.preventDefault();
        searchInputRef.current?.focus();
        searchInputRef.current?.select();
        return;
      }

      if (event.key === "Escape") {
        if (document.activeElement === searchInputRef.current) {
          if (searchQuery) {
            onClearSearch();
          } else {
            searchInputRef.current?.blur();
          }
          return;
        }
        if (selectedId) {
          onSelect(null);
        }
        return;
      }

      if (editing) {
        return;
      }

      if (event.key === "ArrowDown" || event.key === "j") {
        if (entries.length === 0) return;
        event.preventDefault();
        const idx = entries.findIndex((e) => e.id === selectedId);
        const nextIdx = idx < 0 ? 0 : Math.min(entries.length - 1, idx + 1);
        onSelect(entries[nextIdx].id);
        return;
      }

      if (event.key === "ArrowUp" || event.key === "k") {
        if (entries.length === 0) return;
        event.preventDefault();
        const idx = entries.findIndex((e) => e.id === selectedId);
        const prevIdx = idx < 0 ? 0 : Math.max(0, idx - 1);
        onSelect(entries[prevIdx].id);
        return;
      }

      if (event.key === "r") {
        event.preventDefault();
        onRefresh();
        return;
      }

      const selected = entries.find((e) => e.id === selectedId);
      if (!selected) {
        return;
      }

      if (event.key === "o" && selected.protocol === "tcp") {
        event.preventDefault();
        onOpenBrowser(selected);
      } else if (event.key === "c") {
        event.preventDefault();
        onCopyUrl(selected);
      } else if (event.key === "x" && selected.pid !== null) {
        event.preventDefault();
        onStopProcess(selected);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    entries,
    selectedId,
    searchQuery,
    searchInputRef,
    onSelect,
    onClearSearch,
    onOpenBrowser,
    onCopyUrl,
    onStopProcess,
    onRefresh,
  ]);
}
