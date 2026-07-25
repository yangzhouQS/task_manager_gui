import { useEffect, useState } from "react";
import { Button } from "@fluentui/react-components";

export interface CtxItem {
  key: string;
  label: string;
  icon?: React.ReactElement;
  danger?: boolean;
  disabled?: boolean;
  onClick: () => void;
}

export interface CtxState {
  x: number;
  y: number;
  items: CtxItem[];
}

export default function ContextMenu({
  state,
  onClose,
}: {
  state: CtxState | null;
  onClose: () => void;
}) {
  const [, force] = useState(0);
  useEffect(() => {
    if (state) force((n) => n + 1);
  }, [state]);

  if (!state) return null;

  const w = window.innerWidth;
  const h = window.innerHeight;
  const left = Math.min(state.x, w - 220);
  const top = Math.min(state.y, h - state.items.length * 34 - 16);

  return (
    <div
      style={{ position: "fixed", inset: 0, zIndex: 1000 }}
      onClick={onClose}
      onContextMenu={(e) => {
        e.preventDefault();
        onClose();
      }}
    >
      <div
        style={{
          position: "absolute",
          left,
          top,
          minWidth: 200,
          background: "var(--colorNeutralBackground1)",
          border: "1px solid var(--colorNeutralStroke1)",
          borderRadius: 7,
          boxShadow: "0 8px 24px rgba(0,0,0,0.18)",
          padding: 4,
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {state.items.map((it) => (
          <Button
            key={it.key}
            appearance="subtle"
            disabled={it.disabled}
            icon={it.icon}
            style={{
              width: "100%",
              justifyContent: "flex-start",
              color: it.danger ? "var(--colorPaletteRedForeground1)" : undefined,
            }}
            onClick={() => {
              it.onClick();
              onClose();
            }}
          >
            {it.label}
          </Button>
        ))}
      </div>
    </div>
  );
}
