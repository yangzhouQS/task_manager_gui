import { useEffect, useState } from "react";
import { Button } from "@fluentui/react-components";
import { backend } from "@/lib/backend";
import type { StartupEntry } from "@/types";

export default function StartupView() {
  const [entries, setEntries] = useState<StartupEntry[]>([]);
  const [overrides, setOverrides] = useState<Record<string, boolean>>({});

  useEffect(() => {
    backend.getStartup().then(setEntries);
  }, []);

  const refresh = () => {
    setOverrides({});
    backend.getStartup().then(setEntries);
  };

  const effective = (e: StartupEntry) =>
    overrides[e.name + e.source] ?? e.enabled;

  return (
    <div className="win-view">
      <div style={{ marginBottom: 8 }}>
        <Button size="small" appearance="subtle" onClick={refresh}>
          刷新
        </Button>
      </div>
      <div className="row-header" style={{ display: "flex", gap: 8, padding: "0 8px", height: 34, alignItems: "center" }}>
        <span style={{ flex: "0 0 90px" }}>状态</span>
        <span style={{ flex: "1 1 160px" }}>名称</span>
        <span style={{ flex: "2 1 220px" }}>命令</span>
        <span style={{ flex: "1 1 120px" }}>来源</span>
        <span style={{ flex: "0 0 100px" }}>启动影响</span>
      </div>
      <div className="win-view scroll-y">
        {entries.map((e) => (
          <div
            key={e.name + e.source}
            style={{ display: "flex", gap: 8, padding: "0 8px", height: 30, alignItems: "center", fontSize: 13, borderBottom: "1px solid var(--colorNeutralStroke1)" }}
          >
            <span style={{ flex: "0 0 90px", color: effective(e) ? "#107c10" : "#c50f1f" }}>
              {effective(e) ? "已启用" : "已禁用"}
            </span>
            <span style={{ flex: "1 1 160px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{e.name}</span>
            <span style={{ flex: "2 1 220px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} className="muted">{e.command}</span>
            <span style={{ flex: "1 1 120px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{e.source}</span>
            <span style={{ flex: "0 0 100px" }} className="muted">{e.impact}</span>
          </div>
        ))}
        {entries.length === 0 && (
          <div className="muted" style={{ padding: 16 }}>未发现启动项</div>
        )}
      </div>
    </div>
  );
}
