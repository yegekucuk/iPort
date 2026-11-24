import { describe, expect, it } from "vitest";
import type { PortEntry } from "../../types/port";
import { filterEntries } from "../filter";
import {
  buildPortUrl,
  formatCpu,
  formatMemory,
  formatUptime,
  shortenPath,
} from "../format";

function makeEntry(overrides: Partial<PortEntry>): PortEntry {
  return {
    id: "tcp:3000:127.0.0.1:100",
    port: 3000,
    protocol: "tcp",
    bindAddress: "127.0.0.1",
    state: "LISTEN",
    pid: 100,
    ppid: 1,
    processName: "node",
    exePath: "/usr/bin/node",
    commandLine: "node next dev",
    cwd: "/home/dev/projects/web-admin",
    projectName: "web-admin",
    cpuPercent: 1.2,
    memoryBytes: 128 * 1024 * 1024,
    uptimeSecs: 185,
    sourceType: "local",
    category: "dev",
    isCoreOs: false,
    framework: "Next.js",
    runtime: "Node.js",
    serviceManager: "none",
    docker: null,
    availableActions: ["open_browser", "copy_url", "stop", "force_kill"],
    ...overrides,
  };
}

describe("format utilities", () => {
  it("formats memory bytes cleanly", () => {
    expect(formatMemory(0)).toBe("—");
    expect(formatMemory(1536)).toBe("2 KB");
    expect(formatMemory(5 * 1024 * 1024)).toBe("5.0 MB");
    expect(formatMemory(2.5 * 1024 * 1024 * 1024)).toBe("2.5 GB");
  });

  it("formats cpu and uptime", () => {
    expect(formatCpu(0.01)).toBe("0.0%");
    expect(formatCpu(12.34)).toBe("12.3%");
    expect(formatUptime(null)).toBe("—");
    expect(formatUptime(42)).toBe("42s");
    expect(formatUptime(125)).toBe("2m");
    expect(formatUptime(3780)).toBe("1h 3m");
    expect(formatUptime(90000)).toBe("1d 1h");
  });

  it("builds browser URLs and shortens home paths", () => {
    expect(buildPortUrl(3000, "0.0.0.0")).toBe("http://localhost:3000");
    expect(buildPortUrl(8000, "::")).toBe("http://localhost:8000");
    expect(buildPortUrl(8443, "::1")).toBe("https://[::1]:8443");
    expect(shortenPath("/home/yegekucuk/Documents/development/tools/iport", 40)).toBe(
      "~/Documents/development/tools/iport",
    );
    expect(
      shortenPath("/home/yegekucuk/Documents/development/tools/iport/apps/web", 32),
    ).toBe("~/Documents…ols/iport/apps/web");
  });
});

describe("filterEntries", () => {
  const devEntry = makeEntry({ id: "dev-3000", port: 3000, category: "dev", isCoreOs: false });
  const thirdPartyEntry = makeEntry({
    id: "3p-11434",
    port: 11434,
    processName: "ollama",
    framework: null,
    runtime: null,
    projectName: null,
    sourceType: "third_party",
    category: "third_party",
    isCoreOs: false,
  });
  const osEntry = makeEntry({
    id: "os-53",
    port: 53,
    protocol: "udp",
    processName: "systemd-resolved",
    framework: null,
    runtime: null,
    projectName: null,
    sourceType: "system",
    category: "system",
    isCoreOs: true,
  });

  const dockerEntry = makeEntry({
    id: "docker-8888",
    port: 8888,
    processName: "docker-proxy",
    framework: null,
    runtime: null,
    projectName: "hermes",
    sourceType: "docker",
    category: "docker",
    serviceManager: "docker",
    docker: {
      containerId: "abcdef123456",
      containerName: "hermes-searxng",
      image: "searxng/searxng",
      status: "Up 3 hours",
      containerPort: 8080,
      composeProject: "hermes",
      composeService: "searxng",
    },
  });

  const all = [devEntry, dockerEntry, thirdPartyEntry, osEntry];

  it("hides core OS ports on 'all' while preserving 3rd-party and docker apps", () => {
    const visible = filterEntries(all, "all", "");
    expect(visible.map((e) => e.id)).toEqual(["dev-3000", "docker-8888", "3p-11434"]);
  });

  it("reveals core OS ports only on 'system' chip", () => {
    const systemOnly = filterEntries(all, "system", "");
    expect(systemOnly.map((e) => e.id)).toEqual(["os-53"]);
  });

  it("filters by category chip and search query including Docker metadata", () => {
    expect(filterEntries(all, "third_party", "").map((e) => e.id)).toEqual(["3p-11434"]);
    expect(filterEntries(all, "docker", "").map((e) => e.id)).toEqual(["docker-8888"]);
    expect(filterEntries(all, "all", ":3000").map((e) => e.id)).toEqual(["dev-3000"]);
    expect(filterEntries(all, "all", "ollama").map((e) => e.id)).toEqual(["3p-11434"]);
    expect(filterEntries(all, "all", "next.js").map((e) => e.id)).toEqual(["dev-3000"]);
    expect(filterEntries(all, "all", "searxng").map((e) => e.id)).toEqual(["docker-8888"]);
  });
});
