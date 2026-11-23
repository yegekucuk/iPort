import { useCallback, useEffect, useState } from "react";
import {
  fetchPortSnapshot,
  isTauriRuntime,
  refreshPortSnapshot,
  subscribePortSnapshots,
} from "../lib/tauri";
import type { PortSnapshot } from "../types/port";

const EMPTY_SNAPSHOT: PortSnapshot = {
  timestampMs: 0,
  entries: [],
  counts: {
    totalVisible: 0,
    dev: 0,
    docker: 0,
    services: 0,
    tunnels: 0,
    thirdParty: 0,
    system: 0,
  },
};

export function usePortStream() {
  const [snapshot, setSnapshot] = useState<PortSnapshot>(EMPTY_SNAPSHOT);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const previewMode = !isTauriRuntime();

  useEffect(() => {
    let mounted = true;
    let unlisten: (() => void) | null = null;

    fetchPortSnapshot()
      .then((data) => {
        if (!mounted) return;
        setSnapshot(data);
        setError(null);
      })
      .catch((err) => {
        if (!mounted) return;
        setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => {
        if (mounted) {
          setIsLoading(false);
        }
      });

    subscribePortSnapshots((nextSnapshot) => {
      if (!mounted) return;
      setSnapshot(nextSnapshot);
      setIsLoading(false);
      setError(null);
    }).then((fn) => {
      if (mounted) {
        unlisten = fn;
      } else {
        fn();
      }
    });

    return () => {
      mounted = false;
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  const triggerRefresh = useCallback(async () => {
    setIsRefreshing(true);
    try {
      const next = await refreshPortSnapshot();
      setSnapshot(next);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsRefreshing(false);
    }
  }, []);

  return {
    snapshot,
    isLoading,
    isRefreshing,
    error,
    previewMode,
    triggerRefresh,
  };
}
