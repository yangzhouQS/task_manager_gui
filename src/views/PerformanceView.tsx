import { useEffect, useState } from "react";
import {
  Button,
  Dropdown,
  Option,
} from "@fluentui/react-components";
import {
  GaugeRegular,
  DesktopMacRegular,
  HardDriveRegular,
  Wifi1Regular,
  DeviceMeetingRoomRegular,
  GridRegular,
  ArrowClockwiseRegular,
} from "@fluentui/react-icons";

import LineChart from "@/components/LineChart";
import ContextMenu, { type CtxState } from "@/components/ContextMenu";
import { useStore, type RefreshSpeed } from "@/store/useStore";
import { formatBytes, formatDuration, formatMbps, formatSpeed } from "@/lib/backend";
import type { PerformanceSnapshot } from "@/types";

const MAX_POINTS = 60;

interface History {
  cpu: number[];
  mem: number[];
  disk: Record<string, number[]>;
  net: Record<string, number[]>;
  gpu: Record<string, number[]>;
}

const empty: History = { cpu: [], mem: [], disk: {}, net: {}, gpu: {} };

const REFRESH_LABEL: Record<RefreshSpeed, string> = {
  high: "高",
  normal: "正常",
  low: "低",
  paused: "暂停",
};

function push(arr: number[], v: number): number[] {
  const next = [...arr, v];
  return next.length > MAX_POINTS ? next.slice(next.length - MAX_POINTS) : next;
}

export default function PerformanceView() {
  const perf = useStore((s) => s.performance);
  const refresh = useStore((s) => s.refresh);
  const setRefresh = useStore((s) => s.setRefresh);
  const [hist, setHist] = useState<History>(empty);
  const [selected, setSelected] = useState("cpu");
  const [summary, setSummary] = useState(false);
  const [menu, setMenu] = useState<CtxState | null>(null);

  useEffect(() => {
    if (!perf) return;
    setHist((h) => {
      const memPct = perf.memTotalKb ? (perf.memInUseKb / perf.memTotalKb) * 100 : 0;
      const disk: Record<string, number[]> = { ...h.disk };
      for (const d of perf.diskUsage) disk[d.name] = push(h.disk[d.name] ?? [], d.percent);
      const net: Record<string, number[]> = { ...h.net };
      for (const n of perf.netUsage)
        net[n.name] = push(h.net[n.name] ?? [], n.sendMbps + n.recvMbps);
      const gpu: Record<string, number[]> = { ...h.gpu };
      for (const g of perf.gpuUsage) gpu[g.name] = push(h.gpu[g.name] ?? [], g.percent);
      return {
        cpu: push(h.cpu, perf.cpuPercent),
        mem: push(h.mem, memPct),
        disk,
        net,
        gpu,
      };
    });
  }, [perf]);

  if (!perf) return null;

  const onContext = (e: React.MouseEvent) => {
    e.preventDefault();
    const mark = (active: boolean) => (active ? "✓ " : "   ");
    setMenu({
      x: e.clientX,
      y: e.clientY,
      items: [
        {
          key: "summary",
          label: `${mark(summary)}摘要视图`,
          onClick: () => setSummary((v) => !v),
        },
        { key: "rh", label: `${mark(refresh === "high")}刷新速度 · 高`, onClick: () => setRefresh("high") },
        { key: "rn", label: `${mark(refresh === "normal")}刷新速度 · 正常`, onClick: () => setRefresh("normal") },
        { key: "rl", label: `${mark(refresh === "low")}刷新速度 · 低`, onClick: () => setRefresh("low") },
        { key: "rp", label: `${mark(refresh === "paused")}刷新速度 · 暂停`, onClick: () => setRefresh("paused") },
      ],
    });
  };

  return (
    <div className="win-view" onContextMenu={onContext}>
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8, flex: "0 0 auto" }}>
        <Button
          appearance={summary ? "primary" : "subtle"}
          size="small"
          icon={<GridRegular />}
          onClick={() => setSummary((v) => !v)}
        >
          摘要视图
        </Button>
        <span style={{ display: "inline-flex", alignItems: "center", gap: 4, fontSize: 13 }}>
          <ArrowClockwiseRegular />
          刷新速度
        </span>
        <Dropdown
          size="small"
          style={{ minWidth: 100 }}
          selectedOptions={[refresh]}
          onOptionSelect={(_, d) => d.optionValue && setRefresh(d.optionValue as RefreshSpeed)}
        >
          <Option value="high">{REFRESH_LABEL.high}</Option>
          <Option value="normal">{REFRESH_LABEL.normal}</Option>
          <Option value="low">{REFRESH_LABEL.low}</Option>
          <Option value="paused">{REFRESH_LABEL.paused}</Option>
        </Dropdown>
        {refresh === "paused" && (
          <span className="muted" style={{ fontSize: 12 }}>已暂停</span>
        )}
      </div>

      {summary ? (
        <SummaryGrid perf={perf} hist={hist} />
      ) : (
        <div className="perf-grid">
          <div className="perf-side scroll-y">
            {buildSide(perf).map((it) => (
              <button
                key={it.key}
                onClick={() => setSelected(it.key)}
                className={selected === it.key ? "perf-nav-item active" : "perf-nav-item"}
              >
                <span className="perf-nav-icon">{it.icon}</span>
                <span className="perf-nav-label">{it.label}</span>
              </button>
            ))}
          </div>
          <div className="perf-main">{renderMain(selected, perf, hist)}</div>
        </div>
      )}

      <ContextMenu state={menu} onClose={() => setMenu(null)} />
    </div>
  );
}

