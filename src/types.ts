export type ProcessKind = "app" | "background" | "windows";

export interface ProcessInfo {
  pid: number;
  ppid: number;
  name: string;
  icon: string | null;
  description: string | null;
  exePath: string | null;
  commandLine: string | null;
  cpuPercent: number;
  memoryKb: number;
  workingSetKb: number;
  privateKb: number;
  diskMbps: number;
  netMbps: number;
  gpuPercent: number;
  powerUsage: string;
  threadCount: number;
  handleCount: number;
  status: string;
  kind: ProcessKind;
  groupCount: number;
  user: string | null;
}

export interface DiskUsage {
  name: string;
  percent: number;
  readMbps: number;
  writeMbps: number;
  avgResponseMs: number;
}

export interface NetUsage {
  name: string;
  sendMbps: number;
  recvMbps: number;
}

export interface GpuUsage {
  name: string;
  percent: number;
  dedicatedKb: number;
  sharedKb: number;
}

export interface PerformanceSnapshot {
  cpuPercent: number;
  cpuPerCore: number[];
  cpuCoreCount: number;
  cpuSpeedGhz: number;
  cpuUpTimeSec: number;
  memTotalKb: number;
  memInUseKb: number;
  memAvailableKb: number;
  memCommittedKb: number;
  memCommittedLimitKb: number;
  memCachedKb: number;
  memPagedPoolKb: number;
  memNonPagedPoolKb: number;
  diskUsage: DiskUsage[];
  netUsage: NetUsage[];
  gpuUsage: GpuUsage[];
}

export interface ServiceInfo {
  name: string;
  displayName: string;
  pid: number;
  state: string;
  startType: string;
  description: string | null;
  exePath: string | null;
  kind: string;
}

export interface StartupEntry {
  name: string;
  command: string;
  source: string;
  enabled: boolean;
  impact: string;
  publisher: string | null;
  icon: string | null;
}

export interface UserSession {
  user: string;
  sessionId: number;
  isCurrent: boolean;
  processes: ProcessInfo[];
}

export interface AppHistoryEntry {
  name: string;
  icon: string | null;
  totalCpuSec: number;
  networkMb: number;
  meteredMb: number;
  tileUpdates: number;
}

export interface AppInfo {
  elevated: boolean;
  cores: number;
}

export type TabId =
  | "processes"
  | "performance"
  | "appHistory"
  | "startup"
  | "users"
  | "details"
  | "services";
