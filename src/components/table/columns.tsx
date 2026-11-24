import { createColumnHelper, type ColumnDef } from "@tanstack/react-table";
import { Code2, Copy, ExternalLink, Square, Zap } from "lucide-react";
import type { PortEntry } from "../../types/port";
import { categoryLabel } from "../../utils/filter";
import {
  formatCpu,
  formatMemory,
  formatUptime,
  shortenPath,
} from "../../utils/format";

export interface TableActionCallbacks {
  pendingKillId: string | null;
  busyPids: Set<number>;
  onOpenBrowser: (entry: PortEntry) => void;
  onCopyUrl: (entry: PortEntry) => void;
  onOpenIde: (entry: PortEntry) => void;
  onStopProcess: (entry: PortEntry) => void;
  onForceKillClick: (entry: PortEntry) => void;
}

const columnHelper = createColumnHelper<PortEntry>();

function categoryBadgeStyle(category: PortEntry["category"]): string {
  switch (category) {
    case "dev":
      return "bg-[var(--accent-soft-bg)] text-[var(--accent-bg-color)]";
    case "docker":
      return "bg-sky-500/12 text-sky-600 dark:text-sky-400";
    case "service":
      return "bg-amber-500/12 text-amber-700 dark:text-amber-300";
    case "tunnel":
      return "bg-purple-500/12 text-purple-700 dark:text-purple-300";
    case "third_party":
      return "bg-emerald-500/12 text-emerald-700 dark:text-emerald-400";
    case "system":
      return "bg-[var(--entry-bg-color)] text-[var(--muted-fg-color)]";
  }
}

