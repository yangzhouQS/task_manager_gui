import { useMemo, useState } from "react";
import { Button, Input } from "@fluentui/react-components";
import { ArrowClockwiseRegular, WindowConsoleRegular } from "@fluentui/react-icons";
import { backend } from "@/lib/backend";
import { useStore } from "@/store/useStore";
import type { ServiceInfo } from "@/types";
import ContextMenu, { type CtxState } from "@/components/ContextMenu";
import AppIcon from "@/components/AppIcon";

export default function ServicesView() {
  const services = useStore((s) => s.services);
  const [filter, setFilter] = useState("");
  const [menu, setMenu] = useState<CtxState | null>(null);

  const rows = useMemo(() => {
    const q = filter.trim().toLowerCase();
    return [...services]
      .filter((s) => !q || s.name.toLowerCase().includes(q) || s.displayName.toLowerCase().includes(q))
      .sort((a, b) => a.displayName.localeCompare(b.displayName));
  }, [services, filter]);

  const open = (s: ServiceInfo, e: React.MouseEvent) => {
    e.preventDefault();
    const running = s.state === "Running";
    setMenu({
      x: e.clientX,
      y: e.clientY,
      items: [
        { key: "start", label: "启动", disabled: running, onClick: () => backend.controlService(s.name, "start") },
        { key: "stop", label: "停止", disabled: !running, onClick: () => backend.controlService(s.name, "stop") },
        { key: "restart", label: "重新启动", disabled: !running, onClick: () => backend.controlService(s.name, "restart") },
        { key: "msc", label: "打开“服务”管理器", onClick: () => backend.openServicesMsc() },
      ],
    });
  };

  return (
    <div className="win-view">
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
        <Input
          size="small"
          placeholder="按名称筛选"
          value={filter}
          onChange={(_, d) => setFilter(d.value)}
          style={{ width: 220 }}
        />
        <Button
          size="small"
          appearance="subtle"
          icon={<ArrowClockwiseRegular />}
          onClick={() => backend.refreshServices()}
        >
          刷新
        </Button>
        <Button
          size="small"
          appearance="subtle"
          icon={<WindowConsoleRegular />}
          onClick={() => backend.openServicesMsc()}
        >
          打开服务管理器
        </Button>
      </div>
      <div className="row-header" style={{ display: "flex", gap: 8, padding: "0 8px", height: 34, alignItems: "center" }}>
        <span style={{ flex: "0 0 160px" }}>名称</span>
        <span style={{ flex: "2 1 200px" }}>显示名称</span>
        <span style={{ flex: "0 0 90px" }}>状态</span>
        <span style={{ flex: "0 0 130px" }}>启动类型</span>
        <span style={{ flex: "1 1 120px" }}>PID</span>
      </div>
      <div className="win-view scroll-y">
        {rows.map((s) => (
          <div
            key={s.name}
            onContextMenu={(e) => open(s, e)}
            title={s.description ?? ""}
            style={{ display: "flex", gap: 8, padding: "0 8px", height: 28, alignItems: "center", fontSize: 13, borderBottom: "1px solid var(--colorNeutralStroke1)" }}
          >
            <span style={{ flex: "0 0 160px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", display: "flex", alignItems: "center", gap: 6 }}>
              <AppIcon path={s.exePath} size={16} />
              {s.name}
            </span>
            <span style={{ flex: "2 1 200px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{s.displayName}</span>
            <span style={{ flex: "0 0 90px", color: s.state === "Running" ? "var(--colorPaletteGreenForeground1)" : s.state === "Stopped" ? "var(--colorNeutralForeground3)" : undefined }}>{s.state}</span>
            <span style={{ flex: "0 0 130px" }}>{s.startType}</span>
            <span style={{ flex: "1 1 120px" }}>{s.pid || "—"}</span>
          </div>
        ))}
      </div>
      <ContextMenu state={menu} onClose={() => setMenu(null)} />
    </div>
  );
}
