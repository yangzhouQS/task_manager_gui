use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub exe_path: Option<String>,
    pub command_line: Option<String>,
    pub cpu_percent: f32,
    pub memory_kb: u64,
    pub working_set_kb: u64,
    pub private_kb: u64,
    pub disk_mbps: f32,
    pub net_mbps: f32,
    pub gpu_percent: f32,
    pub power_usage: String,
    pub thread_count: u32,
    pub handle_count: u32,
    pub status: String,
    pub kind: ProcessKind,
    pub group_count: u32,
    pub user: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProcessKind {
    App,
    Background,
    Windows,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceSnapshot {
    pub cpu_percent: f32,
    pub cpu_per_core: Vec<f32>,
    pub cpu_core_count: u32,
    pub cpu_speed_ghz: f32,
    pub cpu_up_time_sec: u64,
    pub mem_total_kb: u64,
    pub mem_in_use_kb: u64,
    pub mem_available_kb: u64,
    pub mem_committed_kb: u64,
    pub mem_committed_limit_kb: u64,
    pub mem_cached_kb: u64,
    pub mem_paged_pool_kb: u64,
    pub mem_non_paged_pool_kb: u64,
    pub disk_usage: Vec<DiskUsage>,
    pub net_usage: Vec<NetUsage>,
    pub gpu_usage: Vec<GpuUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskUsage {
    pub name: String,
    pub percent: f32,
    pub read_mbps: f32,
    pub write_mbps: f32,
    pub avg_response_ms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetUsage {
    pub name: String,
    pub send_mbps: f32,
    pub recv_mbps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuUsage {
    pub name: String,
    pub percent: f32,
    pub dedicated_kb: u64,
    pub shared_kb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub pid: u32,
    pub state: String,
    pub start_type: String,
    pub description: Option<String>,
    pub exe_path: Option<String>,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupEntry {
    pub name: String,
    pub command: String,
    pub source: String,
    pub enabled: bool,
    pub impact: String,
    pub publisher: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSession {
    pub user: String,
    pub session_id: u32,
    pub is_current: bool,
    pub processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppHistoryEntry {
    pub name: String,
    pub icon: Option<String>,
    pub total_cpu_sec: f32,
    pub network_mb: f64,
    pub metered_mb: f64,
    pub tile_updates: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabId {
    Processes,
    Performance,
    AppHistory,
    Startup,
    Users,
    Details,
    Services,
}

impl TabId {
    pub fn from_str(s: &str) -> Self {
        match s {
            "performance" => TabId::Performance,
            "appHistory" => TabId::AppHistory,
            "startup" => TabId::Startup,
            "users" => TabId::Users,
            "details" => TabId::Details,
            "services" => TabId::Services,
            _ => TabId::Processes,
        }
    }
}
