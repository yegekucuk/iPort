import { useCallback, useEffect, useRef, useState } from "react";
import { fetchSystemTheme } from "../lib/tauri";
import type { FilterChip, SystemThemeInfo, ThemeMode } from "../types/port";

export interface ToastItem {
  id: number;
  kind: "info" | "success" | "warning" | "error";
  message: string;
}

const THEME_STORAGE_KEY = "iport:theme";

function loadInitialTheme(): ThemeMode {
  try {
    const saved = localStorage.getItem(THEME_STORAGE_KEY);
    if (saved === "system" || saved === "dark" || saved === "light") {
      return saved;
    }
  } catch {
    // Ignore storage errors
  }
  return "system";
}

export function usePortStore() {
  const [activeChip, setActiveChip] = useState<FilterChip>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [theme, setThemeState] = useState<ThemeMode>(loadInitialTheme);
  const [systemTheme, setSystemTheme] = useState<SystemThemeInfo | null>(null);
  const [pendingKillId, setPendingKillId] = useState<string | null>(null);
  const [busyPids, setBusyPids] = useState<Set<number>>(new Set());
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  const toastCounter = useRef(0);
  const killResetTimer = useRef<number | null>(null);

  useEffect(() => {
    let active = true;
    fetchSystemTheme()
      .then((info) => {
        if (!active) return;
        setSystemTheme(info);
        const root = document.documentElement;
        root.style.setProperty("--gnome-accent-light", info.accentHex);
        root.style.setProperty("--gnome-accent-dark", info.accentDarkHex);
      })
      .catch(() => {
        // Keep CSS fallback accents
      });
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    const root = document.documentElement;
    if (theme === "dark" || theme === "light") {
      root.dataset.theme = theme;
    } else if (systemTheme?.colorScheme === "prefer-dark" || systemTheme?.gtkTheme.toLowerCase().endsWith("-dark")) {
      root.dataset.theme = "dark";
    } else if (systemTheme?.colorScheme === "prefer-light") {
      root.dataset.theme = "light";
    } else {
      delete root.dataset.theme;
    }

    try {
      localStorage.setItem(THEME_STORAGE_KEY, theme);
    } catch {
      // Ignore storage errors
    }
  }, [theme, systemTheme]);

  const setTheme = useCallback((next: ThemeMode) => {
    setThemeState(next);
  }, []);

  const cycleTheme = useCallback(() => {
    setThemeState((prev) => {
      if (prev === "system") return "light";
      if (prev === "light") return "dark";
      return "system";
    });
  }, []);

  const pushToast = useCallback((kind: ToastItem["kind"], message: string) => {
    toastCounter.current += 1;
    const id = toastCounter.current;
    setToasts((prev) => [...prev.slice(-2), { id, kind, message }]);
    window.setTimeout(() => {
      setToasts((prev) => prev.filter((item) => item.id !== id));
    }, 3400);
  }, []);

  const dismissToast = useCallback((id: number) => {
    setToasts((prev) => prev.filter((item) => item.id !== id));
  }, []);

  const armPendingKill = useCallback((entryId: string | null) => {
    if (killResetTimer.current !== null) {
      window.clearTimeout(killResetTimer.current);
      killResetTimer.current = null;
    }
    setPendingKillId(entryId);
    if (entryId) {
      killResetTimer.current = window.setTimeout(() => {
        setPendingKillId(null);
        killResetTimer.current = null;
      }, 3200);
    }
  }, []);

  const markPidBusy = useCallback((pid: number, busy: boolean) => {
    setBusyPids((prev) => {
      const next = new Set(prev);
      if (busy) {
        next.add(pid);
      } else {
        next.delete(pid);
      }
      return next;
    });
  }, []);

  return {
    activeChip,
    setActiveChip,
    searchQuery,
    setSearchQuery,
    selectedId,
    setSelectedId,
    theme,
    setTheme,
    cycleTheme,
    systemTheme,
    pendingKillId,
    armPendingKill,
    busyPids,
    markPidBusy,
    toasts,
    pushToast,
    dismissToast,
  };
}
