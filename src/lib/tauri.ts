import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ActionResult,
  IdeOption,
  PortSnapshot,
  SystemThemeInfo,
} from "../types/port";

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

const PREVIEW_SNAPSHOT: PortSnapshot = {
  timestampMs: Date.now(),
  counts: {
    totalVisible: 5,
    dev: 3,
    docker: 1,
    services: 0,
    tunnels: 1,
    thirdParty: 1,
    system: 2,
  },
  entries: [
    {
      id: "tcp:3000:127.0.0.1:4120",
      port: 3000,
      protocol: "tcp",
      bindAddress: "127.0.0.1",
      state: "LISTEN",
      pid: 4120,
      ppid: 3980,
      processName: "node",
      exePath: "/usr/bin/node",
      commandLine: "node /home/dev/projects/web-admin/node_modules/.bin/next dev -p 3000",
      cwd: "/home/dev/projects/web-admin",
      projectName: "web-admin",
      cpuPercent: 1.4,
      memoryBytes: 156 * 1024 * 1024,
      uptimeSecs: 2540,
      sourceType: "local",
      category: "dev",
      isCoreOs: false,
      framework: "Next.js",
      runtime: "Node.js",
      serviceManager: "none",
      docker: null,
      availableActions: ["open_browser", "copy_url", "copy_port", "reveal_folder", "open_ide", "stop", "force_kill"],
    },
    {
      id: "tcp:5432:127.0.0.1:2890",
      port: 5432,
      protocol: "tcp",
      bindAddress: "127.0.0.1",
      state: "LISTEN",
      pid: 2890,
      ppid: 1,
      processName: "docker-proxy",
      exePath: "/usr/bin/docker-proxy",
      commandLine: "/usr/bin/docker-proxy -proto tcp -host-ip 127.0.0.1 -host-port 5432 -container-port 5432",
      cwd: null,
      projectName: "iport-stack",
      cpuPercent: 0.2,
      memoryBytes: 22 * 1024 * 1024,
      uptimeSecs: 14800,
      sourceType: "docker",
      category: "docker",
      isCoreOs: false,
      framework: null,
      runtime: null,
      serviceManager: "docker",
      docker: {
        containerId: "e2d4f6a8c012",
        containerName: "postgres-dev",
        image: "postgres:16-alpine",
        status: "Up 4 hours",
        containerPort: 5432,
        composeProject: "iport-stack",
        composeService: "db",
      },
      availableActions: ["open_browser", "copy_url", "copy_port", "stop", "force_kill"],
    },
    {
      id: "tcp:5433:127.0.0.1:7124",
      port: 5433,
      protocol: "tcp",
      bindAddress: "127.0.0.1",
      state: "LISTEN",
      pid: 7124,
      ppid: 6900,
      processName: "ssh",
      exePath: "/usr/bin/ssh",
      commandLine: "ssh -N -L 5433:127.0.0.1:5432 staging-db",
      cwd: "/home/dev",
      projectName: null,
      cpuPercent: 0.0,
      memoryBytes: 11 * 1024 * 1024,
      uptimeSecs: 820,
      sourceType: "tunnel",
      category: "tunnel",
      isCoreOs: false,
      framework: null,
      runtime: null,
      serviceManager: "none",
      docker: null,
      availableActions: ["open_browser", "copy_url", "copy_port", "reveal_folder", "open_ide", "stop", "force_kill"],
    },
    {
      id: "tcp:8000:0.0.0.0:4892",
      port: 8000,
      protocol: "tcp",
      bindAddress: "0.0.0.0",
      state: "LISTEN",
      pid: 4892,
      ppid: 4810,
      processName: "python3",
      exePath: "/usr/bin/python3",
      commandLine: "python3 -m uvicorn app.main:app --reload --port 8000",
      cwd: "/home/dev/projects/api-core",
      projectName: "api-core",
      cpuPercent: 0.6,
      memoryBytes: 84 * 1024 * 1024,
      uptimeSecs: 3920,
      sourceType: "local",
      category: "dev",
      isCoreOs: false,
      framework: "Uvicorn",
      runtime: "Python",
      serviceManager: "none",
      docker: null,
      availableActions: ["open_browser", "copy_url", "copy_port", "reveal_folder", "open_ide", "stop", "force_kill"],
    },
    {
      id: "tcp:11434:127.0.0.1:2104",
      port: 11434,
      protocol: "tcp",
      bindAddress: "127.0.0.1",
      state: "LISTEN",
      pid: 2104,
      ppid: 1,
      processName: "ollama",
      exePath: "/usr/local/bin/ollama",
      commandLine: "/usr/local/bin/ollama serve",
      cwd: "/usr/share/ollama",
      projectName: null,
      cpuPercent: 0.3,
      memoryBytes: 642 * 1024 * 1024,
      uptimeSecs: 41200,
      sourceType: "third_party",
      category: "third_party",
      isCoreOs: false,
      framework: null,
      runtime: null,
      serviceManager: "none",
      docker: null,
      availableActions: ["open_browser", "copy_url", "copy_port", "reveal_folder", "open_ide", "stop", "force_kill"],
    },
    {
      id: "udp:53:127.0.0.53:612",
      port: 53,
      protocol: "udp",
      bindAddress: "127.0.0.53",
      state: "UNCONN",
      pid: 612,
      ppid: 1,
      processName: "systemd-resolved",
      exePath: "/usr/lib/systemd/systemd-resolved",
      commandLine: "/usr/lib/systemd/systemd-resolved",
      cwd: "/",
      projectName: null,
      cpuPercent: 0.0,
      memoryBytes: 18 * 1024 * 1024,
      uptimeSecs: 86400,
      sourceType: "system",
      category: "system",
      isCoreOs: true,
      framework: null,
      runtime: null,
      serviceManager: "systemd",
      docker: null,
      availableActions: ["copy_url", "copy_port", "stop", "force_kill"],
    },
  ],
};

