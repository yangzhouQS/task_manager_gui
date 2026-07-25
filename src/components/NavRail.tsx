import { Tooltip } from "@fluentui/react-components";
import {
  AppsRegular,
  GaugeRegular,
  ArrowTrendingLinesRegular,
  RocketRegular,
  PeopleRegular,
  DocumentChevronDoubleRegular,
  SettingsRegular,
} from "@fluentui/react-icons";

import { useStore } from "@/store/useStore";
import type { TabId } from "@/types";

interface NavItem {
  id: TabId;
  label: string;
  icon: React.ReactNode;
}

const ITEMS: NavItem[] = [
  { id: "processes", label: "进程", icon: <AppsRegular /> },
  { id: "performance", label: "性能", icon: <GaugeRegular /> },
  { id: "appHistory", label: "应用历史记录", icon: <ArrowTrendingLinesRegular /> },
  { id: "startup", label: "启动应用", icon: <RocketRegular /> },
  { id: "users", label: "用户", icon: <PeopleRegular /> },
  { id: "details", label: "详细信息", icon: <DocumentChevronDoubleRegular /> },
  { id: "services", label: "服务", icon: <SettingsRegular /> },
];

export default function NavRail() {
  const tab = useStore((s) => s.tab);
  const setTab = useStore((s) => s.setTab);

  return (
    <nav className="win-navrail">
      {ITEMS.map((it) => {
        const active = tab === it.id;
        return (
          <Tooltip key={it.id} content={it.label} relationship="label" positioning="after">
            <button
              onClick={() => setTab(it.id)}
              style={{
                width: 56,
                height: 52,
                margin: "2px 0",
                border: "none",
                borderRadius: 6,
                background: active ? "var(--colorSubtleBackgroundHover)" : "transparent",
                color: active ? "var(--colorBrandForeground1)" : "var(--colorNeutralForeground1)",
                cursor: "pointer",
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                gap: 2,
                fontSize: 11,
              }}
            >
              <span style={{ fontSize: 18, display: "flex" }}>{it.icon}</span>
              <span>{it.label.slice(0, 2)}</span>
            </button>
          </Tooltip>
        );
      })}
    </nav>
  );
}
