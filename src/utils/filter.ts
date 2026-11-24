import type { FilterChip, PortEntry } from "../types/port";

export function filterEntries(
  entries: PortEntry[],
  activeChip: FilterChip,
  query: string,
): PortEntry[] {
  const normalized = query.trim().toLowerCase();
  const portQuery = normalized.startsWith(":") ? normalized.slice(1) : normalized;

  return entries.filter((entry) => {
    if (activeChip === "all") {
      if (entry.isCoreOs) {
        return false;
      }
    } else if (activeChip === "system") {
      if (!entry.isCoreOs && entry.category !== "system") {
        return false;
      }
    } else {
      if (entry.isCoreOs || entry.category !== activeChip) {
        return false;
      }
    }

    if (!normalized) {
      return true;
    }

    const haystack = [
      String(entry.port),
      `:${entry.port}`,
      entry.protocol,
      entry.processName,
      entry.framework ?? "",
      entry.runtime ?? "",
      entry.projectName ?? "",
      entry.bindAddress,
      entry.pid !== null ? String(entry.pid) : "",
      entry.commandLine,
      entry.cwd ?? "",
      entry.docker?.containerName ?? "",
      entry.docker?.image ?? "",
      entry.docker?.composeProject ?? "",
      entry.docker?.composeService ?? "",
    ]
      .join(" ")
      .toLowerCase();

    return haystack.includes(normalized) || (portQuery.length > 0 && String(entry.port).includes(portQuery));
  });
}

export function categoryLabel(category: PortEntry["category"]): string {
  switch (category) {
    case "dev":
      return "Dev";
    case "docker":
      return "Docker";
    case "service":
      return "Service";
    case "tunnel":
      return "Tunnel";
    case "third_party":
      return "3rd-Party";
    case "system":
      return "System";
  }
}
