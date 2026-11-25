import { useLayoutEffect, useMemo, useState, type RefObject } from "react";
import {
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  useReactTable,
  type SortingState,
} from "@tanstack/react-table";
import { ArrowDown, ArrowUp, ArrowUpDown, Radio } from "lucide-react";
import type { FilterChip, PortEntry } from "../../types/port";
import { buildPortColumns, type TableActionCallbacks } from "./columns";

interface PortTableProps extends TableActionCallbacks {
  entries: PortEntry[];
  orderedEntriesRef?: RefObject<PortEntry[]>;
  isLoading: boolean;
  selectedId: string | null;
  activeChip: FilterChip;
  searchQuery: string;
  onSelect: (id: string) => void;
  onClearFilters: () => void;
}

export function PortTable({
  entries,
  orderedEntriesRef,
  isLoading,
  selectedId,
  activeChip,
  searchQuery,
  onSelect,
  onClearFilters,
  pendingKillId,
  busyPids,
  onOpenBrowser,
  onCopyUrl,
  onOpenIde,
  onStopProcess,
  onForceKillClick,
}: PortTableProps) {
  const [sorting, setSorting] = useState<SortingState>([
    { id: "port", desc: false },
  ]);

  const columns = useMemo(
    () =>
      buildPortColumns({
        pendingKillId,
        busyPids,
        onOpenBrowser,
        onCopyUrl,
        onOpenIde,
        onStopProcess,
        onForceKillClick,
      }),
    [
      pendingKillId,
      busyPids,
      onOpenBrowser,
      onCopyUrl,
      onOpenIde,
      onStopProcess,
      onForceKillClick,
    ],
  );

  const table = useReactTable({
    data: entries,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getRowId: (row) => row.id,
  });

  const rows = table.getRowModel().rows;

  useLayoutEffect(() => {
    if (orderedEntriesRef) {
      orderedEntriesRef.current = rows.map((row) => row.original);
    }
  }, [orderedEntriesRef, rows]);

  if (isLoading && entries.length === 0) {
    return (
      <div className="flex-1 bg-[var(--view-bg-color)] flex items-center justify-center text-[var(--muted-fg-color)] text-[13px]">
        Scanning local listening ports…
      </div>
    );
  }

  if (entries.length === 0) {
    const hasFilters = activeChip !== "all" || searchQuery.trim().length > 0;
    return (
      <div className="flex-1 bg-[var(--view-bg-color)] flex flex-col items-center justify-center gap-2.5 px-6 text-center">
        <Radio className="w-8 h-8 text-[var(--subtle-fg-color)]" />
        <div className="text-[14px] font-medium text-[var(--window-fg-color)]">
          {hasFilters
            ? "No listening ports match your filters"
            : "No active non-OS ports detected"}
        </div>
        <p className="text-[12.5px] text-[var(--muted-fg-color)] max-w-md">
          {hasFilters
            ? "Clear the search query or switch back to All to inspect every active listener."
            : "Start a local dev server, Docker container, or tunnel—iPort polls every 2 seconds."}
        </p>
        {hasFilters && (
          <button
            type="button"
            onClick={onClearFilters}
            className="mt-1 px-3 py-1.5 rounded-[6px] bg-[var(--entry-bg-color)] hover:bg-[var(--hover-bg-color)] border border-[var(--border-color)] text-[12px] font-medium text-[var(--window-fg-color)] cursor-pointer"
          >
            Reset Filters
          </button>
        )}
      </div>
    );
  }

  return (
    <div className="flex-1 min-h-0 overflow-auto bg-[var(--view-bg-color)]">
      <table className="w-full border-collapse text-left">
        <thead className="sticky top-0 z-10 bg-[var(--window-bg-color)] border-b border-[var(--border-color)]">
          {table.getHeaderGroups().map((headerGroup) => (
            <tr key={headerGroup.id}>
              {headerGroup.headers.map((header) => {
                const canSort = header.column.getCanSort();
                const sorted = header.column.getIsSorted();
                const isNumeric =
                  header.id === "pid" || header.id === "uptime";
                const isActions = header.id === "actions";

                return (
                  <th
                    key={header.id}
                    scope="col"
                    style={{ width: header.getSize() !== 150 ? header.getSize() : undefined }}
                    className={`h-[32px] px-3 text-[11.5px] font-medium text-[var(--muted-fg-color)] whitespace-nowrap ${
                      isNumeric || isActions ? "text-right" : "text-left"
                    }`}
                  >
                    {canSort ? (
                      <button
                        type="button"
                        onClick={header.column.getToggleSortingHandler()}
                        className={`inline-flex items-center gap-1 hover:text-[var(--window-fg-color)] transition-colors cursor-pointer ${
                          isNumeric ? "flex-row-reverse" : ""
                        }`}
                      >
                        <span>
                          {flexRender(
                            header.column.columnDef.header,
                            header.getContext(),
                          )}
                        </span>
                        {sorted === "asc" ? (
                          <ArrowUp className="w-3 h-3 text-[var(--accent-bg-color)]" />
                        ) : sorted === "desc" ? (
                          <ArrowDown className="w-3 h-3 text-[var(--accent-bg-color)]" />
                        ) : (
                          <ArrowUpDown className="w-3 h-3 opacity-35" />
                        )}
                      </button>
                    ) : (
                      flexRender(
                        header.column.columnDef.header,
                        header.getContext(),
                      )
                    )}
                  </th>
                );
              })}
            </tr>
          ))}
        </thead>
        <tbody>
          {table.getRowModel().rows.map((row) => {
            const entry = row.original;
            const selected = entry.id === selectedId;
            return (
              <tr
                key={row.id}
                onClick={() => onSelect(entry.id)}
                aria-selected={selected}
                className={`h-[44px] border-b border-[var(--hairline-color)] cursor-pointer transition-colors ${
                  selected
                    ? "bg-[var(--accent-soft-bg)]"
                    : "hover:bg-[var(--hover-bg-color)]"
                }`}
              >
                {row.getVisibleCells().map((cell, idx) => {
                  const isNumeric =
                    cell.column.id === "pid" || cell.column.id === "uptime";
                  const isActions = cell.column.id === "actions";
                  return (
                    <td
                      key={cell.id}
                      className={`px-3 py-1 align-middle ${
                        idx === 0 && selected
                          ? "shadow-[inset_3px_0_0_var(--accent-bg-color)]"
                          : ""
                      } ${isNumeric || isActions ? "text-right" : "text-left"}`}
                    >
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext(),
                      )}
                    </td>
                  );
                })}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
