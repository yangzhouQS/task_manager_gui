import { useState } from "react";
import { Button, Input, Tooltip, Dialog } from "@fluentui/react-components";
import {
  SearchRegular,
  RocketRegular,
  PinRegular,
  PinOffRegular,
  WeatherMoonRegular,
  WeatherSunnyRegular,
  ShieldKeyholeRegular,
  SubtractRegular,
  MaximizeRegular,
  DismissRegular,
} from "@fluentui/react-icons";

import { backend } from "@/lib/backend";
import { useStore } from "@/store/useStore";
import RunTaskDialog from "@/components/RunTaskDialog";

export default function TitleBar() {
  const theme = useStore((s) => s.theme);
  const setTheme = useStore((s) => s.setTheme);
  const alwaysOnTop = useStore((s) => s.alwaysOnTop);
  const toggleTop = useStore((s) => s.toggleAlwaysOnTop);
  const elevated = useStore((s) => s.elevated);
  const setSearch = useStore((s) => s.setSearch);
  const [runOpen, setRunOpen] = useState(false);

  return (
    <div className="titlebar" data-tauri-drag-region>
      <div className="titlebar-drag" data-tauri-drag-region>
        <span style={{ fontSize: 13, fontWeight: 600 }}>任务管理器</span>
      </div>

      <Input
        contentBefore={<SearchRegular />}
        placeholder="搜索"
        appearance="filled-lighter"
        size="small"
        style={{ width: 220 }}
        onChange={(_, d) => setSearch(d.value)}
      />

      <Tooltip content="运行新任务" relationship="label">
        <Button
          appearance="subtle"
          size="small"
          icon={<RocketRegular />}
          onClick={() => setRunOpen(true)}
        />
      </Tooltip>

      <Tooltip content={alwaysOnTop ? "取消置顶" : "置顶"} relationship="label">
        <Button
          appearance={alwaysOnTop ? "primary" : "subtle"}
          size="small"
          icon={alwaysOnTop ? <PinRegular /> : <PinOffRegular />}
          onClick={toggleTop}
        />
      </Tooltip>

      <Tooltip content={theme === "dark" ? "浅色" : "深色"} relationship="label">
        <Button
          appearance="subtle"
          size="small"
          icon={theme === "dark" ? <WeatherSunnyRegular /> : <WeatherMoonRegular />}
          onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
        />
      </Tooltip>

      {!elevated && (
        <Tooltip content="以管理员身份运行" relationship="label">
          <Button
            appearance="subtle"
            size="small"
            icon={<ShieldKeyholeRegular />}
            onClick={() => backend.restartAsAdmin()}
          />
        </Tooltip>
      )}

      <div className="win-caption">
        <button className="caption-btn" onClick={() => backend.windowMinimize()}>
          <SubtractRegular />
        </button>
        <button className="caption-btn" onClick={() => backend.windowToggleMaximize()}>
          <MaximizeRegular />
        </button>
        <button className="caption-btn close" onClick={() => backend.windowClose()}>
          <DismissRegular />
        </button>
      </div>

      <Dialog open={runOpen} onOpenChange={(_, d) => setRunOpen(d.open)}>
        <RunTaskDialog onClose={() => setRunOpen(false)} />
      </Dialog>
    </div>
  );
}
