import { useMemo, useRef, useState } from "react";
import { Button } from "@fluentui/react-components";
import { useVirtualizer } from "@tanstack/react-virtual";
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
const GRID = "1fr 84px 110px 92px 92px 84px";

interface Agg {
  cpu: number;
  mem: number;
  disk: number;
  net: number;
  gpu: number;
  hung: boolean;
  exePath: string | null;
}

interface Group {
  name: string;
  kind: ProcessKind;
  children: ProcessInfo[];
  agg: Agg;
}

type Row =
  | { t: "kind"; label: string; count: number; mem: number }
  | { t: "group"; g: Group; expanded: boolean }
  | { t: "child"; p: ProcessInfo };

function aggOf(children: ProcessInfo[]): Agg {
  let cpu = 0, mem = 0, disk = 0, net = 0, gpu = 0, hung = false;
  let exePath: string | null = null;
  for (const p of children) {
    cpu += p.cpuPercent;
    mem += p.memoryKb;
    disk += p.diskMbps;
    net += p.netMbps;
    gpu += p.gpuPercent;
    if (p.status === "Not responding") hung = true;
    if (!exePath) exePath = p.exePath;
  }
  return { cpu, mem, disk, net, gpu, hung, exePath };
}

export default function ProcessesView() {
  const processes = useStore((s) => s.processes);
  const search = useStore((s) => s.search);
  const [sortKey, setSortKey] = useState<SortKey>("cpu");
  const [asc, setAsc] = useState(false);
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  const [menu, setMenu] = useState<CtxState | null>(null);
  const parentRef = useRef<HTMLDivElement>(null);

  const rows: Row[] = useMemo(() => {
    const q = search.trim().toLowerCase();
    const filtered = processes.filter((p) => !q || p.name.toLowerCase().includes(q));
    const byKind: Record<ProcessKind, Group[]> = { app: [], background: [], windows: [] };
    const map: Record<string, Record<string, ProcessInfo[]>> = { app: {}, background: {}, windows: {} };

    for (const p of filtered) {
      const key = p.name.toLowerCase();
      (map[p.kind][key] ??= []).push(p);
    }
    const cmpAgg = (a: Group, b: Group) => cmp(sortKey, asc, a.agg, b.agg, a.name, b.name);
    const cmpChild = (a: ProcessInfo, b: ProcessInfo) => cmpChildBy(sortKey, asc, a, b);

    for (const kind of KIND_ORDER) {
      const groups = Object.entries(map[kind]).map(([, children]) => ({
        name: children[0].name,
        kind,
        children: [...children].sort(cmpChild),
        agg: aggOf(children),
      }));
      groups.sort(cmpAgg);
      byKind[kind] = groups;
    }

    const out: Row[] = [];
    for (const kind of KIND_ORDER) {
      const groups = byKind[kind];
      if (groups.length === 0) continue;
      const mem = groups.reduce((s, g) => s + g.agg.mem, 0);
      out.push({ t: "kind", label: KIND_LABEL[kind], count: groups.length, mem });
      for (const g of groups) {
        const key = `${kind}|${g.name.toLowerCase()}`;
        const isOpen = !!expanded[key];
        out.push({ t: "group", g, expanded: isOpen });
        if (isOpen) for (const p of g.children) out.push({ t: "child", p });
      }
    }
    return out;
  }, [processes, search, sortKey, asc, expanded]);

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 34,
    overscan: 16,
  });

  const toggleSort = (k: SortKey) => {
    if (k === sortKey) setAsc(!asc);
    else {
      setSortKey(k);
      setAsc(false);
    }
  };

  const openMenu = (p: ProcessInfo, e: React.MouseEvent) => {
    e.preventDefault();
    if (p.pid === 0) return;
    setMenu({
      x: e.clientX,
      y: e.clientY,
      items: [
        { key: "end", label: "结束任务", danger: true, onClick: () => backend.endTask(p.pid) },
        { key: "loc", label: "打开文件位置", onClick: () => backend.openFileLocation(p.pid) },
        { key: "prop", label: "属性", onClick: () => backend.openProperties(p.pid) },
        { key: "search", label: "联机搜索", onClick: () => backend.searchOnline(p.name) },
        { key: "eff", label: "效率模式", onClick: () => backend.efficiencyMode(p.pid, true) },
      ],
    });
  };

  return (
    <div className="win-view">
      <div style={{ display: "grid", gridTemplateColumns: GRID, gap: 8, padding: "0 8px", height: 36, alignItems: "center", fontSize: 12, fontWeight: 600, borderBottom: "1px solid var(--colorNeutralStroke1Accessible)" }}>
        <Hdr k="name" sortKey={sortKey} asc={asc} onClick={toggleSort}>名称</Hdr>
        <Hdr k="cpu" sortKey={sortKey} asc={asc} onClick={toggleSort}>CPU</Hdr>
        <Hdr k="memory" sortKey={sortKey} asc={asc} onClick={toggleSort}>内存</Hdr>
        <Hdr k="disk" sortKey={sortKey} asc={asc} onClick={toggleSort}>磁盘</Hdr>
        <Hdr k="net" sortKey={sortKey} asc={asc} onClick={toggleSort}>网络</Hdr>
        <Hdr k="gpu" sortKey={sortKey} asc={asc} onClick={toggleSort}>GPU</Hdr>
      </div>

      <div ref={parentRef} style={{ flex: 1, minHeight: 0, overflow: "auto", scrollbarGutter: "stable" }}>
        <div style={{ height: virtualizer.getTotalSize(), position: "relative", width: "100%" }}>
          {virtualizer.getVirtualItems().map((vi) => {
            const row = rows[vi.index];
            return (
              <div
                key={vi.key}
                data-index={vi.index}
                ref={virtualizer.measureElement}
                style={{ position: "absolute", top: 0, left: 0, width: "100%", transform: `translateY(${vi.start}px)` }}
              >
                {renderRow(row, expanded, setExpanded, openMenu)}
              </div>
            );
          })}
        </div>
      </div>

      <ContextMenu state={menu} onClose={() => setMenu(null)} />
    </div>
  );
}

