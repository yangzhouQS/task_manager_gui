import { useStore } from "@/store/useStore";
import type { ProcessKind } from "@/types";

interface AppIconProps {
  path?: string | null;
  direct?: string | null;
  kind?: ProcessKind;
  size?: number;
}

const KIND_COLOR: Record<ProcessKind, string> = {
  app: "#0f6cbd",
  background: "#8764b8",
  windows: "#6b6b6b",
};

export default function AppIcon({ path, direct, kind, size = 16 }: AppIconProps) {
  const cached = useStore((s) => (path ? s.iconMap[path] : undefined));
  const src = direct ?? cached;
  if (src) {
    return (
      <img
        src={src}
        width={size}
        height={size}
        alt=""
        style={{ width: size, height: size, borderRadius: 3, flex: "0 0 auto", objectFit: "contain" }}
      />
    );
  }
  const color = kind ? KIND_COLOR[kind] : "#6b6b6b";
  return (
    <span
      style={{
        width: size,
        height: size,
        borderRadius: 3,
        background: color,
        flex: "0 0 auto",
        display: "inline-block",
      }}
    />
  );
}
