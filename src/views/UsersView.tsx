import { useState } from "react";
import { Button } from "@fluentui/react-components";
import { ChevronDownRegular, ChevronRightRegular } from "@fluentui/react-icons";
import { backend, formatBytes } from "@/lib/backend";
import { useStore } from "@/store/useStore";
import type { ProcessInfo } from "@/types";

export default function UsersView() {
  const sessions = useStore((s) => s.users);
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

  return (
    <div className="win-view">
      <div className="row-header" style={{ display: "flex", gap: 8, padding: "0 8px", height: 34, alignItems: "center" }}>
        <span style={{ flex: "2 1 200px" }}>名称</span>
        <span style={{ flex: "0 0 80px" }}>CPU</span>
        <span style={{ flex: "0 0 100px" }}>内存</span>
        <span style={{ flex: "1 1 100px" }}>会话</span>
      </div>
      <div className="win-view scroll-y">
        {sessions.map((u) => {
          const isCol = collapsed[u.user + u.sessionId];
          const mem = u.processes.reduce((s, p) => s + p.memoryKb, 0);
          const cpu = u.processes.reduce((s, p) => s + p.cpuPercent, 0);
          return (
            <div key={u.user + u.sessionId}>
              <div className="group-header">
                <Button
                  appearance="subtle"
                  size="small"
                  icon={isCol ? <ChevronRightRegular /> : <ChevronDownRegular />}
                  onClick={() => setCollapsed({ ...collapsed, [u.user + u.sessionId]: !isCol })}
                />
                <span>{u.user}</span>
                {u.isCurrent && <span className="muted">(当前)</span>}
                <span className="muted" style={{ fontWeight: 400 }}>
                  CPU {cpu.toFixed(0)}% · {formatBytes(mem)} · 会话 {u.sessionId}
                </span>
              </div>
              {!isCol &&
                u.processes.slice(0, 50).map((p) => <UserProcRow key={p.pid} p={p} sessionId={u.sessionId} />)}
            </div>
          );
        })}
        {sessions.length === 0 && <div className="muted" style={{ padding: 16 }}>加载中…</div>}
      </div>
    </div>
  );
}

function UserProcRow({ p, sessionId }: { p: ProcessInfo; sessionId: number }) {
  void sessionId;
  return (
    <div
      onContextMenu={(e) => {
        e.preventDefault();
        backend.endTask(p.pid);
      }}
      style={{ display: "flex", gap: 8, padding: "0 8px", height: 28, alignItems: "center", fontSize: 13, borderBottom: "1px solid var(--colorNeutralStroke1)" }}
    >
      <span style={{ flex: "2 1 200px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{p.name}</span>
      <span style={{ flex: "0 0 80px" }}>{p.cpuPercent.toFixed(0)}%</span>
      <span style={{ flex: "0 0 100px" }}>{formatBytes(p.memoryKb)}</span>
      <span style={{ flex: "1 1 100px" }}>{sessionId}</span>
    </div>
  );
}
