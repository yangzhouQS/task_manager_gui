import { useMemo, useState } from "react";
import { Button } from "@fluentui/react-components";
import { ChevronDownRegular, ChevronRightRegular } from "@fluentui/react-icons";

import { backend, formatBytes, formatSpeed } from "@/lib/backend";
import { useStore } from "@/store/useStore";
import type { ProcessInfo, ProcessKind } from "@/types";
import ContextMenu, { type CtxState } from "@/components/ContextMenu";
import AppIcon from "@/components/AppIcon";

type SortKey = "name" | "cpu" | "memory" | "disk" | "net" | "gpu";

const KIND_LABEL: Record<ProcessKind, string> = {
  app: "应用进程",
  background: "后台进程",
  windows: "Windows 进程",
};
const KIND_ORDER: ProcessKind[] = ["app", "background", "windows"];

export default function ProcessesView() {
  const processes = useStore((s) => s.processes);
  const search = useStore((s) => s.search);
  const [sortKey, setSortKey] = useState<SortKey>("cpu");
  const [asc, setAsc] = useState(false);
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});
  const [menu, setMenu] = useState<CtxState | null>(null);

  const grouped = useMemo(() => {
    const q = search.trim().toLowerCase();
    const filtered = processes.filter(
      (p) => !q || p.name.toLowerCase().includes(q)
    );
    const sorted = [...filtered].sort((a, b) => {
      const dir = asc ? 1 : -1;
      switch (sortKey) {
        case "name":
          return a.name.localeCompare(b.name) * (asc ? 1 : -1);
        case "cpu":
          return (a.cpuPercent - b.cpuPercent) * dir;
        case "memory":
          return (a.memoryKb - b.memoryKb) * dir;
        case "disk":
          return (a.diskMbps - b.diskMbps) * dir;
        case "net":
          return (a.netMbps - b.netMbps) * dir;
        case "gpu":
          return (a.gpuPercent - b.gpuPercent) * dir;
      }
    });
    const map: Record<ProcessKind, ProcessInfo[]> = {
      app: [],
      background: [],
      windows: [],
    };
    for (const p of sorted) map[p.kind].push(p);
    return map;
  }, [processes, search, sortKey, asc]);

  const toggleSort = (k: SortKey) => {
    if (k === sortKey) setAsc(!asc);
    else {
      setSortKey(k);
      setAsc(false);
    }
  };

  const openMenu = (p: ProcessInfo, e: React.MouseEvent) => {
    e.preventDefault();
    setMenu({
      x: e.clientX,
      y: e.clientY,
      items:
        p.pid === 0
          ? []
          : [
              {
                key: "end",
                label: "结束任务",
                danger: true,
                onClick: () => backend.endTask(p.pid),
              },
              {
                key: "loc",
                label: "打开文件位置",
                onClick: () => backend.openFileLocation(p.pid),
              },
              { key: "prop", label: "属性", onClick: () => backend.openProperties(p.pid) },
              {
                key: "search",
                label: "联机搜索",
                onClick: () => backend.searchOnline(p.name),
              },
              {
                key: "eff",
                label: "效率模式",
                onClick: () => backend.efficiencyMode(p.pid, true),
              },
            ],
    });
  };

  return (
    <div className="win-view">
      <div className="row-grid row-header">
        <span onClick={() => toggleSort("name")} style={{ cursor: "pointer" }}>
          名称 {sortKey === "name" ? (asc ? "▲" : "▼") : ""}
        </span>
        <span onClick={() => toggleSort("cpu")} style={{ cursor: "pointer" }}>
          CPU {sortKey === "cpu" ? (asc ? "▲" : "▼") : ""}
        </span>
        <span onClick={() => toggleSort("memory")} style={{ cursor: "pointer" }}>
          内存 {sortKey === "memory" ? (asc ? "▲" : "▼") : ""}
        </span>
        <span onClick={() => toggleSort("disk")} style={{ cursor: "pointer" }}>
          磁盘 {sortKey === "disk" ? (asc ? "▲" : "▼") : ""}
        </span>
        <span onClick={() => toggleSort("net")} style={{ cursor: "pointer" }}>
          网络 {sortKey === "net" ? (asc ? "▲" : "▼") : ""}
        </span>
        <span onClick={() => toggleSort("gpu")} style={{ cursor: "pointer" }}>
          GPU {sortKey === "gpu" ? (asc ? "▲" : "▼") : ""}
        </span>
      </div>

      <div className="win-view scroll-y">
        {KIND_ORDER.map((kind) => {
          const list = grouped[kind];
          if (list.length === 0) return null;
          const isCol = collapsed[kind];
          const totalMem = list.reduce((s, p) => s + p.memoryKb, 0);
          return (
            <div key={kind}>
              <div className="group-header">
                <Button
                  appearance="subtle"
                  size="small"
                  icon={isCol ? <ChevronRightRegular /> : <ChevronDownRegular />}
                  onClick={() => setCollapsed({ ...collapsed, [kind]: !isCol })}
                />
                <span>{KIND_LABEL[kind]}</span>
                <span className="muted" style={{ fontWeight: 400 }}>
                  ({list.length}) · {formatBytes(totalMem)}
                </span>
              </div>
              {!isCol &&
                list.map((p) => (
                  <ProcessRow key={`${p.kind}-${p.pid}`} p={p} onMenu={openMenu} />
                ))}
            </div>
          );
        })}
      </div>

      <ContextMenu state={menu} onClose={() => setMenu(null)} />
    </div>
  );
}

function ProcessRow({
  p,
  onMenu,
}: {
  p: ProcessInfo;
  onMenu: (p: ProcessInfo, e: React.MouseEvent) => void;
}) {
  return (
    <div
      className="row-grid"
      style={{ cursor: "default" }}
      onContextMenu={(e) => onMenu(p, e)}
      onDoubleClick={() => p.pid !== 0 && backend.openProperties(p.pid)}
    >
      <span className="proc-name">
        <AppIcon path={p.exePath} kind={p.kind} />
        <span className="proc-name-text">{p.name}</span>
        {p.groupCount > 1 && (
          <span className="muted" style={{ fontSize: 11 }}>
            ({p.groupCount})
          </span>
        )}
      </span>
      <span className="row-cell">{p.cpuPercent.toFixed(0)}%</span>
      <span className="row-cell">{formatBytes(p.memoryKb)}</span>
      <span className="row-cell">{formatSpeed(p.diskMbps)}</span>
      <span className="row-cell">{formatSpeed(p.netMbps)}</span>
      <span className="row-cell">{p.gpuPercent.toFixed(0)}%</span>
    </div>
  );
}
