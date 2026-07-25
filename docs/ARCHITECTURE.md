# Windows 任务管理器（Rust 实现）— 架构与技术方案

## 1. 目标

用 Rust 实现 Windows 11 任务管理器，操作体验与原生 Win11 TM 一致（Fluent 视觉、Mica 背景、7 个标签页、右键菜单、结束任务/运行新任务/置顶/效率模式等）。

## 2. 已确认决策

| 维度 | 选型 | 说明 |
|------|------|------|
| GUI 框架 | **Tauri 2 + Web** | Rust 后端采集系统数据，Web 做 UI |
| 前端 | **React 18 + TS + FluentUI v9** | `@fluentui/react-components`，Fluent 还原度最高 |
| 数据采集 | **windows crate 直调 Win32/PDH/NtQuery** | CPU 周期法、私有工作集、GPU Engine 计数器，数值贴近原生 |
| 功能范围 | **全部 7 个标签页** | 进程/性能/应用历史/启动应用/用户/详细信息/服务 |
| 权限 | **可选提升** | 默认普通权限，提供「以管理员身份运行」入口 |
| 窗口 | **Mica + 集成顶栏** | `window-vibrancy` 的 Mica + Tauri 无边框自定义顶栏 + 置顶 |

## 3. 技术栈与依赖

### 后端（src-tauri）
- `tauri = "2"` / `tauri-build = "2"`
- `tauri-plugin-shell = "2"`（打开资源管理器/浏览器/运行新任务）
- `windows = "0.x"` —— 关键 feature：`Win32_Foundation`、`Win32_System_Threading`、`Win32_System_Diagnostics_ProcessStatus`、`Win32_System_Performance`（PDH）、`Win32_System_SystemInformation`、`Win32_System_Services`、`Win32_System_Registry`、`Win32_System_Power`、`Win32_Graphics_Dxgi`、`Win32_UI_Shell`、`Win32_Security`、`Win32_Storage_FileSystem`
- `window-vibrancy` —— `apply_mica`
- `serde` / `serde_json`
- `parking_lot` —— 同步锁
- `rusqlite`（bundled）—— 应用历史持久化
- `anyhow` / `thiserror`

### 前端（src）
- React 18 + TypeScript + Vite
- `@fluentui/react-components` + `@fluentui/react-icons`
- `@tanstack/react-virtual` —— 大列表虚拟化
- `zustand` —— 状态管理
- Tauri JS API（`@tauri-apps/api`、`@tauri-apps/plugin-shell`）

## 4. 项目结构

```
task_manager_gui/
├── package.json / vite.config.ts / tsconfig.json / index.html
├── src/                         # React 前端
│   ├── main.tsx / App.tsx
│   ├── components/              # 标题栏、导航轨、命令栏、图表、表格
│   ├── views/                   # 7 个标签页视图
│   ├── store/  hooks/  types/  styles/
└── src-tauri/                   # Rust 后端
    ├── Cargo.toml / tauri.conf.json / build.rs
    ├── capabilities/default.json
    └── src/
        ├── main.rs / lib.rs
        ├── error.rs / models.rs / state.rs
        ├── collector/           # 采样调度 + 事件推送
        ├── processes/           # 进程枚举与度量（CPU/内存/磁盘/网络）
        ├── performance/         # 全局 CPU/内存/磁盘/网络/GPU
        ├── gpu/                 # PDH GPU Engine 计数器
        ├── services/            # 服务枚举与启停
        ├── startup/             # 启动项（注册表/启动文件夹）
        ├── users/               # 进程按用户/会话分组
        ├── apphistory/          # 累积统计 + rusqlite 持久化
        └── commands.rs          # Tauri invoke 命令
```

## 5. 数据流（推送模型）

- 后端 `Collector` 在独立线程中按固定间隔（默认 1000ms）采样，发射 Tauri 事件：
  `tm://processes`、`tm://performance`、`tm://services`、`tm://users`、`tm://apphistory` 等。
- 仅对**当前激活标签**所需数据做重采集，降低 CPU。
- 一次性动作用 `invoke('end_task', { pid })` 等。
- 前端用 `listen()` 订阅，写入 zustand store，视图渲染。