export function buildPortColumns(
  callbacks: TableActionCallbacks,
): ColumnDef<PortEntry, unknown>[] {
  return [
    columnHelper.accessor("port", {
      header: "Port",
      size: 132,
      cell: (info) => {
        const entry = info.row.original;
        return (
          <div className="flex items-center gap-2">
            <span
              className="w-2 h-2 rounded-full shrink-0"
              style={{
                backgroundColor: entry.isCoreOs
                  ? "var(--subtle-fg-color)"
                  : "var(--success-color)",
              }}
              aria-hidden="true"
            />
            <span className="font-mono-tabular font-semibold text-[13px] text-[var(--window-fg-color)]">
              :{entry.port}
            </span>
            <span className="font-mono-tabular text-[10.5px] uppercase px-1.5 py-0.5 rounded-[4px] bg-[var(--entry-bg-color)] text-[var(--muted-fg-color)]">
              {entry.protocol}
            </span>
          </div>
        );
      },
    }) as ColumnDef<PortEntry, unknown>,

    columnHelper.accessor(
      (row) => row.docker?.containerName || row.framework || row.processName,
      {
        id: "process",
        header: "Process / Stack",
        cell: (info) => {
          const entry = info.row.original;
          const docker = entry.docker;
          const title = docker
            ? docker.containerName
            : entry.framework
              ? entry.framework
              : entry.processName;
          const subtitle = docker
            ? docker.image
            : entry.projectName ||
              (entry.cwd ? shortenPath(entry.cwd, 42) : entry.processName);
          const showProcessTag =
            !docker &&
            Boolean(entry.framework) &&
            entry.processName.toLowerCase() !== entry.framework?.toLowerCase();

          return (
            <div className="min-w-0 py-0.5">
              <div className="flex items-center gap-1.5 min-w-0">
                <span className="font-medium text-[13px] text-[var(--window-fg-color)] truncate">
                  {title}
                </span>
                {docker && docker.containerPort !== entry.port && (
                  <span className="font-mono-tabular text-[11px] px-1.5 py-0.25 rounded-[4px] bg-[var(--entry-bg-color)] text-[var(--muted-fg-color)] shrink-0">
                    → :{docker.containerPort}
                  </span>
                )}
                {docker?.composeProject && (
                  <span className="text-[11px] px-1.5 py-0.25 rounded-[4px] bg-[var(--entry-bg-color)] text-[var(--muted-fg-color)] shrink-0">
                    {docker.composeProject}
                  </span>
                )}
                {entry.runtime && (
                  <span className="text-[11px] px-1.5 py-0.25 rounded-[4px] bg-[var(--entry-bg-color)] text-[var(--muted-fg-color)] shrink-0">
                    {entry.runtime}
                  </span>
                )}
                {showProcessTag && (
                  <span className="font-mono-tabular text-[11px] text-[var(--subtle-fg-color)] truncate">
                    {entry.processName}
                  </span>
                )}
                {entry.serviceManager !== "none" && (
                  <span className="text-[10.5px] uppercase px-1.5 py-0.25 rounded-[4px] bg-[var(--entry-bg-color)] text-[var(--muted-fg-color)] shrink-0">
                    {entry.serviceManager}
                  </span>
                )}
              </div>
              <div className="text-[11.5px] text-[var(--muted-fg-color)] truncate">
                {subtitle}
              </div>
            </div>
          );
        },
      },
    ) as ColumnDef<PortEntry, unknown>,

    columnHelper.accessor("category", {
      header: "Category",
      size: 100,
      cell: (info) => {
        const cat = info.getValue();
        return (
          <span
            className={`inline-flex items-center px-2 py-0.5 rounded-[5px] text-[11px] font-medium ${categoryBadgeStyle(cat)}`}
          >
            {categoryLabel(cat)}
          </span>
        );
      },
    }) as ColumnDef<PortEntry, unknown>,

    columnHelper.accessor("bindAddress", {
      header: "Bind",
      size: 120,
      cell: (info) => (
        <span className="font-mono-tabular text-[12px] text-[var(--muted-fg-color)]">
          {info.getValue()}
        </span>
      ),
    }) as ColumnDef<PortEntry, unknown>,

    columnHelper.accessor((row) => row.pid ?? -1, {
      id: "pid",
      header: "PID",
      size: 76,
      cell: (info) => {
        const pid = info.row.original.pid;
        return (
          <span className="font-mono-tabular text-[12px] text-[var(--muted-fg-color)]">
            {pid ?? "—"}
          </span>
        );
      },
    }) as ColumnDef<PortEntry, unknown>,

    columnHelper.accessor("memoryBytes", {
      header: "CPU / RAM",
      size: 126,
      cell: (info) => {
        const entry = info.row.original;
        return (
          <span className="font-mono-tabular text-[12px] text-[var(--muted-fg-color)] whitespace-nowrap">
            {formatCpu(entry.cpuPercent)} · {formatMemory(entry.memoryBytes)}
          </span>
        );
      },
    }) as ColumnDef<PortEntry, unknown>,

    columnHelper.accessor((row) => row.uptimeSecs ?? -1, {
      id: "uptime",
      header: "Uptime",
      size: 78,
      cell: (info) => (
        <span className="font-mono-tabular text-[12px] text-[var(--muted-fg-color)]">
          {formatUptime(info.row.original.uptimeSecs)}
        </span>
      ),
    }) as ColumnDef<PortEntry, unknown>,

    columnHelper.display({
      id: "actions",
      header: () => <span className="sr-only">Actions</span>,
      size: 174,
      cell: (info) => {
        const entry = info.row.original;
        const pid = entry.pid;
        const busy = pid !== null && callbacks.busyPids.has(pid);
        const killArmed = callbacks.pendingKillId === entry.id;

        return (
          <div
            className="flex items-center justify-end gap-0.5"
            onClick={(e) => e.stopPropagation()}
          >
            {entry.protocol === "tcp" && (
              <button
                type="button"
                title="Open in Browser (o)"
                aria-label="Open in Browser"
                onClick={() => callbacks.onOpenBrowser(entry)}
                className="adw-flat-btn w-7 h-7 flex items-center justify-center text-[var(--muted-fg-color)] hover:text-[var(--window-fg-color)] cursor-pointer"
              >
                <ExternalLink className="w-3.5 h-3.5" />
              </button>
            )}

            <button
              type="button"
              title="Copy URL (c)"
              aria-label="Copy URL"
              onClick={() => callbacks.onCopyUrl(entry)}
              className="adw-flat-btn w-7 h-7 flex items-center justify-center text-[var(--muted-fg-color)] hover:text-[var(--window-fg-color)] cursor-pointer"
            >
              <Copy className="w-3.5 h-3.5" />
            </button>

            {entry.cwd && (
              <button
                type="button"
                title="Open Project in IDE"
                aria-label="Open Project in IDE"
                onClick={() => callbacks.onOpenIde(entry)}
                className="adw-flat-btn w-7 h-7 flex items-center justify-center text-[var(--muted-fg-color)] hover:text-[var(--window-fg-color)] cursor-pointer"
              >
                <Code2 className="w-3.5 h-3.5" />
              </button>
            )}

            {pid !== null && (
              <>
                <button
                  type="button"
                  disabled={busy}
                  title="Graceful Stop (SIGTERM) (x)"
                  aria-label="Graceful Stop"
                  onClick={() => callbacks.onStopProcess(entry)}
                  className="adw-flat-btn w-7 h-7 flex items-center justify-center text-[var(--muted-fg-color)] hover:text-[var(--warning-color)] disabled:opacity-40 cursor-pointer"
                >
                  <Square className="w-3.5 h-3.5" />
                </button>

                {killArmed ? (
                  <button
                    type="button"
                    disabled={busy}
                    title="Click again to send SIGKILL"
                    onClick={() => callbacks.onForceKillClick(entry)}
                    className="adw-destructive-btn h-6 px-2 text-[11px] font-medium cursor-pointer"
                  >
                    Kill?
                  </button>
                ) : (
                  <button
                    type="button"
                    disabled={busy}
                    title="Force Kill (SIGKILL)"
                    aria-label="Force Kill"
                    onClick={() => callbacks.onForceKillClick(entry)}
                    className="adw-flat-btn w-7 h-7 flex items-center justify-center text-[var(--muted-fg-color)] hover:text-[var(--destructive-bg-color)] disabled:opacity-40 cursor-pointer"
                  >
                    <Zap className="w-3.5 h-3.5" />
                  </button>
                )}
              </>
            )}
          </div>
        );
      },
    }) as ColumnDef<PortEntry, unknown>,
  ];
}
