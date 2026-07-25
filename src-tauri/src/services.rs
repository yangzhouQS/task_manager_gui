use std::collections::HashMap;

use windows::core::PCWSTR;
use windows::Win32::System::Services::{
    CloseServiceHandle, ControlService, EnumServicesStatusExW, OpenSCManagerW, OpenServiceW,
    QueryServiceConfig2W, QueryServiceConfigW, StartServiceW, ENUM_SERVICE_STATUS_PROCESSW,
    QUERY_SERVICE_CONFIGW, SC_ENUM_PROCESS_INFO, SC_HANDLE, SC_MANAGER_CONNECT,
    SC_MANAGER_ENUMERATE_SERVICE, SERVICE_CONFIG_DESCRIPTION,
    SERVICE_CONFIG_DELAYED_AUTO_START_INFO, SERVICE_CONTROL_CONTINUE, SERVICE_CONTROL_PAUSE,
    SERVICE_CONTROL_STOP, SERVICE_DELAYED_AUTO_START_INFO, SERVICE_DESCRIPTIONW,
    SERVICE_PAUSE_CONTINUE, SERVICE_QUERY_CONFIG, SERVICE_START, SERVICE_STATE_ALL,
    SERVICE_STATUS, SERVICE_STATUS_PROCESS, SERVICE_WIN32, SERVICE_STOP,
};

use crate::models::ServiceInfo;

#[derive(Clone)]
struct ServiceConfig {
    start_type: String,
    description: Option<String>,
    exe: Option<String>,
}

/// 服务配置缓存：启动类型/描述/路径几乎不变，缓存后避免每次枚举都逐个句柄查询。
pub struct ServiceCache {
    configs: HashMap<String, ServiceConfig>,
    seq: u32,
}

impl Default for ServiceCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceCache {
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            seq: 0,
        }
    }

    /// 枚举服务。`force` 为 true 时清空配置缓存强制重查（手动刷新/切到本标签时）。
    pub fn enumerate(&mut self, force: bool) -> Vec<ServiceInfo> {
        self.seq = self.seq.wrapping_add(1);
        if force || self.seq % 30 == 0 {
            self.configs.clear();
        }
        unsafe { enumerate_with_cache(&mut self.configs) }
    }
}

unsafe fn read_pwstr(p: *const u16) -> String {
    if p.is_null() {
        return String::new();
    }
    let mut len = 0;
    while *p.add(len) != 0 {
        len += 1;
    }
    String::from_utf16_lossy(std::slice::from_raw_parts(p, len))
}

fn state_name(state: u32) -> String {
    match state {
        0x00000004 => "Running".to_string(),
        0x00000001 => "Stopped".to_string(),
        0x00000007 => "Paused".to_string(),
        0x00000005 => "Continue pending".to_string(),
        0x00000006 => "Pause pending".to_string(),
        0x00000002 => "Start pending".to_string(),
        0x00000003 => "Stop pending".to_string(),
        _ => "Unknown".to_string(),
    }
}

fn start_type_name(t: u32) -> String {
    match t {
        0x00000002 => "Automatic".to_string(),
        0x00000003 => "Manual".to_string(),
        0x00000004 => "Disabled".to_string(),
        0x00000000 => "Boot".to_string(),
        0x00000001 => "System".to_string(),
        _ => "Other".to_string(),
    }
}

unsafe fn enumerate_with_cache(cache: &mut HashMap<String, ServiceConfig>) -> Vec<ServiceInfo> {
    let scm = match OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_ENUMERATE_SERVICE) {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };

    let mut needed: u32 = 0;
    let mut returned: u32 = 0;
    let _ = EnumServicesStatusExW(
        scm,
        SC_ENUM_PROCESS_INFO,
        SERVICE_WIN32,
        SERVICE_STATE_ALL,
        None,
        &mut needed,
        &mut returned,
        None,
        PCWSTR::null(),
    );

    if needed == 0 {
        let _ = CloseServiceHandle(scm);
        return Vec::new();
    }
    let mut buf = vec![0u8; needed as usize];
    if EnumServicesStatusExW(
        scm,
        SC_ENUM_PROCESS_INFO,
        SERVICE_WIN32,
        SERVICE_STATE_ALL,
        Some(&mut buf),
        &mut needed,
        &mut returned,
        None,
        PCWSTR::null(),
    )
    .is_err()
    {
        let _ = CloseServiceHandle(scm);
        return Vec::new();
    }

    let base = buf.as_ptr() as *const ENUM_SERVICE_STATUS_PROCESSW;
    let mut out = Vec::with_capacity(returned as usize);
    for i in 0..returned as isize {
        let entry = &*base.offset(i);
        let name = read_pwstr(entry.lpServiceName.0);
        let display = read_pwstr(entry.lpDisplayName.0);
        let status: SERVICE_STATUS_PROCESS = entry.ServiceStatusProcess;

        let cfg = if let Some(c) = cache.get(&name) {
            c.clone()
        } else {
            let c = query_config(scm, &name);
            cache.insert(name.clone(), c.clone());
            c
        };

        out.push(ServiceInfo {
            name,
            display_name: display,
            pid: status.dwProcessId,
            state: state_name(status.dwCurrentState.0),
            start_type: cfg.start_type,
            description: cfg.description,
            exe_path: cfg.exe,
            kind: "Win32".to_string(),
        });
    }

    let _ = CloseServiceHandle(scm);
    out
}