## 6. 关键算法（贴近 Win11 原生数值）

- **CPU（进程）**：周期法。每次采样记录 `QueryProcessCycleTime`；系统总周期 = `QueryIdleProcessorCycleTime` 空闲周期 + 所有进程忙碌周期之和；CPU% = 进程Δ周期 / 系统总Δ周期 × 100（满载所有核 = 核心×100%，与 TM 一致）。
- **CPU（整体/每核）**：PDH `\Processor(*)\% Processor Time`（每核一条线），`\Processor Information(_Total)` 为总体。
- **内存（进程）**：`K32GetProcessMemoryInfo` 取工作集；详情页同时给私有工作集与私有字节。
- **内存（整体）**：`GlobalMemoryStatusEx`（总量/可用/已提交/缓存/池）。
- **磁盘（整体）**：PDH `\PhysicalDisk(*)\% Disk Time`、`\Disk Read/Write Bytes/sec`。
- **网络（整体）**：PDH `\Network Interface(*)\Bytes Sent/Received/sec`。
- **GPU**：PDH `\GPU Engine(*)\Utilization Percentage`（按 PID 关联到进程）+ `\GPU Adapter Memory(*)`（专用/共享）。
- **进程磁盘/网络**：磁盘用 `GetProcessIoCounters` 的 IO 字节增量近似；网络难精确，按 ETW 可选增强（先标记近似）。

## 7. 标签页实现要点

1. **进程**：按 应用进程/后台进程/Windows 进程 三组，可展开折叠；列：名称/状态/CPU/内存/磁盘/网络/GPU/电源用量；列头排序；搜索过滤；右键菜单（结束任务、运行新任务、打开文件位置、属性、联机搜索、效率模式）。
2. **性能**：左侧子导航 CPU/内存/磁盘/网络/GPU；大图实时折线 + 摘要数字（与 TM 布局一致）。
3. **应用历史**：rusqlite 累积 CPU 时间/网络自上次重置；可重置。
4. **启动应用**：HKCU/HKLM Run + StartupApproved + 启动文件夹 + 计划任务；启用/禁用（启用=删 StartupApproved 禁用项/或注册表项）。
5. **用户**：进程按会话/账户分组（TokenUser→SID→账户名）。
6. **详细信息**：经典列表（PID/账户/CPU/内存/描述/状态/会话）；右键结束进程树/设置优先级（SetPriorityClass）/设置亲和性（SetProcessAffinityMask）。
7. **服务**：`EnumServicesStatusEx` + `QueryServiceConfig`（启动类型）；启动/停止/重启/打开服务。

## 8. 原生窗口与主题

- `tauri.conf.json` 设 `"decorations": false`；Rust 侧 `apply_mica(&window)` 应用 Win11 Mica。
- 自定义顶栏：左侧 logo，中部搜索框，右侧「运行新任务」「置顶」「主题」+ 最小化/最大化/关闭（data-tauri-drag-region 实现拖拽）。
- 主题：`webDarkTheme`/`webLightTheme`，随系统 `prefers-color-scheme` 切换。

## 9. 权限提升

- manifest `asInvoker`（默认不弹 UAC）。
- 提供「以管理员身份运行」入口：用 `ShellExecuteEx` 的 `runas` 动词重启自身。

## 10. 已知限制/风险

- 进程级网络数值为近似（TM 用 ETW）；精确实现后续增强。
- 启动应用「影响程度」依赖启动 ETW 测量，先显示「未测量」。
- 与原生数值 1:1 对齐需采样调参（周期法实现后需对照校验）。

## 11. 里程碑（实施顺序）

1. 脚手架（前端 + 后端）可启动的空白 Mica 窗口。
2. 进程采集 + 进程页（分组/排序/结束任务）端到端打通。
3. 性能页图表（CPU/内存/磁盘/网络/GPU）。
4. 服务页 / 详细信息页 / 用户页。
5. 启动应用页 / 应用历史页。
6. 主题、置顶、运行新任务、右键菜单全量、管理员提升。
7. 打包（MSI）与回归校验。
