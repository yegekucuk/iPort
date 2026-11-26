export type Protocol = "tcp" | "udp";

export type SourceType =
  | "local"
  | "docker"
  | "service"
  | "tunnel"
  | "third_party"
  | "system";

export type AppCategory =
  | "dev"
  | "docker"
  | "service"
  | "tunnel"
  | "third_party"
  | "system";

export type ServiceManager =
  | "none"
  | "pm2"
  | "brew"
  | "systemd"
  | "launchd"
  | "windows_service"
  | "docker";

export type ManagedAction =
  | "stop"
  | "force_kill"
  | "restart"
  | "open_browser"
  | "copy_url"
  | "copy_port"
  | "reveal_folder";

export interface DockerMeta {
  containerId: string;
  containerName: string;
  image: string;
  status: string | null;
  containerPort: number;
  composeProject: string | null;
  composeService: string | null;
}

export interface PortEntry {
  id: string;
  port: number;
  protocol: Protocol;
  bindAddress: string;
  state: string;
  pid: number | null;
  ppid: number | null;
  processName: string;
  exePath: string | null;
  commandLine: string;
  cwd: string | null;
  projectName: string | null;
  cpuPercent: number;
  memoryBytes: number;
  uptimeSecs: number | null;
  sourceType: SourceType;
  category: AppCategory;
  isCoreOs: boolean;
  framework: string | null;
  runtime: string | null;
  serviceManager: ServiceManager;
  docker: DockerMeta | null;
  availableActions: ManagedAction[];
}

export interface SnapshotCounts {
  totalVisible: number;
  dev: number;
  docker: number;
  services: number;
  tunnels: number;
  thirdParty: number;
  system: number;
}

export interface PortSnapshot {
  timestampMs: number;
  entries: PortEntry[];
  counts: SnapshotCounts;
}

export interface ActionResult {
  ok: boolean;
  message: string;
  requiresElevation: boolean;
}

export interface SystemThemeInfo {
  colorScheme: string;
  gtkTheme: string;
  accentHex: string;
  accentDarkHex: string;
}

export type FilterChip = "all" | AppCategory;
export type ThemeMode = "system" | "dark" | "light";
