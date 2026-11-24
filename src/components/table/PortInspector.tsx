import {
  Code2,
  Copy,
  ExternalLink,
  FolderOpen,
  Square,
  X,
  Zap,
} from "lucide-react";
import type { PortEntry } from "../../types/port";
import { categoryLabel } from "../../utils/filter";
import { buildPortUrl } from "../../utils/format";

interface PortInspectorProps {
  entry: PortEntry;
  pendingKillId: string | null;
  busyPids: Set<number>;
  onClose: () => void;
  onCopyText: (text: string, label: string) => void;
  onOpenBrowser: (entry: PortEntry) => void;
  onRevealFolder: (path: string) => void;
  onOpenIde: (entry: PortEntry) => void;
  onStopProcess: (entry: PortEntry) => void;
  onForceKillClick: (entry: PortEntry) => void;
}

export function PortInspector({
  entry,
  pendingKillId,
  busyPids,
  onClose,
  onCopyText,
  onOpenBrowser,
  onRevealFolder,
  onOpenIde,
  onStopProcess,
  onForceKillClick,
}: PortInspectorProps) {
  const pid = entry.pid;
  const busy = pid !== null && busyPids.has(pid);
  const killArmed = pendingKillId === entry.id;
  const url = buildPortUrl(entry.port, entry.bindAddress);

  return (
    <section
      aria-label="Port inspector"
      className="shrink-0 bg-[var(--window-bg-color)] border-t border-[var(--border-color)] px-3.5 py-2.5"
    >
      <div className="adw-boxed-list overflow-hidden">
        {/* Card Header */}
        <div className="px-3.5 py-2 border-b border-[var(--hairline-color)] flex items-center justify-between gap-3 flex-wrap">
          <div className="flex items-center gap-2 min-w-0">
            <span className="font-mono-tabular font-semibold text-[13px] text-[var(--window-fg-color)]">
              :{entry.port}
            </span>
            <span className="font-mono-tabular text-[11px] uppercase px-1.5 py-0.5 rounded-[4px] bg-[var(--entry-bg-color)] text-[var(--muted-fg-color)]">
              {entry.protocol}
            </span>
            <span className="text-[13px] font-medium text-[var(--window-fg-color)] truncate">
              {entry.docker?.containerName || entry.framework || entry.processName}
            </span>
            {entry.projectName && (
              <span className="text-[12px] text-[var(--muted-fg-color)] truncate">
                · {entry.projectName}
              </span>
            )}
          </div>

          <div className="flex items-center gap-1.5 shrink-0">
            {entry.protocol === "tcp" && (
              <button
                type="button"
                onClick={() => onOpenBrowser(entry)}
                className="adw-flat-btn h-7 px-2.5 text-[12px] flex items-center gap-1.5 border border-[var(--border-color)] text-[var(--window-fg-color)] cursor-pointer"
              >
                <ExternalLink className="w-3.5 h-3.5" />
                <span>Open Browser</span>
              </button>
            )}

            {entry.cwd && (
              <>
                <button
                  type="button"
                  onClick={() => onRevealFolder(entry.cwd!)}
                  className="adw-flat-btn h-7 px-2.5 text-[12px] flex items-center gap-1.5 border border-[var(--border-color)] text-[var(--window-fg-color)] cursor-pointer"
                >
                  <FolderOpen className="w-3.5 h-3.5" />
                  <span>Files</span>
                </button>
                <button
                  type="button"
                  onClick={() => onOpenIde(entry)}
                  className="adw-flat-btn h-7 px-2.5 text-[12px] flex items-center gap-1.5 border border-[var(--border-color)] text-[var(--window-fg-color)] cursor-pointer"
                >
                  <Code2 className="w-3.5 h-3.5" />
                  <span>IDE</span>
                </button>
              </>
            )}

            {pid !== null && (
              <>
                <button
                  type="button"
                  disabled={busy}
                  onClick={() => onStopProcess(entry)}
                  className="adw-flat-btn h-7 px-2.5 text-[12px] flex items-center gap-1.5 border border-[var(--border-color)] text-[var(--window-fg-color)] disabled:opacity-40 cursor-pointer"
                >
                  <Square className="w-3 h-3" />
                  <span>Stop</span>
                </button>
                <button
                  type="button"
                  disabled={busy}
                  onClick={() => onForceKillClick(entry)}
                  className={`h-7 px-2.5 text-[12px] flex items-center gap-1.5 font-medium cursor-pointer disabled:opacity-40 ${
                    killArmed
                      ? "adw-destructive-btn"
                      : "adw-flat-btn border border-[var(--border-color)] text-[var(--destructive-bg-color)]"
                  }`}
                >
                  <Zap className="w-3.5 h-3.5" />
                  <span>{killArmed ? "Confirm Force Kill" : "Force Kill"}</span>
                </button>
              </>
            )}

            <button
              type="button"
              onClick={onClose}
              title="Close inspector (Esc)"
              aria-label="Close inspector"
              className="adw-flat-btn w-7 h-7 flex items-center justify-center text-[var(--muted-fg-color)] cursor-pointer"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        {/* Boxed List Rows */}
        <div className="divide-y divide-[var(--hairline-color)] text-[12px]">
          {entry.docker && (
            <div className="px-3.5 py-1.5 flex items-center justify-between gap-3 flex-wrap">
              <div className="flex items-center gap-3 min-w-0">
                <span className="w-24 shrink-0 text-[var(--muted-fg-color)]">
                  Container
                </span>
                <code className="font-mono-tabular selectable-text text-[var(--window-fg-color)] truncate">
                  {entry.docker.image}
                </code>
              </div>
              <div className="font-mono-tabular text-[11.5px] text-[var(--muted-fg-color)] flex items-center gap-3 shrink-0">
                <span>
                  :{entry.port} → :{entry.docker.containerPort}/{entry.protocol}
                </span>
                {entry.docker.composeProject && (
                  <span>
                    {entry.docker.composeProject}
                    {entry.docker.composeService
                      ? `/${entry.docker.composeService}`
                      : ""}
                  </span>
                )}
                {entry.docker.status && <span>{entry.docker.status}</span>}
              </div>
            </div>
          )}
          <div className="px-3.5 py-1.5 flex items-center justify-between gap-3">
            <div className="flex items-center gap-3 min-w-0">
              <span className="w-24 shrink-0 text-[var(--muted-fg-color)]">
                Command
              </span>
              <code className="font-mono-tabular selectable-text text-[var(--window-fg-color)] truncate">
                {entry.commandLine || entry.processName}
              </code>
            </div>
            <button
              type="button"
              onClick={() =>
                onCopyText(entry.commandLine || entry.processName, "command")
              }
              className="adw-flat-btn h-6 px-2 text-[11px] text-[var(--muted-fg-color)] flex items-center gap-1 shrink-0 cursor-pointer"
            >
              <Copy className="w-3 h-3" />
              <span>Copy</span>
            </button>
          </div>

          <div className="px-3.5 py-1.5 flex items-center justify-between gap-3">
            <div className="flex items-center gap-3 min-w-0">
              <span className="w-24 shrink-0 text-[var(--muted-fg-color)]">
                Working Dir
              </span>
              <code className="font-mono-tabular selectable-text text-[var(--window-fg-color)] truncate">
                {entry.cwd ?? "—"}
              </code>
            </div>
            <button
              type="button"
              onClick={() => onCopyText(url, "URL")}
              className="adw-flat-btn h-6 px-2 text-[11px] text-[var(--muted-fg-color)] flex items-center gap-1 shrink-0 cursor-pointer"
            >
              <Copy className="w-3 h-3" />
              <span>{url}</span>
            </button>
          </div>

          <div className="px-3.5 py-1.5 flex items-center justify-between gap-3 flex-wrap">
            <div className="flex items-center gap-3 min-w-0">
              <span className="w-24 shrink-0 text-[var(--muted-fg-color)]">
                Executable
              </span>
              <code className="font-mono-tabular selectable-text text-[var(--muted-fg-color)] truncate">
                {entry.exePath ?? "—"}
              </code>
            </div>
            <div className="font-mono-tabular text-[11.5px] text-[var(--muted-fg-color)] flex items-center gap-3 shrink-0">
              <span>PID {entry.pid ?? "—"}</span>
              <span>PPID {entry.ppid ?? "—"}</span>
              <span>{entry.bindAddress}</span>
              <span>{entry.state}</span>
              <span>{categoryLabel(entry.category)}</span>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
