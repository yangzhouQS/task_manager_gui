use std::collections::{HashMap, HashSet};
use std::time::Instant;

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, BOOL, HANDLE, HWND, LPARAM};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX};
use windows::Win32::System::Threading::{
    GetProcessHandleCount, GetProcessIoCounters, IO_COUNTERS, OpenProcess, PROCESS_NAME_FORMAT,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ, QueryFullProcessImageNameW,
};
use windows::Win32::System::WindowsProgramming::QueryProcessCycleTime;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowLongPtrW, GetWindowTextLengthW, GetWindowThreadProcessId,
    IsWindowVisible, GWL_EXSTYLE, WS_EX_TOOLWINDOW,
};

use crate::models::{ProcessInfo, ProcessKind};
use crate::util::{from_wide, logical_processor_count, query_idle_cycles};

const CORE_PROCESSES: &[&str] = &[
    "system idle process", "system", "registry", "memory compression", "smss.exe", "csrss.exe",
    "wininit.exe", "winlogon.exe", "services.exe", "lsass.exe", "svchost.exe", "fontdrvhost.exe",
    "dwm.exe", "spoolsv.exe", "sihost.exe", "taskhostw.exe", "ctfmon.exe", "runtimebroker.exe",
    "searchhost.exe", "startmenuexperiencehost.exe", "textinputhost.exe", "dllhost.exe",
    "conhost.exe", "wudfhost.exe", "dashost.exe", "msmpeng.exe", "tiworker.exe",
    "searchindexer.exe", "wmiprvse.exe",
];

struct RawSample {
    pid: u32,
    ppid: u32,
    name: String,
    cycles: u64,
    read_bytes: u64,
    write_bytes: u64,
    working_set: u64,
    private_bytes: u64,
    threads: u32,
    handles: u32,
    exe: Option<String>,
}

pub struct ProcessCollector {
    num_cores: u32,
    last_instant: Option<Instant>,
    prev_proc_cycles: HashMap<u32, u64>,
    prev_idle: u64,
    prev_io: HashMap<u32, (u64, u64)>,
}

impl Default for ProcessCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessCollector {
    pub fn new() -> Self {
        Self {
            num_cores: logical_processor_count(),
            last_instant: None,
            prev_proc_cycles: HashMap::new(),
            prev_idle: 0,
            prev_io: HashMap::new(),
        }
    }

    pub fn collect(&mut self) -> Vec<ProcessInfo> {
        let now = Instant::now();
        let dt = self
            .last_instant
            .map(|t| now.duration_since(t).as_secs_f64())
            .unwrap_or(0.0);

        let snapshot = match self.snapshot_raw() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let services_pid = snapshot
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case("services.exe"))
            .map(|p| p.pid)
            .unwrap_or(0);

        let app_pids = enum_app_pids();

        let idle_now = query_idle_cycles(self.num_cores);
        let idle_delta = idle_now.saturating_sub(self.prev_idle);

        let mut total_busy: u64 = 0;
        for p in &snapshot {
            let prev = self.prev_proc_cycles.get(&p.pid).copied().unwrap_or(0);
            total_busy = total_busy.saturating_add(p.cycles.saturating_sub(prev));
        }
        let total = idle_delta.saturating_add(total_busy).max(1);

        let mut out = Vec::with_capacity(snapshot.len());
        for p in &snapshot {
            let prev = self.prev_proc_cycles.get(&p.pid).copied().unwrap_or(0);
            let cycle_delta = p.cycles.saturating_sub(prev);
            let cpu = (cycle_delta as f64 / total as f64) * self.num_cores as f64 * 100.0;

            let (read_prev, write_prev) = self.prev_io.get(&p.pid).copied().unwrap_or((0, 0));
            let read_delta = p.read_bytes.saturating_sub(read_prev);
            let write_delta = p.write_bytes.saturating_sub(write_prev);
            let disk_mbps = if dt > 0.0 {
                ((read_delta + write_delta) as f64 / dt / 1_048_576.0) as f32
            } else {
                0.0
            };

            let kind = classify(p, services_pid, &app_pids);
            let power = power_usage(cpu, disk_mbps);

            out.push(ProcessInfo {
                pid: p.pid,
                ppid: p.ppid,
                name: p.name.clone(),
                icon: None,
                description: None,
                exe_path: p.exe.clone(),
                command_line: None,
                cpu_percent: cpu as f32,
                memory_kb: p.working_set / 1024,
                working_set_kb: p.working_set / 1024,
                private_kb: p.private_bytes / 1024,
                disk_mbps,
                net_mbps: 0.0,
                gpu_percent: 0.0,
                power_usage: power,
                thread_count: p.threads,
                handle_count: p.handles,
                status: "Running".to_string(),
                kind,
                group_count: 0,
                user: None,
            });
        }

        let mut group_map: HashMap<(String, u8), u32> = HashMap::new();
        for p in &out {
            let k = p.kind as u8;
            *group_map.entry((p.name.to_lowercase(), k)).or_default() += 1;
        }
        for p in out.iter_mut() {
            let k = p.kind as u8;
            p.group_count = *group_map
                .get(&(p.name.to_lowercase(), k))
                .unwrap_or(&1);
        }

