use std::collections::HashMap;

use windows::core::PCWSTR;
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};
use windows::Win32::System::ProcessStatus::{GetPerformanceInfo, PERFORMANCE_INFORMATION};
use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD};
use windows::Win32::System::SystemInformation::{GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX};

use crate::models::{DiskUsage, GpuUsage, NetUsage, PerformanceSnapshot};
use crate::pdh::{self, Pdh};

type Counter = isize;

struct CpuPdh {
    pdh: Pdh,
    per_core: Vec<(String, Counter)>,
    total: Counter,
    speed_ghz: f32,
}

struct DiskPdh {
    pdh: Pdh,
    entries: Vec<DiskEntry>,
}

struct DiskEntry {
    name: String,
    pct: Counter,
    read: Counter,
    write: Counter,
}

struct NetPdh {
    pdh: Pdh,
    entries: Vec<NetEntry>,
}

struct NetEntry {
    name: String,
    send: Counter,
    recv: Counter,
}

struct GpuPdh {
    pdh: Pdh,
    engines_by_luid: HashMap<u64, Vec<Counter>>,
    adapters: Vec<(u64, String, u64, u64)>,
}

pub struct PerformanceCollector {
    cpu: CpuPdh,
    disk: DiskPdh,
    net: NetPdh,
    gpu: GpuPdh,
    cores: u32,
}

impl PerformanceCollector {
    pub fn new() -> anyhow::Result<Self> {
        let cores = crate::util::logical_processor_count();
        Ok(Self {
            cpu: build_cpu(cores)?,
            disk: build_disk()?,
            net: build_net()?,
            gpu: build_gpu()?,
            cores,
        })
    }

    pub fn snapshot(&self) -> PerformanceSnapshot {
        let mem = read_memory();
        let uptime = unsafe { GetTickCount64() } / 1000;

        self.cpu.pdh.collect();
        let cpu_per_core = self
            .cpu
            .per_core
            .iter()
            .map(|(_, c)| self.cpu.pdh.value(*c).unwrap_or(0.0) as f32)
            .collect();
        let cpu_total = self.cpu.pdh.value(self.cpu.total).unwrap_or(0.0) as f32;

        self.disk.pdh.collect();
        let mut disk_usage = Vec::new();
        for e in &self.disk.entries {
            let pct = self.disk.pdh.value(e.pct).unwrap_or(0.0);
            let read = self.disk.pdh.value(e.read).unwrap_or(0.0);
            let write = self.disk.pdh.value(e.write).unwrap_or(0.0);
            disk_usage.push(DiskUsage {
                name: e.name.clone(),
                percent: pct as f32,
                read_mbps: (read / 1_048_576.0) as f32,
                write_mbps: (write / 1_048_576.0) as f32,
                avg_response_ms: 0.0,
            });
        }

        self.net.pdh.collect();
        let mut net_usage = Vec::new();
        for e in &self.net.entries {
            let send = self.net.pdh.value(e.send).unwrap_or(0.0);
            let recv = self.net.pdh.value(e.recv).unwrap_or(0.0);
            net_usage.push(NetUsage {
                name: e.name.clone(),
                send_mbps: (send * 8.0 / 1_000_000.0) as f32,
                recv_mbps: (recv * 8.0 / 1_000_000.0) as f32,
            });
        }

        self.gpu.pdh.collect();
        let mut gpu_usage = Vec::new();
        for (luid, name, dedicated, shared) in &self.gpu.adapters {
            let mut sum = 0.0_f64;
            if let Some(counters) = self.gpu.engines_by_luid.get(luid) {
                for c in counters {
                    sum += self.gpu.pdh.value(*c).unwrap_or(0.0);
                }
            }
            gpu_usage.push(GpuUsage {
                name: name.clone(),
                percent: sum.min(100.0) as f32,
                dedicated_kb: *dedicated,
                shared_kb: *shared,
            });
        }

        PerformanceSnapshot {
            cpu_percent: cpu_total,
            cpu_per_core,
            cpu_core_count: self.cores,
            cpu_speed_ghz: self.cpu.speed_ghz,
            cpu_up_time_sec: uptime,
            mem_total_kb: mem.total_kb,
            mem_in_use_kb: mem.total_kb.saturating_sub(mem.avail_kb),
            mem_available_kb: mem.avail_kb,
            mem_committed_kb: mem.committed_kb,
            mem_committed_limit_kb: mem.commit_limit_kb,
            mem_cached_kb: mem.cached_kb,
            mem_paged_pool_kb: mem.paged_kb,
            mem_non_paged_pool_kb: mem.nonpaged_kb,
            disk_usage,
            net_usage,
            gpu_usage,
        }
    }
}

struct MemInfo {
    total_kb: u64,
    avail_kb: u64,
    committed_kb: u64,
    commit_limit_kb: u64,
    cached_kb: u64,
    paged_kb: u64,
    nonpaged_kb: u64,
}

fn read_memory() -> MemInfo {
    unsafe {
        let mut status = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            ..std::mem::zeroed()
        };
        let _ = GlobalMemoryStatusEx(&mut status);
        let total_kb = status.ullTotalPhys / 1024;
        let avail_kb = status.ullAvailPhys / 1024;

        let mut perf = PERFORMANCE_INFORMATION {
            cb: std::mem::size_of::<PERFORMANCE_INFORMATION>() as u32,
            ..std::mem::zeroed()
        };
        let _ = GetPerformanceInfo(&mut perf, std::mem::size_of::<PERFORMANCE_INFORMATION>() as u32);
        let ps = perf.PageSize / 1024;
        MemInfo {
            total_kb,
            avail_kb,
            committed_kb: (perf.CommitTotal * ps) as u64,
            commit_limit_kb: (perf.CommitLimit * ps) as u64,
            cached_kb: (perf.SystemCache * ps) as u64,
            paged_kb: (perf.KernelPaged * ps) as u64,
            nonpaged_kb: (perf.KernelNonpaged * ps) as u64,
        }
    }
}