interface SideItem {
  key: string;
  label: string;
  icon: React.ReactNode;
}

function buildSide(perf: PerformanceSnapshot): SideItem[] {
  const items: SideItem[] = [
    { key: "cpu", label: "CPU", icon: <GaugeRegular /> },
    { key: "memory", label: "内存", icon: <DesktopMacRegular /> },
  ];
  const seen = new Set<string>();
  for (const d of perf.diskUsage) {
    if (seen.has(d.name)) continue;
    seen.add(d.name);
    items.push({ key: `disk:${d.name}`, label: `磁盘 ${d.name}`, icon: <HardDriveRegular /> });
  }
  for (const n of perf.netUsage) {
    if (seen.has(n.name)) continue;
    seen.add(n.name);
    items.push({ key: `net:${n.name}`, label: n.name, icon: <Wifi1Regular /> });
  }
  for (const g of perf.gpuUsage) {
    if (seen.has(g.name)) continue;
    seen.add(g.name);
    items.push({ key: `gpu:${g.name}`, label: g.name, icon: <DeviceMeetingRoomRegular /> });
  }
  return items;
}

function renderMain(selected: string, perf: PerformanceSnapshot, hist: History) {
  if (selected === "cpu") {
    return (
      <Section title="CPU" big={<LineChart data={hist.cpu} max={100} color="#0f6cbd" height={200} />}>
        <Stat label="利用率" value={`${perf.cpuPercent.toFixed(0)}%`} />
        <Stat label="速度" value={`${perf.cpuSpeedGhz.toFixed(2)} GHz`} />
        <Stat label="逻辑处理器" value={`${perf.cpuCoreCount}`} />
        <Stat label="运行时间" value={formatDuration(perf.cpuUpTimeSec)} />
      </Section>
    );
  }
  if (selected === "memory") {
    const memPct = perf.memTotalKb ? (perf.memInUseKb / perf.memTotalKb) * 100 : 0;
    return (
      <Section title="内存" big={<LineChart data={hist.mem} max={100} color="#8764b8" height={200} />}>
        <Stat label="使用中" value={`${memPct.toFixed(0)}% (${formatBytes(perf.memInUseKb)})`} />
        <Stat label="可用" value={formatBytes(perf.memAvailableKb)} />
        <Stat label="已提交" value={`${formatBytes(perf.memCommittedKb)} / ${formatBytes(perf.memCommittedLimitKb)}`} />
        <Stat label="缓存" value={formatBytes(perf.memCachedKb)} />
        <Stat label="分页池" value={formatBytes(perf.memPagedPoolKb)} />
        <Stat label="非分页池" value={formatBytes(perf.memNonPagedPoolKb)} />
      </Section>
    );
  }
  if (selected.startsWith("disk:")) {
    const name = selected.slice(5);
    const d = perf.diskUsage.find((x) => x.name === name);
    if (!d) return null;
    return (
      <Section title={`磁盘 ${d.name}`} big={<LineChart data={hist.disk[name] ?? []} max={100} color="#107c10" height={200} />}>
        <Stat label="活动时间" value={`${d.percent.toFixed(0)}%`} />
        <Stat label="读取速度" value={formatSpeed(d.readMbps)} />
        <Stat label="写入速度" value={formatSpeed(d.writeMbps)} />
      </Section>
    );
  }
  if (selected.startsWith("net:")) {
    const name = selected.slice(4);
    const n = perf.netUsage.find((x) => x.name === name);
    if (!n) return null;
    return (
      <Section title={n.name} big={<LineChart data={hist.net[name] ?? []} color="#ca5010" height={200} />}>
        <Stat label="发送" value={formatMbps(n.sendMbps)} />
        <Stat label="接收" value={formatMbps(n.recvMbps)} />
      </Section>
    );
  }
  if (selected.startsWith("gpu:")) {
    const name = selected.slice(4);
    const g = perf.gpuUsage.find((x) => x.name === name);
    if (!g) return null;
    return (
      <Section title={g.name} big={<LineChart data={hist.gpu[name] ?? []} max={100} color="#038387" height={200} />}>
        <Stat label="利用率" value={`${g.percent.toFixed(0)}%`} />
        <Stat label="专用 GPU 内存" value={formatBytes(g.dedicatedKb)} />
        <Stat label="共享 GPU 内存" value={formatBytes(g.sharedKb)} />
      </Section>
    );
  }
  return null;
}

