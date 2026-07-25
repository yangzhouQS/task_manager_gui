import { create } from "zustand";
import type {
  AppHistoryEntry,
  PerformanceSnapshot,
  ProcessInfo,
  ServiceInfo,
  UserSession,
  TabId,
} from "@/types";

export type ThemeMode = "light" | "dark";

interface AppState {
  tab: TabId;
  theme: ThemeMode;
  alwaysOnTop: boolean;
  search: string;
  elevated: boolean;
  cores: number;

  processes: ProcessInfo[];
  performance: PerformanceSnapshot | null;
  services: ServiceInfo[];
  users: UserSession[];
  appHistory: AppHistoryEntry[];
  iconMap: Record<string, string>;

  setTab: (t: TabId) => void;
  setTheme: (t: ThemeMode) => void;
  toggleAlwaysOnTop: () => void;
  setSearch: (s: string) => void;
  setMeta: (elevated: boolean, cores: number) => void;
  setProcesses: (p: ProcessInfo[]) => void;
  setPerformance: (p: PerformanceSnapshot) => void;
  setServices: (s: ServiceInfo[]) => void;
  setUsers: (u: UserSession[]) => void;
  setAppHistory: (h: AppHistoryEntry[]) => void;
  setIcon: (path: string, icon: string) => void;
}

export const useStore = create<AppState>((set) => ({
  tab: "processes",
  theme:
    typeof window !== "undefined" &&
    window.matchMedia &&
    window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light",
  alwaysOnTop: false,
  search: "",
  elevated: false,
  cores: 1,

  processes: [],
  performance: null,
  services: [],
  users: [],
  appHistory: [],
  iconMap: {},

  setTab: (t) => set({ tab: t }),
  setTheme: (t) => set({ theme: t }),
  toggleAlwaysOnTop: () => set((s) => ({ alwaysOnTop: !s.alwaysOnTop })),
  setSearch: (s) => set({ search: s }),
  setMeta: (elevated, cores) => set({ elevated, cores }),
  setProcesses: (p) => set({ processes: p }),
  setPerformance: (p) => set({ performance: p }),
  setServices: (s) => set({ services: s }),
  setUsers: (u) => set({ users: u }),
  setAppHistory: (h) => set({ appHistory: h }),
  setIcon: (path, icon) => set((s) => ({ iconMap: { ...s.iconMap, [path]: icon } })),
}));
