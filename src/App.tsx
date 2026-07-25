import { useEffect } from "react";
import { FluentProvider, webDarkTheme, webLightTheme } from "@fluentui/react-components";
import type { UnlistenFn } from "@tauri-apps/api/event";

import { backend, subscribe } from "@/lib/backend";
import { useStore } from "@/store/useStore";
import type {
  AppHistoryEntry,
  PerformanceSnapshot,
  ProcessInfo,
  ServiceInfo,
  UserSession,
} from "@/types";

import TitleBar from "@/components/TitleBar";
import NavRail from "@/components/NavRail";
import ProcessesView from "@/views/ProcessesView";
import PerformanceView from "@/views/PerformanceView";
import AppHistoryView from "@/views/AppHistoryView";
import StartupView from "@/views/StartupView";
import UsersView from "@/views/UsersView";
import DetailsView from "@/views/DetailsView";
import ServicesView from "@/views/ServicesView";

export default function App() {
  const theme = useStore((s) => s.theme);
  const setMeta = useStore((s) => s.setMeta);
  const setTheme = useStore((s) => s.setTheme);
  const setProcesses = useStore((s) => s.setProcesses);
  const setPerformance = useStore((s) => s.setPerformance);
  const setServices = useStore((s) => s.setServices);
  const setUsers = useStore((s) => s.setUsers);
  const setAppHistory = useStore((s) => s.setAppHistory);
  const setIcon = useStore((s) => s.setIcon);
  const alwaysOnTop = useStore((s) => s.alwaysOnTop);
  const tab = useStore((s) => s.tab);

  useEffect(() => {
    let alive = true;
    backend.getAppInfo().then((i) => alive && setMeta(i.elevated, i.cores));
    const unlistens: UnlistenFn[] = [];
    subscribe<ProcessInfo[]>("tm://processes", (p) => setProcesses(p)).then((u) => unlistens.push(u));
    subscribe<PerformanceSnapshot>("tm://performance", (p) => setPerformance(p)).then((u) =>
      unlistens.push(u)
    );
    subscribe<ServiceInfo[]>("tm://services", (s) => setServices(s)).then((u) => unlistens.push(u));
    subscribe<UserSession[]>("tm://users", (u) => setUsers(u)).then((u) => unlistens.push(u));
    subscribe<AppHistoryEntry[]>("tm://appHistory", (h) => setAppHistory(h)).then((u) =>
      unlistens.push(u)
    );
    subscribe<{ path: string; icon: string }>("tm://icon", (p) => setIcon(p.path, p.icon)).then(
      (u) => unlistens.push(u)
    );
    backend.getUsers();
    backend.getAppHistory();
    backend.getStartup();

    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => setTheme(mq.matches ? "dark" : "light");
    mq.addEventListener("change", onChange);

    return () => {
      alive = false;
      unlistens.forEach((u) => u());
      mq.removeEventListener("change", onChange);
    };
  }, [setMeta, setTheme, setProcesses, setPerformance, setServices, setUsers, setAppHistory, setIcon]);

  useEffect(() => {
    backend.toggleAlwaysOnTop(alwaysOnTop);
  }, [alwaysOnTop]);

  useEffect(() => {
    backend.setActiveTab(tab);
  }, [tab]);

  const activeTheme = theme === "dark" ? webDarkTheme : webLightTheme;
  const bg = theme === "dark" ? "rgba(32,32,32,0.92)" : "rgba(243,243,243,0.92)";

  return (
    <FluentProvider theme={activeTheme} style={{ height: "100%", background: "transparent" }}>
      <div className="win-root" style={{ background: bg }}>
        <TitleBar />
        <div className="win-body">
          <NavRail />
          <div className="win-content">
            <div className={"view-slot" + (tab === "processes" ? " active" : "")}>
              <ProcessesView />
            </div>
            <div className={"view-slot" + (tab === "performance" ? " active" : "")}>
              <PerformanceView />
            </div>
            <div className={"view-slot" + (tab === "appHistory" ? " active" : "")}>
              <AppHistoryView />
            </div>
            <div className={"view-slot" + (tab === "startup" ? " active" : "")}>
              <StartupView />
            </div>
            <div className={"view-slot" + (tab === "users" ? " active" : "")}>
              <UsersView />
            </div>
            <div className={"view-slot" + (tab === "details" ? " active" : "")}>
              <DetailsView />
            </div>
            <div className={"view-slot" + (tab === "services" ? " active" : "")}>
              <ServicesView />
            </div>
          </div>
        </div>
      </div>
    </FluentProvider>
  );
}