        let idle_cpu = (idle_delta as f64 / total as f64) * self.num_cores as f64 * 100.0;
        out.push(ProcessInfo {
            pid: 0,
            ppid: 0,
            name: "System Idle Process".to_string(),
            icon: None,
            description: None,
            exe_path: None,
            command_line: None,
            cpu_percent: idle_cpu as f32,
            memory_kb: 0,
            working_set_kb: 0,
            private_kb: 0,
            disk_mbps: 0.0,
            net_mbps: 0.0,
            gpu_percent: 0.0,
            power_usage: "Very low".to_string(),
            thread_count: 0,
            handle_count: 0,
            status: "Running".to_string(),
            kind: ProcessKind::Windows,
            group_count: 1,
            user: None,
        });

        self.prev_proc_cycles.clear();
        self.prev_io.clear();
        for p in &snapshot {
            self.prev_proc_cycles.insert(p.pid, p.cycles);
            self.prev_io.insert(p.pid, (p.read_bytes, p.write_bytes));
        }
        self.prev_idle = idle_now;
        self.last_instant = Some(now);

        out
    }

    fn snapshot_raw(&self) -> anyhow::Result<Vec<RawSample>> {
        unsafe {
            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
            let mut out = Vec::new();
            if Process32FirstW(snap, &mut entry).is_ok() {
                loop {
                    let pid = entry.th32ProcessID;
                    let (cycles, read, write, ws, priv_b, handles, exe) = if pid == 0 {
                        (0u64, 0u64, 0u64, 0u64, 0u64, 0u32, None)
                    } else {
                        query_process(pid)
                    };
                    out.push(RawSample {
                        pid,
                        ppid: entry.th32ParentProcessID,
                        name: from_wide(&entry.szExeFile),
                        cycles,
                        read_bytes: read,
                        write_bytes: write,
                        working_set: ws,
                        private_bytes: priv_b,
                        threads: entry.cntThreads,
                        handles,
                        exe,
                    });
                    if Process32NextW(snap, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snap);
            Ok(out)
        }
    }
}

unsafe fn query_process(pid: u32) -> (u64, u64, u64, u64, u64, u32, Option<String>) {
    let access = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ;
    let handle = match OpenProcess(access, false, pid) {
        Ok(h) => h,
        Err(_) => return (0, 0, 0, 0, 0, 0, None),
    };

    let mut cycles: u64 = 0;
    let _ = QueryProcessCycleTime(handle, &mut cycles);

    let mut counters = PROCESS_MEMORY_COUNTERS_EX::default();
    counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
    let _ = K32GetProcessMemoryInfo(
        handle,
        &mut counters as *mut _ as *mut _,
        std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
    );

    let mut io = IO_COUNTERS::default();
    let _ = GetProcessIoCounters(handle, &mut io);

    let mut handles: u32 = 0;
    let _ = GetProcessHandleCount(handle, &mut handles);

    let exe = read_image_path(handle);

    let _ = CloseHandle(handle);
    (
        cycles,
        io.ReadTransferCount,
        io.WriteTransferCount,
        counters.WorkingSetSize as u64,
        counters.PrivateUsage as u64,
        handles,
        exe,
    )
}

unsafe fn read_image_path(handle: HANDLE) -> Option<String> {
    let mut buf = [0u16; 1024];
    let mut len = buf.len() as u32;
    if QueryFullProcessImageNameW(handle, PROCESS_NAME_FORMAT(0), PWSTR(buf.as_mut_ptr()), &mut len)
        .is_ok()
    {
        Some(from_wide(&buf[..len as usize]))
    } else {
        None
    }
}

fn classify(p: &RawSample, services_pid: u32, app_pids: &HashSet<u32>) -> ProcessKind {
    let lower = p.name.to_lowercase();
    if app_pids.contains(&p.pid) {
        return ProcessKind::App;
    }
    if p.pid <= 4
        || CORE_PROCESSES.contains(&lower.as_str())
        || p.ppid == services_pid
        || p.exe.as_deref().map(is_under_system32).unwrap_or(false)
    {
        return ProcessKind::Windows;
    }
    ProcessKind::Background
}

fn is_under_system32(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("\\windows\\system32\\") || lower.contains("\\windows\\syswow64\\")
}

fn power_usage(cpu: f64, disk: f32) -> String {
    if cpu > 50.0 || disk > 50.0 {
        "Very high".to_string()
    } else if cpu > 15.0 || disk > 10.0 {
        "High".to_string()
    } else if cpu > 2.0 || disk > 1.0 {
        "Medium".to_string()
    } else if cpu > 0.2 {
        "Low".to_string()
    } else {
        "Very low".to_string()
    }
}

fn enum_app_pids() -> HashSet<u32> {
    unsafe {
        let mut set: HashSet<u32> = HashSet::new();
        let ptr = &mut set as *mut HashSet<u32> as isize;
        let _ = EnumWindows(Some(enum_proc), LPARAM(ptr));
        set
    }
}

unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let set = &mut *(lparam.0 as *mut HashSet<u32>);
    if !IsWindowVisible(hwnd).as_bool() || GetWindowTextLengthW(hwnd) == 0 {
        return BOOL(1);
    }
    let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
    if ex & WS_EX_TOOLWINDOW.0 != 0 {
        return BOOL(1);
    }
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid != 0 {
        set.insert(pid);
    }
    BOOL(1)
}
