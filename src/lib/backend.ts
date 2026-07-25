import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppHistoryEntry,
  AppInfo,
  ProcessInfo,
  ServiceInfo,
  StartupEntry,
  UserSession,
} from "@/types";

export const backend = {
  getAppInfo: () => invoke<AppInfo>("get_app_info"),
  setActiveTab: (tab: string) => invoke("set_active_tab", { tab }),
  getProcesses: () => invoke<ProcessInfo[]>("get_processes"),
  endTask: (pid: number) => invoke("end_task", { pid }),
  runNewTask: (exe: string, args: string | null, admin: boolean | null) =>
    invoke("run_new_task", { exe, args, admin }),
  openFileLocation: (pid: number) => invoke("open_file_location", { pid }),
  openProperties: (pid: number) => invoke("open_properties", { pid }),
  searchOnline: (name: string) => invoke("search_online", { name }),
  toggleAlwaysOnTop: (on: boolean) => invoke("toggle_always_on_top", { on }),
  windowMinimize: () => invoke("window_minimize"),
  windowToggleMaximize: () => invoke("window_toggle_maximize"),
  windowClose: () => invoke("window_close"),
  setPriority: (pid: number, level: string) => invoke("set_priority", { pid, level }),
  setAffinity: (pid: number, mask: number) => invoke("set_affinity", { pid, mask }),
  efficiencyMode: (pid: number, on: boolean) => invoke("efficiency_mode", { pid, on }),
  controlService: (name: string, action: string) =>
    invoke("control_service", { name, action }),
  refreshServices: () => invoke("refresh_services"),
  openServicesMsc: () => invoke("run_new_task", { exe: "services.msc", args: null, admin: null }),
  getStartup: () => invoke<StartupEntry[]>("get_startup"),
  setStartupEnabled: (name: string, source: string, enabled: boolean) =>
    invoke("set_startup_enabled", { name, source, enabled }),
  setRefreshSpeed: (speed: string) => invoke("set_refresh_speed", { speed }),
  getUsers: () => invoke<UserSession[]>("get_users"),
  getAppHistory: () => invoke<AppHistoryEntry[]>("get_app_history"),
  resetAppHistory: () => invoke("reset_app_history"),
  getServices: () => invoke<ServiceInfo[]>("get_services"),
  restartAsAdmin: () => invoke("restart_as_admin"),
};

export function subscribe<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(event, (e) => handler(e.payload));
}

export function formatBytes(kb: number): string {
  if (kb >= 1024 * 1024) return `${(kb / 1024 / 1024).toFixed(1)} GB`;
  if (kb >= 1024) return `${(kb / 1024).toFixed(1)} MB`;
  return `${kb} KB`;
}

export function formatSpeed(mbps: number): string {
  if (mbps >= 1024) return `${(mbps / 1024).toFixed(1)} GB/s`;
  return `${mbps.toFixed(1)} MB/s`;
}

export function formatMbps(mbps: number): string {
  if (mbps >= 1000) return `${(mbps / 1000).toFixed(1)} Gbps`;
  return `${mbps.toFixed(0)} Mbps`;
}

export function formatDuration(sec: number): string {
  const d = Math.floor(sec / 86400);
  const h = Math.floor((sec % 86400) / 3600);
  const m = Math.floor((sec % 3600) / 60);
  const s = Math.floor(sec % 60);
  const pad = (n: number) => n.toString().padStart(2, "0");
  if (d > 0) return `${d}:${pad(h)}:${pad(m)}:${pad(s)}`;
  return `${pad(h)}:${pad(m)}:${pad(s)}`;
}

export function formatCpuTime(sec: number): string {
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  const s = Math.floor(sec % 60);
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${pad(h)}:${pad(m)}:${pad(s)}`;
}
