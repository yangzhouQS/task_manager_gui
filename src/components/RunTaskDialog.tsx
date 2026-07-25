import { useState } from "react";
import {
  DialogSurface,
  DialogBody,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Input,
  Checkbox,
  Label,
} from "@fluentui/react-components";

import { backend } from "@/lib/backend";

const fieldStyle: React.CSSProperties = {
  display: "flex",
  flexDirection: "column",
  gap: 4,
  marginBottom: 12,
};

export default function RunTaskDialog({ onClose }: { onClose: () => void }) {
  const [exe, setExe] = useState("");
  const [args, setArgs] = useState("");
  const [admin, setAdmin] = useState(false);

  const submit = async () => {
    if (!exe.trim()) return;
    await backend.runNewTask(exe.trim(), args.trim() || null, admin || null);
    onClose();
  };

  return (
    <DialogSurface>
      <DialogBody>
        <DialogTitle>新建任务</DialogTitle>
        <DialogContent>
          <div style={fieldStyle}>
            <Label>打开</Label>
            <Input
              value={exe}
              onChange={(_, d) => setExe(d.value)}
              placeholder="例如 notepad.exe 或完整路径"
              autoFocus
            />
          </div>
          <div style={fieldStyle}>
            <Label>参数（可选）</Label>
            <Input value={args} onChange={(_, d) => setArgs(d.value)} />
          </div>
          <Checkbox
            checked={admin}
            onChange={(_, d) => setAdmin(d.checked as boolean)}
            label="以系统管理权限创建此任务"
          />
        </DialogContent>
        <DialogActions>
          <Button appearance="primary" onClick={submit} disabled={!exe.trim()}>
            确定
          </Button>
          <Button appearance="secondary" onClick={onClose}>
            取消
          </Button>
        </DialogActions>
      </DialogBody>
    </DialogSurface>
  );
}
