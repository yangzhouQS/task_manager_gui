import { useStore } from "@/store/useStore";
import { formatBytes } from "@/lib/backend";

export default function StatusBar() {
  const procs = useStore((s) => s.processes);
  const perf = useStore((s) => s.performance);
  const count = procs.filter((p) => p.pid !== 0).length;
  const cpu = perf?.cpuPercent ?? 0;
  const memInUse = perf?.memInUseKb ?? 0;
  const memTotal = perf?.memTotalKb ?? 0;

  return (
    <div className="win-statusbar">
      <span>进程 {count}</span>
      <span>CPU {cpu.toFixed(0)}%</span>
      <span>
        内存 {formatBytes(memInUse)} / {formatBytes(memTotal)}
      </span>
    </div>
  );
}