function Hdr({
  k,
  sortKey,
  asc,
  onClick,
  children,
}: {
  k: SortKey;
  sortKey: SortKey;
  asc: boolean;
  onClick: (k: SortKey) => void;
  children: React.ReactNode;
}) {
  return (
    <span style={{ cursor: "pointer" }} onClick={() => onClick(k)}>
      {children} {sortKey === k ? (asc ? "▲" : "▼") : ""}
    </span>
  );
}

function renderRow(
  row: Row,
  expanded: Record<string, boolean>,
  setExpanded: (v: Record<string, boolean>) => void,
  onMenu: (p: ProcessInfo, e: React.MouseEvent) => void
) {
  const cellBase: React.CSSProperties = { fontSize: 13, display: "flex", alignItems: "center" };

  if (row.t === "kind") {
    return (
      <div style={{ display: "flex", alignItems: "center", gap: 6, height: 30, padding: "0 8px", fontSize: 13, fontWeight: 600, marginTop: 2 }}>
        <span>{row.label}</span>
        <span className="muted" style={{ fontWeight: 400 }}>
          ({row.count}) · {formatBytes(row.mem)}
        </span>
      </div>
    );
  }

  if (row.t === "group") {
    const { g } = row;
    const key = `${g.kind}|${g.name.toLowerCase()}`;
    const multi = g.children.length > 1;
    return (
      <div
        style={{ display: "grid", gridTemplateColumns: GRID, gap: 8, padding: "0 8px", height: 34, alignItems: "center", borderBottom: "1px solid var(--colorNeutralStroke1)" }}
        onContextMenu={(e) => onMenu(g.children[0], e)}
      >
        <span style={{ ...cellBase, gap: 4, minWidth: 0 }}>
          {multi ? (
            <Button
              appearance="subtle"
              size="small"
              icon={row.expanded ? <ChevronDownRegular /> : <ChevronRightRegular />}
              onClick={() => setExpanded({ ...expanded, [key]: !row.expanded })}
            />
          ) : (
            <span style={{ width: 24 }} />
          )}
          <AppIcon path={g.agg.exePath} kind={g.kind} />
          <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: g.agg.hung ? "var(--colorPaletteRedForeground1)" : undefined }}>{g.name}</span>
          {multi && <span className="muted" style={{ fontSize: 11 }}>({g.children.length})</span>}
        </span>
        <ValueCell v={`${g.agg.cpu.toFixed(0)}%`} />
        <ValueCell v={formatBytes(g.agg.mem)} />
        <ValueCell v={formatSpeed(g.agg.disk)} />
        <ValueCell v={formatSpeed(g.agg.net)} />
        <ValueCell v={`${g.agg.gpu.toFixed(0)}%`} />
      </div>
    );
  }

  const p = row.p;
  return (
    <div
      style={{ display: "grid", gridTemplateColumns: GRID, gap: 8, padding: "0 8px 0 40px", height: 30, alignItems: "center", fontSize: 13, borderBottom: "1px solid var(--colorNeutralStroke1)" }}
      onContextMenu={(e) => onMenu(p, e)}
      onDoubleClick={() => backend.openProperties(p.pid)}
    >
      <span style={{ ...cellBase, gap: 8, minWidth: 0 }}>
        <AppIcon path={p.exePath} kind={p.kind} size={14} />
        <span className="muted" style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", color: p.status === "Not responding" ? "var(--colorPaletteRedForeground1)" : undefined }}>{p.name}</span>
      </span>
      <ValueCell v={`${p.cpuPercent.toFixed(0)}%`} />
      <ValueCell v={formatBytes(p.memoryKb)} />
      <ValueCell v={formatSpeed(p.diskMbps)} />
      <ValueCell v={formatSpeed(p.netMbps)} />
      <ValueCell v={`${p.gpuPercent.toFixed(0)}%`} />
    </div>
  );
}

function ValueCell({ v }: { v: string }) {
  return <span style={{ fontSize: 13 }}>{v}</span>;
}

function cmp(key: SortKey, asc: boolean, a: Agg, b: Agg, na: string, nb: string): number {
  const d = asc ? 1 : -1;
  switch (key) {
    case "name":
      return na.localeCompare(nb) * (asc ? 1 : -1);
    case "cpu":
      return (a.cpu - b.cpu) * d;
    case "memory":
      return (a.mem - b.mem) * d;
    case "disk":
      return (a.disk - b.disk) * d;
    case "net":
      return (a.net - b.net) * d;
    case "gpu":
      return (a.gpu - b.gpu) * d;
  }
}

function cmpChildBy(key: SortKey, asc: boolean, a: ProcessInfo, b: ProcessInfo): number {
  const d = asc ? 1 : -1;
  switch (key) {
    case "name":
      return a.name.localeCompare(b.name) * (asc ? 1 : -1);
    case "cpu":
      return (a.cpuPercent - b.cpuPercent) * d;
    case "memory":
      return (a.memoryKb - b.memoryKb) * d;
    case "disk":
      return (a.diskMbps - b.diskMbps) * d;
    case "net":
      return (a.netMbps - b.netMbps) * d;
    case "gpu":
      return (a.gpuPercent - b.gpuPercent) * d;
  }
}