unsafe fn query_config(scm: SC_HANDLE, name: &str) -> ServiceConfig {
    let wname = crate::util::to_wide(name);
    let service = match OpenServiceW(scm, PCWSTR(wname.as_ptr()), SERVICE_QUERY_CONFIG) {
        Ok(s) => s,
        Err(_) => {
            return ServiceConfig {
                start_type: "Unknown".to_string(),
                description: None,
                exe: None,
            }
        }
    };

    let mut start_type = "Unknown".to_string();
    let mut exe: Option<String> = None;

    let mut needed: u32 = 0;
    let _ = QueryServiceConfigW(service, None, 0, &mut needed);
    if needed > 0 {
        let mut buf = vec![0u8; needed as usize];
        if QueryServiceConfigW(
            service,
            Some(buf.as_mut_ptr() as *mut _),
            buf.len() as u32,
            &mut needed,
        )
        .is_ok()
        {
            let cfg = &*(buf.as_ptr() as *const QUERY_SERVICE_CONFIGW);
            let st = cfg.dwStartType.0;
            let mut label = start_type_name(st);
            if st == 0x2 && query_delayed(service) {
                label = "Automatic (Delayed)".to_string();
            }
            start_type = label;
            let path = read_pwstr(cfg.lpBinaryPathName.0);
            if !path.is_empty() {
                exe = Some(clean_service_path(&path));
            }
        }
    }

    let mut desc = None;
    let mut desc_needed: u32 = 0;
    let _ = QueryServiceConfig2W(service, SERVICE_CONFIG_DESCRIPTION, None, &mut desc_needed);
    if desc_needed > 0 {
        let mut buf = vec![0u8; desc_needed as usize];
        if QueryServiceConfig2W(
            service,
            SERVICE_CONFIG_DESCRIPTION,
            Some(&mut buf),
            &mut desc_needed,
        )
        .is_ok()
        {
            let d = &*(buf.as_ptr() as *const SERVICE_DESCRIPTIONW);
            let s = read_pwstr(d.lpDescription.0);
            if !s.is_empty() {
                desc = Some(s);
            }
        }
    }

    let _ = CloseServiceHandle(service);
    ServiceConfig {
        start_type,
        description: desc,
        exe,
    }
}

unsafe fn query_delayed(service: SC_HANDLE) -> bool {
    let mut needed: u32 = 0;
    let _ = QueryServiceConfig2W(
        service,
        SERVICE_CONFIG_DELAYED_AUTO_START_INFO,
        None,
        &mut needed,
    );
    if needed < 4 {
        return false;
    }
    let mut buf = vec![0u8; needed as usize];
    if QueryServiceConfig2W(
        service,
        SERVICE_CONFIG_DELAYED_AUTO_START_INFO,
        Some(&mut buf),
        &mut needed,
    )
    .is_ok()
    {
        let info = &*(buf.as_ptr() as *const SERVICE_DELAYED_AUTO_START_INFO);
        return info.fDelayedAutostart.as_bool();
    }
    false
}

fn clean_service_path(p: &str) -> String {
    let s = p.trim();
    if s.starts_with('"') {
        if let Some(end) = s[1..].find('"') {
            return s[1..1 + end].to_string();
        }
    }
    s.split_whitespace().next().unwrap_or(s).to_string()
}

/// 一次性枚举（无缓存），供 get_services 命令使用。
pub fn enumerate_services() -> Vec<ServiceInfo> {
    let mut cache = HashMap::new();
    unsafe { enumerate_with_cache(&mut cache) }
}

pub fn control_service(name: &str, action: &str) -> anyhow::Result<()> {
    unsafe {
        let scm = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT)?;
        let wname = crate::util::to_wide(name);
        let access = match action {
            "stop" => SERVICE_STOP,
            "start" => SERVICE_START,
            "restart" => SERVICE_STOP | SERVICE_START,
            "pause" | "continue" => SERVICE_PAUSE_CONTINUE,
            _ => SERVICE_STOP,
        };
        let service = OpenServiceW(scm, PCWSTR(wname.as_ptr()), access)?;
        match action {
            "stop" => {
                let mut status = SERVICE_STATUS::default();
                ControlService(service, SERVICE_CONTROL_STOP, &mut status)?;
            }
            "restart" => {
                let mut status = SERVICE_STATUS::default();
                let _ = ControlService(service, SERVICE_CONTROL_STOP, &mut status);
                let _ = StartServiceW(service, None);
            }
            "start" => {
                StartServiceW(service, None)?;
            }
            "pause" => {
                let mut status = SERVICE_STATUS::default();
                ControlService(service, SERVICE_CONTROL_PAUSE, &mut status)?;
            }
            "continue" => {
                let mut status = SERVICE_STATUS::default();
                ControlService(service, SERVICE_CONTROL_CONTINUE, &mut status)?;
            }
            _ => {}
        }
        let _ = CloseServiceHandle(service);
        let _ = CloseServiceHandle(scm);
        Ok(())
    }
}
