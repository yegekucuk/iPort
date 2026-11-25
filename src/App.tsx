import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { StatusBar } from "./components/StatusBar";
import { ToastStack } from "./components/ToastStack";
import { TopBar } from "./components/TopBar";
import { PortInspector } from "./components/table/PortInspector";
import { PortTable } from "./components/table/PortTable";
import { useKeyboardNav } from "./hooks/useKeyboardNav";
import { usePortStream } from "./hooks/usePortStream";
import {
  copyToClipboard,
  killProcess,
  openInBrowser,
  openInIde,
  revealInFolder,
  stopProcess,
} from "./lib/tauri";
import { usePortStore } from "./store/usePortStore";
import type { ActionResult, PortEntry } from "./types/port";
import { filterEntries } from "./utils/filter";
import { buildPortUrl } from "./utils/format";

export function App() {
  const {
    snapshot,
    isLoading,
    isRefreshing,
    error,
    previewMode,
    triggerRefresh,
  } = usePortStream();

  const {
    activeChip,
    setActiveChip,
    searchQuery,
    setSearchQuery,
    selectedId,
    setSelectedId,
    theme,
    cycleTheme,
    preferredIde,
    setPreferredIde,
    pendingKillId,
    armPendingKill,
    busyPids,
    markPidBusy,
    toasts,
    pushToast,
    dismissToast,
  } = usePortStore();

  const searchInputRef = useRef<HTMLInputElement | null>(null);
  const orderedEntriesRef = useRef<PortEntry[]>([]);
  const [, setTick] = useState(0);

  // Keep relative timestamp fresh in StatusBar
  useEffect(() => {
    const id = window.setInterval(() => setTick((t) => t + 1), 1000);
    return () => window.clearInterval(id);
  }, []);

  const visibleEntries = useMemo(
    () => filterEntries(snapshot.entries, activeChip, searchQuery),
    [snapshot.entries, activeChip, searchQuery],
  );

  const selectedEntry = useMemo(
    () => visibleEntries.find((e) => e.id === selectedId) ?? null,
    [visibleEntries, selectedId],
  );

  const handleActionResult = useCallback(
    (result: ActionResult, refreshAfter = false) => {
      if (result.ok) {
        pushToast("success", result.message);
        if (refreshAfter) {
          window.setTimeout(() => {
            void triggerRefresh();
          }, 250);
        }
      } else if (result.requiresElevation) {
        pushToast("warning", result.message);
      } else {
        pushToast("error", result.message);
      }
    },
    [pushToast, triggerRefresh],
  );

  const handleOpenBrowser = useCallback(
    async (entry: PortEntry) => {
      const res = await openInBrowser(entry.port, entry.bindAddress);
      handleActionResult(res);
    },
    [handleActionResult],
  );

  const handleCopyText = useCallback(
    async (text: string, label: string) => {
      const ok = await copyToClipboard(text);
      if (ok) {
        pushToast("success", `Copied ${label}: ${text}`);
      } else {
        pushToast("error", `Failed to copy ${label}`);
      }
    },
    [pushToast],
  );

  const handleCopyUrl = useCallback(
    (entry: PortEntry) => {
      const url = buildPortUrl(entry.port, entry.bindAddress);
      void handleCopyText(url, "URL");
    },
    [handleCopyText],
  );

  const handleRevealFolder = useCallback(
    async (path: string) => {
      const res = await revealInFolder(path);
      handleActionResult(res);
    },
    [handleActionResult],
  );

  const handleOpenIde = useCallback(
    async (entry: PortEntry) => {
      if (!entry.cwd) return;
      const res = await openInIde(entry.cwd, preferredIde);
      handleActionResult(res);
    },
    [handleActionResult, preferredIde],
  );

  const handleStopProcess = useCallback(
    async (entry: PortEntry) => {
      if (entry.pid === null) return;
      markPidBusy(entry.pid, true);
      try {
        const res = await stopProcess(entry.pid);
        handleActionResult(res, true);
      } finally {
        markPidBusy(entry.pid, false);
      }
    },
    [handleActionResult, markPidBusy],
  );

  const handleForceKillClick = useCallback(
    async (entry: PortEntry) => {
      if (entry.pid === null) return;
      if (pendingKillId !== entry.id) {
        armPendingKill(entry.id);
        pushToast("warning", `Click Kill again to send SIGKILL to PID ${entry.pid}`);
        return;
      }

      armPendingKill(null);
      markPidBusy(entry.pid, true);
      try {
        const res = await killProcess(entry.pid);
        handleActionResult(res, true);
      } finally {
        markPidBusy(entry.pid, false);
      }
    },
    [armPendingKill, handleActionResult, markPidBusy, pendingKillId, pushToast],
  );

  const handleClearFilters = useCallback(() => {
    setActiveChip("all");
    setSearchQuery("");
  }, [setActiveChip, setSearchQuery]);

  const handleToggleSystem = useCallback(() => {
    setActiveChip((prev) => (prev === "system" ? "all" : "system"));
  }, [setActiveChip]);

  useKeyboardNav({
    entries: visibleEntries,
    orderedEntriesRef,
    selectedId,
    searchQuery,
    searchInputRef,
    onSelect: setSelectedId,
    onClearSearch: () => setSearchQuery(""),
    onOpenBrowser: (entry) => {
      void handleOpenBrowser(entry);
    },
    onCopyUrl: handleCopyUrl,
    onStopProcess: (entry) => {
      void handleStopProcess(entry);
    },
    onRefresh: () => {
      void triggerRefresh();
    },
  });

  return (
    <div className="h-screen w-screen flex flex-col overflow-hidden bg-[var(--window-bg-color)] text-[var(--window-fg-color)]">
      <TopBar
        searchInputRef={searchInputRef}
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        activeChip={activeChip}
        onChipChange={setActiveChip}
        counts={snapshot.counts}
        preferredIde={preferredIde}
        onIdeChange={setPreferredIde}
        theme={theme}
        onCycleTheme={cycleTheme}
        isRefreshing={isRefreshing}
        onRefresh={() => {
          void triggerRefresh();
        }}
        previewMode={previewMode}
      />

      <PortTable
        entries={visibleEntries}
        orderedEntriesRef={orderedEntriesRef}
        isLoading={isLoading}
        selectedId={selectedId}
        activeChip={activeChip}
        searchQuery={searchQuery}
        onSelect={setSelectedId}
        onClearFilters={handleClearFilters}
        pendingKillId={pendingKillId}
        busyPids={busyPids}
        onOpenBrowser={(entry) => {
          void handleOpenBrowser(entry);
        }}
        onCopyUrl={handleCopyUrl}
        onOpenIde={(entry) => {
          void handleOpenIde(entry);
        }}
        onStopProcess={(entry) => {
          void handleStopProcess(entry);
        }}
        onForceKillClick={(entry) => {
          void handleForceKillClick(entry);
        }}
      />

      {selectedEntry && (
        <PortInspector
          entry={selectedEntry}
          pendingKillId={pendingKillId}
          busyPids={busyPids}
          onClose={() => setSelectedId(null)}
          onCopyText={(text, label) => {
            void handleCopyText(text, label);
          }}
          onOpenBrowser={(entry) => {
            void handleOpenBrowser(entry);
          }}
          onRevealFolder={(path) => {
            void handleRevealFolder(path);
          }}
          onOpenIde={(entry) => {
            void handleOpenIde(entry);
          }}
          onStopProcess={(entry) => {
            void handleStopProcess(entry);
          }}
          onForceKillClick={(entry) => {
            void handleForceKillClick(entry);
          }}
        />
      )}

      <StatusBar
        counts={snapshot.counts}
        activeChip={activeChip}
        timestampMs={snapshot.timestampMs}
        error={error}
        onToggleSystem={handleToggleSystem}
      />

      <ToastStack toasts={toasts} onDismiss={dismissToast} />
    </div>
  );
}

export default App;
