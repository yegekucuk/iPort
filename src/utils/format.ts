export function formatMemory(bytes: number): string {
  if (!bytes || bytes <= 0) {
    return "—";
  }
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let idx = 0;
  while (value >= 1024 && idx < units.length - 1) {
    value /= 1024;
    idx += 1;
  }
  const decimals = idx >= 2 && value < 10 ? 1 : 0;
  return `${value.toFixed(decimals)} ${units[idx]}`;
}

export function formatCpu(percent: number): string {
  if (!Number.isFinite(percent) || percent < 0.05) {
    return "0.0%";
  }
  return `${percent.toFixed(1)}%`;
}

export function formatUptime(secs: number | null): string {
  if (secs === null || secs < 0) {
    return "—";
  }
  if (secs < 60) {
    return `${secs}s`;
  }
  const mins = Math.floor(secs / 60);
  if (mins < 60) {
    return `${mins}m`;
  }
  const hours = Math.floor(mins / 60);
  const remMins = mins % 60;
  if (hours < 24) {
    return remMins > 0 ? `${hours}h ${remMins}m` : `${hours}h`;
  }
  const days = Math.floor(hours / 24);
  const remHours = hours % 24;
  return remHours > 0 ? `${days}d ${remHours}h` : `${days}d`;
}

export function buildPortUrl(
  port: number,
  bindAddress: string,
  useHttps?: boolean,
): string {
  const isHttps = useHttps ?? (port === 443 || port === 8443);
  const scheme = isHttps ? "https" : "http";
  const trimmed = bindAddress.trim();

  let host = trimmed;
  if (!trimmed || trimmed === "0.0.0.0" || trimmed === "::" || trimmed === "*" || trimmed === "[::]") {
    host = "localhost";
  } else if (trimmed.includes(":") && !trimmed.startsWith("[")) {
    host = `[${trimmed}]`;
  }

  return `${scheme}://${host}:${port}`;
}

export function shortenPath(fullPath: string | null, maxLen = 46): string {
  if (!fullPath) {
    return "—";
  }
  const homeReplaced = fullPath.replace(/^\/(home|Users)\/[^/]+/, "~");
  if (homeReplaced.length <= maxLen) {
    return homeReplaced;
  }
  const keepTail = Math.max(18, maxLen - 14);
  return `${homeReplaced.slice(0, 11)}…${homeReplaced.slice(-keepTail)}`;
}

export function formatRelativeAge(timestampMs: number, nowMs = Date.now()): string {
  if (!timestampMs) {
    return "Pending…";
  }
  const diffSecs = Math.max(0, Math.floor((nowMs - timestampMs) / 1000));
  if (diffSecs <= 2) {
    return "Just now";
  }
  if (diffSecs < 60) {
    return `${diffSecs}s ago`;
  }
  const mins = Math.floor(diffSecs / 60);
  return `${mins}m ago`;
}