export async function fetchPortSnapshot(): Promise<PortSnapshot> {
  if (!isTauriRuntime()) {
    return { ...PREVIEW_SNAPSHOT, timestampMs: Date.now() };
  }
  return invoke<PortSnapshot>("get_port_snapshot");
}

export async function refreshPortSnapshot(): Promise<PortSnapshot> {
  if (!isTauriRuntime()) {
    return { ...PREVIEW_SNAPSHOT, timestampMs: Date.now() };
  }
  return invoke<PortSnapshot>("refresh_ports");
}

export async function fetchSystemTheme(): Promise<SystemThemeInfo> {
  if (!isTauriRuntime()) {
    return {
      colorScheme: "default",
      gtkTheme: "Yaru",
      accentHex: "#e95420",
      accentDarkHex: "#f27447",
    };
  }
  return invoke<SystemThemeInfo>("get_system_theme");
}

export async function subscribePortSnapshots(
  handler: (snapshot: PortSnapshot) => void,
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return () => {};
  }
  return listen<PortSnapshot>("ports:snapshot", (event) => {
    handler(event.payload);
  });
}

export async function stopProcess(pid: number): Promise<ActionResult> {
  if (!isTauriRuntime()) {
    return { ok: false, message: "Process control requires Tauri desktop runtime.", requiresElevation: false };
  }
  return invoke<ActionResult>("stop_process", { pid });
}

export async function killProcess(pid: number): Promise<ActionResult> {
  if (!isTauriRuntime()) {
    return { ok: false, message: "Process control requires Tauri desktop runtime.", requiresElevation: false };
  }
  return invoke<ActionResult>("kill_process", { pid });
}

export async function openInBrowser(
  port: number,
  bindAddress: string,
  useHttps?: boolean,
): Promise<ActionResult> {
  if (!isTauriRuntime()) {
    return { ok: false, message: "Browser launcher requires Tauri desktop runtime.", requiresElevation: false };
  }
  return invoke<ActionResult>("open_in_browser", { port, bindAddress, useHttps });
}

export async function revealInFolder(path: string): Promise<ActionResult> {
  if (!isTauriRuntime()) {
    return { ok: false, message: "File manager action requires Tauri desktop runtime.", requiresElevation: false };
  }
  return invoke<ActionResult>("reveal_in_folder", { path });
}

export async function openInIde(path: string, ide: IdeOption): Promise<ActionResult> {
  if (!isTauriRuntime()) {
    return { ok: false, message: "IDE launcher requires Tauri desktop runtime.", requiresElevation: false };
  }
  return invoke<ActionResult>("open_in_ide", { path, ide });
}

export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    // Fallback below
  }
  try {
    const el = document.createElement("textarea");
    el.value = text;
    el.style.position = "fixed";
    el.style.opacity = "0";
    document.body.appendChild(el);
    el.select();
    const ok = document.execCommand("copy");
    document.body.removeChild(el);
    return ok;
  } catch {
    return false;
  }
}