fn read_cpu_mhz() -> u32 {
    unsafe {
        let mut val: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;
        let path = crate::util::to_wide("~MHz");
        let sub = crate::util::to_wide("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0");
        let _ = RegGetValueW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(sub.as_ptr()),
            PCWSTR(path.as_ptr()),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut val as *mut u32 as *mut core::ffi::c_void),
            Some(&mut size),
        );
        val
    }
}

fn build_cpu(cores: u32) -> anyhow::Result<CpuPdh> {
    let pdh = Pdh::new()?;
    let mut per_core: Vec<(String, Counter)> = Vec::new();
    let mut total: Counter = 0;

    for p in pdh::expand("\\Processor(*)\\% Processor Time") {
        let inst = pdh::parse_instance(&p).unwrap_or_default();
        let counter = pdh.add(&p)?;
        if inst == "_Total" {
            total = counter;
        } else {
            per_core.push((inst, counter));
        }
    }
    if per_core.is_empty() {
        for i in 0..cores {
            if let Ok(c) = pdh.add(&format!("\\Processor({i})\\% Processor Time")) {
                per_core.push((i.to_string(), c));
            }
        }
    }
    Ok(CpuPdh {
        pdh,
        per_core,
        total,
        speed_ghz: read_cpu_mhz() as f32 / 1000.0,
    })
}

fn build_disk() -> anyhow::Result<DiskPdh> {
    let pdh = Pdh::new()?;
    let mut entries = Vec::new();
    for inst in pdh::expand("\\PhysicalDisk(*)\\% Disk Time") {
        let name = pdh::parse_instance(&inst).unwrap_or_default();
        if name == "_Total" || name.is_empty() {
            continue;
        }
        let pct = pdh.add(&format!("\\PhysicalDisk({name})\\% Disk Time"));
        let read = pdh.add(&format!("\\PhysicalDisk({name})\\Disk Read Bytes/sec"));
        let write = pdh.add(&format!("\\PhysicalDisk({name})\\Disk Write Bytes/sec"));
        if let (Ok(pct), Ok(read), Ok(write)) = (pct, read, write) {
            entries.push(DiskEntry {
                name: name.trim().trim_start_matches('0').to_string(),
                pct,
                read,
                write,
            });
        }
    }
    Ok(DiskPdh { pdh, entries })
}

fn build_net() -> anyhow::Result<NetPdh> {
    let pdh = Pdh::new()?;
    let mut entries = Vec::new();
    for inst in pdh::expand("\\Network Interface(*)\\Bytes Sent/sec") {
        let name = pdh::parse_instance(&inst).unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let send = pdh.add(&format!("\\Network Interface({name})\\Bytes Sent/sec"));
        let recv = pdh.add(&format!("\\Network Interface({name})\\Bytes Received/sec"));
        if let (Ok(send), Ok(recv)) = (send, recv) {
            entries.push(NetEntry { name, send, recv });
        }
    }
    Ok(NetPdh { pdh, entries })
}

fn build_gpu() -> anyhow::Result<GpuPdh> {
    let pdh = Pdh::new()?;
    let adapters = enum_dxgi_adapters();
    let mut engines_by_luid: HashMap<u64, Vec<Counter>> = HashMap::new();
    for p in pdh::expand("\\GPU Engine(*)\\Utilization Percentage") {
        if let Some(inst) = pdh::parse_instance(&p) {
            if let Some(luid) = parse_engine_luid(&inst) {
                if let Ok(c) = pdh.add(&p) {
                    engines_by_luid.entry(luid).or_default().push(c);
                }
            }
        }
    }
    Ok(GpuPdh {
        pdh,
        engines_by_luid,
        adapters,
    })
}

fn parse_engine_luid(inst: &str) -> Option<u64> {
    let parts: Vec<&str> = inst.split('_').collect();
    let mut iter = parts.iter();
    let mut low: Option<u64> = None;
    let mut high: Option<u64> = None;
    while let Some(p) = iter.next() {
        if *p == "luid" {
            if let (Some(a), Some(b)) = (iter.next(), iter.next()) {
                low = u64::from_str_radix(a.trim_start_matches("0x"), 16).ok();
                high = u64::from_str_radix(b.trim_start_matches("0x"), 16).ok();
            }
        }
    }
    match (low, high) {
        (Some(l), Some(h)) => Some((h << 32) | l),
        _ => None,
    }
}

fn enum_dxgi_adapters() -> Vec<(u64, String, u64, u64)> {
    unsafe {
        let mut out = Vec::new();
        let mut seen_luid: std::collections::HashSet<u64> = std::collections::HashSet::new();
        let mut seen_name: std::collections::HashSet<String> = std::collections::HashSet::new();
        let factory: IDXGIFactory1 = match CreateDXGIFactory1() {
            Ok(f) => f,
            Err(_) => return out,
        };
        let mut idx = 0;
        while let Ok(adapter) = factory.EnumAdapters1(idx) {
            if let Ok(desc) = adapter.GetDesc1() {
                let name = crate::util::from_wide(&desc.Description);
                let luid =
                    ((desc.AdapterLuid.HighPart as u64) << 32) | (desc.AdapterLuid.LowPart as u64);
                if seen_luid.insert(luid) && seen_name.insert(name.clone()) {
                    out.push((
                        luid,
                        name,
                        desc.DedicatedVideoMemory as u64 / 1024,
                        desc.SharedSystemMemory as u64 / 1024,
                    ));
                }
            }
            idx += 1;
        }
        out
    }
}
