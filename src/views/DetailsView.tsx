import { useEffect, useMemo, useState } from "react";
import { backend, formatBytes } from "@/lib/backend";
import { useStore } from "@/store/useStore";
import type { ProcessInfo } from "@/types";
import ContextMenu, { type CtxState } from "@/components/ContextMenu";
import AppIcon from "@/components/AppIcon";

type Key = "pid" | "name" | "user" | "cpu" | "memory" | "status";

export default function DetailsView() {
  const processes = useStore((s) => s.processes);
  const users = useStore((s) => s.users);
  const search = useStore((s) => s.search);
  const [key, setKey] = useState<Key>("pid");
  const [asc, setAsc] = useState(true);
  const [menu, setMenu] = useState<CtxState | null>(null);

  useEffect(() => {
    backend.getUsers();
  }, []);

  const pidUser = useMemo(() => {
    const m = new Map<number, string>();
    for (const u of users) for (const p of u.processes) m.set(p.pid, u.user);
    return m;
  }, [users]);

  const rows = useMemo(() => {
    const q = search.trim().toLowerCase();
    const filtered = processes.filter(
      (p) => p.pid !== 0 && (!q || p.name.toLowerCase().includes(q))
    );
    const dir = asc ? 1 : -1;
    return [...filtered].sort((a, b) => {
      switch (key) {
        case "pid":
          return (a.pid - b.pid) * dir;
        case "name":
          return a.name.localeCompare(b.name) * (asc ? 1 : -1);
        case "user":
          return (pidUser.get(a.pid) ?? "").localeCompare(pidUser.get(b.pid) ?? "") * (asc ? 1 : -1);
        case "cpu":
          return (a.cpuPercent - b.cpuPercent) * dir;
        case "memory":
          return (a.memoryKb - b.memoryKb) * dir;
        case "status":
          return a.status.localeCompare(b.status) * (asc ? 1 : -1);
      }
    });
  }, [processes, search, key, asc, pidUser]);

  const open = (p: ProcessInfo, e: React.MouseEvent) => {
    e.preventDefault();
    setMenu({
      x: e.clientX,
      y: e.clientY,
      items: [
        { key: "end", label: "结束任务", danger: true, onClick: () => backend.endTask(p.pid) },
        { key: "loc", label: "打开文件位置", onClick: () => backend.openFileLocation(p.pid) },
        { key: "prop", label: "属性", onClick: () => backend.openProperties(p.pid) },
        { key: "search", label: "联机搜索", onClick: () => backend.searchOnline(p.name) },
        { key: "rt", label: "优先级 · 实时", onClick: () => backend.setPriority(p.pid, "realtime") },
        { key: "hi", label: "优先级 · 高", onClick: () => backend.setPriority(p.pid, "high") },
        { key: "an", label: "优先级 · 高于正常", onClick: () => backend.setPriority(p.pid, "above_normal") },
        { key: "no", label: "优先级 · 正常", onClick: () => backend.setPriority(p.pid, "normal") },
        { key: "bn", label: "优先级 · 低于正常", onClick: () => backend.setPriority(p.pid, "below_normal") },
        { key: "lo", label: "优先级 · 低", onClick: () => backend.setPriority(p.pid, "low") },
        { key: "eff", label: "效率模式", onClick: () => backend.efficiencyMode(p.pid, true) },
      ],
    });
  };

  const col = (k: Key, label: string, flex: string) => (
    <span style={{ cursor: "pointer", flex }} onClick={() => (k === key ? setAsc(!asc) : (setKey(k), setAsc(k === "pid" || k === "name")))}>
      {label} {key === k ? (asc ? "▲" : "▼") : ""}
    </span>
  );

  return (
    <div className="win-view">
      <div className="row-header" style={{ display: "flex", gap: 8, padding: "0 8px", height: 36, alignItems: "center" }}>
        {col("pid", "PID", "70px")}
        {col("name", "名称", "2 1 200px")}
        {col("user", "账户", "1 1 120px")}
        {col("cpu", "CPU", "80px")}
        {col("memory", "内存", "100px")}
        {col("status", "状态", "1 1 100px")}
      </div>
      <div className="win-view scroll-y">
        {rows.map((p) => (
          <div
            key={p.pid}
            onContextMenu={(e) => open(p, e)}
            style={{ display: "flex", gap: 8, padding: "0 8px", height: 30, alignItems: "center", fontSize: 13, borderBottom: "1px solid var(--colorNeutralStroke1)" }}
          >
            <span style={{ flex: "0 0 70px" }}>{p.pid}</span>
            <span style={{ flex: "2 1 200px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", display: "flex", alignItems: "center", gap: 6 }}>
              <AppIcon path={p.exePath} size={16} />
              {p.name}
            </span>
            <span style={{ flex: "1 1 120px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{pidUser.get(p.pid) ?? "—"}</span>
            <span style={{ flex: "0 0 80px" }}>{p.cpuPercent.toFixed(0)}%</span>
            <span style={{ flex: "0 0 100px" }}>{formatBytes(p.memoryKb)}</span>
            <span style={{ flex: "1 1 100px" }}>{p.status}</span>
          </div>
        ))}
      </div>
      <ContextMenu state={menu} onClose={() => setMenu(null)} />
    </div>
  );
}
