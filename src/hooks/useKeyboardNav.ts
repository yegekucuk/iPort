import { useEffect, useRef, type RefObject } from "react";
import type { PortEntry } from "../types/port";

interface UseKeyboardNavOptions {
  entries: PortEntry[];
  orderedEntriesRef?: RefObject<PortEntry[]>;
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
  orderedEntriesRef,
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
  const suppressSlashInputRef = useRef(false);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.metaKey || event.ctrlKey || event.altKey) {
        suppressSlashInputRef.current = false;
        return;
      }

      const editing = isEditableElement(event.target);

      if (event.key === "/" && !editing) {
        event.preventDefault();
        suppressSlashInputRef.current = true;
        searchInputRef.current?.focus();
        searchInputRef.current?.select();
        return;
      }

      suppressSlashInputRef.current = false;

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

      const navEntries = orderedEntriesRef?.current ?? entries;

      if (event.key === "ArrowDown" || event.key === "j") {
        if (navEntries.length === 0) return;
        event.preventDefault();
        const idx = navEntries.findIndex((e) => e.id === selectedId);
        const nextIdx = idx < 0 ? 0 : Math.min(navEntries.length - 1, idx + 1);
        onSelect(navEntries[nextIdx].id);
        return;
      }

      if (event.key === "ArrowUp" || event.key === "k") {
        if (navEntries.length === 0) return;
        event.preventDefault();
        const idx = navEntries.findIndex((e) => e.id === selectedId);
        const prevIdx = idx < 0 ? 0 : Math.max(0, idx - 1);
        onSelect(navEntries[prevIdx].id);
        return;
      }

      if (event.key === "r") {
        event.preventDefault();
        onRefresh();
        return;
      }

      const selected = navEntries.find((e) => e.id === selectedId);
      if (!selected) {
        return;
      }

      if (
        event.key === "o" &&
        selected.protocol === "tcp" &&
        selected.availableActions.includes("open_browser")
      ) {
        event.preventDefault();
        onOpenBrowser(selected);
      } else if (
        event.key === "c" &&
        selected.availableActions.includes("copy_url")
      ) {
        event.preventDefault();
        onCopyUrl(selected);
      } else if (
        event.key === "x" &&
        selected.pid !== null &&
        selected.availableActions.includes("stop")
      ) {
        event.preventDefault();
        onStopProcess(selected);
      }
    };

    const handleKeyPress = (event: KeyboardEvent) => {
      if (suppressSlashInputRef.current && event.key === "/") {
        event.preventDefault();
      }
    };

    const handleBeforeInput = (event: Event) => {
      const inputEvent = event as InputEvent;
      if (suppressSlashInputRef.current && inputEvent.data === "/") {
        event.preventDefault();
        suppressSlashInputRef.current = false;
      }
    };

    const handleKeyUp = (event: KeyboardEvent) => {
      if (event.key === "/") {
        suppressSlashInputRef.current = false;
      }
    };

    const handleWindowBlur = () => {
      suppressSlashInputRef.current = false;
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keypress", handleKeyPress, true);
    window.addEventListener("beforeinput", handleBeforeInput, true);
    window.addEventListener("keyup", handleKeyUp, true);
    window.addEventListener("blur", handleWindowBlur);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keypress", handleKeyPress, true);
      window.removeEventListener("beforeinput", handleBeforeInput, true);
      window.removeEventListener("keyup", handleKeyUp, true);
      window.removeEventListener("blur", handleWindowBlur);
    };
  }, [
    entries,
    orderedEntriesRef,
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
