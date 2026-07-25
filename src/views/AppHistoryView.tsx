import { Button } from "@fluentui/react-components";
import { backend, formatCpuTime } from "@/lib/backend";
import { useStore } from "@/store/useStore";

export default function AppHistoryView() {
  const history = useStore((s) => s.appHistory);

  const reset = async () => {
    await backend.resetAppHistory();
  };

  return (
    <div className="win-view">
      <div style={{ marginBottom: 8, display: "flex", justifyContent: "flex-end" }}>
        <Button size="small" appearance="subtle" onClick={reset}>
          删除使用历史记录
        </Button>
      </div>
      <div className="row-header" style={{ display: "flex", gap: 8, padding: "0 8px", height: 34, alignItems: "center" }}>
        <span style={{ flex: "2 1 200px" }}>名称</span>
        <span style={{ flex: "0 0 120px" }}>CPU 时间</span>
        <span style={{ flex: "1 1 120px" }}>网络</span>
        <span style={{ flex: "1 1 120px" }}>按流量计费网络</span>
        <span style={{ flex: "0 0 90px" }}>磁贴更新</span>
      </div>
      <div className="win-view scroll-y">
        {history.map((h) => (
          <div
            key={h.name}
            style={{ display: "flex", gap: 8, padding: "0 8px", height: 28, alignItems: "center", fontSize: 13, borderBottom: "1px solid var(--colorNeutralStroke1)" }}
          >
            <span style={{ flex: "2 1 200px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{h.name}</span>
            <span style={{ flex: "0 0 120px" }}>{formatCpuTime(h.totalCpuSec)}</span>
            <span style={{ flex: "1 1 120px" }}>{h.networkMb.toFixed(1)} MB</span>
            <span style={{ flex: "1 1 120px" }}>{h.meteredMb.toFixed(1)} MB</span>
            <span style={{ flex: "0 0 90px" }} className="muted">{h.tileUpdates}</span>
          </div>
        ))}
        {history.length === 0 && <div className="muted" style={{ padding: 16 }}>暂无记录（将随使用累积）</div>}
      </div>
    </div>
  );
}