interface Tile {
  name: string;
  color: string;
  data: number[];
  max?: number;
  stat: string;
}

function SummaryGrid({ perf, hist }: { perf: PerformanceSnapshot; hist: History }) {
  const memPct = perf.memTotalKb ? (perf.memInUseKb / perf.memTotalKb) * 100 : 0;
  const tiles: Tile[] = [
    { name: "CPU", color: "#0f6cbd", data: hist.cpu, max: 100, stat: `${perf.cpuPercent.toFixed(0)}%` },
    { name: "内存", color: "#8764b8", data: hist.mem, max: 100, stat: `${memPct.toFixed(0)}%` },
  ];
  for (const d of perf.diskUsage)
    tiles.push({ name: `磁盘 ${d.name}`, color: "#107c10", data: hist.disk[d.name] ?? [], max: 100, stat: `${d.percent.toFixed(0)}%` });
  for (const n of perf.netUsage)
    tiles.push({ name: n.name, color: "#ca5010", data: hist.net[n.name] ?? [], stat: `${formatMbps(n.sendMbps)} / ${formatMbps(n.recvMbps)}` });
  for (const g of perf.gpuUsage)
    tiles.push({ name: g.name, color: "#038387", data: hist.gpu[g.name] ?? [], max: 100, stat: `${g.percent.toFixed(0)}%` });

  return (
    <div style={{ flex: 1, minHeight: 0, overflow: "auto", scrollbarGutter: "stable" }}>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))", gap: 10 }}>
        {tiles.map((t) => (
          <div key={t.name} className="perf-chart-box" style={{ width: "auto" }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", marginBottom: 4 }}>
              <span style={{ fontSize: 13, fontWeight: 600 }}>{t.name}</span>
              <span style={{ fontSize: 12, color: t.color }}>{t.stat}</span>
            </div>
            <LineChart data={t.data} color={t.color} max={t.max} height={56} />
          </div>
        ))}
      </div>
    </div>
  );
}

function Section({ title, big, children }: { title: string; big: React.ReactNode; children: React.ReactNode }) {
  return (
    <>
      <h2 style={{ margin: "4px 0", fontSize: 18, color: "var(--colorNeutralForeground1)" }}>{title}</h2>
      <div className="perf-chart-box">{big}</div>
      <div className="perf-stat-grid">{children}</div>
    </>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="perf-stat">
      <span className="muted">{label}</span>
      <span>{value}</span>
    </div>
  );
}
