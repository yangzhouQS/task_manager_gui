use std::sync::Arc;

use tauri::Manager;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION,
    PROCESS_TERMINATE, ProcessPowerThrottling, QueryFullProcessImageNameW, SetPriorityClass,
    SetProcessAffinityMask, SetProcessInformation, TerminateProcess, ABOVE_NORMAL_PRIORITY_CLASS,
    BELOW_NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS, IDLE_PRIORITY_CLASS,
    NORMAL_PRIORITY_CLASS, PROCESS_INFORMATION_CLASS, PROCESS_POWER_THROTTLING_STATE,
    REALTIME_PRIORITY_CLASS,
};
use windows::Win32::UI::Shell::{ShellExecuteExW, ShellExecuteW, SHELLEXECUTEINFOW};
use windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD;

use crate::error::{AppError, AppResult};
use crate::models::{ProcessInfo, TabId};
use crate::state::AppState;
use crate::{apphistory, services, startup, users};

fn exe_path_of(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let res =
            QueryFullProcessImageNameW(handle, PROCESS_NAME_FORMAT(0), PWSTR(buf.as_mut_ptr()), &mut len);
        let _ = windows::Win32::Foundation::CloseHandle(handle);
        if res.is_ok() {
            Some(crate::util::from_wide(&buf[..len as usize]))
        } else {
            None
        }
    }
}

fn shell(verb: Option<&str>, file: &str, params: Option<&str>) {
    unsafe {
        let f = crate::util::to_wide(file);
        let v = verb.map(|s| crate::util::to_wide(s));
        let p = params.map(|s| crate::util::to_wide(s));
        let _ = ShellExecuteW(
            None,
            v.as_ref()
                .map(|x| PCWSTR(x.as_ptr()))
                .unwrap_or(PCWSTR::null()),
            PCWSTR(f.as_ptr()),
            p.as_ref()
                .map(|x| PCWSTR(x.as_ptr()))
                .unwrap_or(PCWSTR::null()),
            PCWSTR::null(),
            SHOW_WINDOW_CMD(1),
        );
    }
}

#[tauri::command]
pub fn get_app_info(state: tauri::State<'_, Arc<AppState>>) -> AppInfo {
    AppInfo {
        elevated: state.elevated,
        cores: state.cores,
    }
}

#[derive(serde::Serialize)]
pub struct AppInfo {
    pub elevated: bool,
    pub cores: u32,
}

#[tauri::command]
pub fn set_active_tab(tab: String, state: tauri::State<'_, Arc<AppState>>) {
    *state.active_tab.write() = TabId::from_str(&tab);
    if tab == "services" {
        state
            .force_services
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

#[tauri::command]
pub fn refresh_services(state: tauri::State<'_, Arc<AppState>>) {
    state
        .force_services
        .store(true, std::sync::atomic::Ordering::Relaxed);
}

#[tauri::command]
pub fn set_refresh_speed(speed: String, state: tauri::State<'_, Arc<AppState>>) {
    let ms = match speed.as_str() {
        "high" => 500,
        "low" => 2000,
        "paused" => 0,
        _ => 1000,
    };
    state
        .refresh_ms
        .store(ms, std::sync::atomic::Ordering::Relaxed);
}

#[tauri::command]
pub fn get_processes(state: tauri::State<'_, Arc<AppState>>) -> Vec<ProcessInfo> {
    state.processes.read().clone()
}

#[tauri::command]
pub fn get_performance(
    state: tauri::State<'_, Arc<AppState>>,
) -> Option<crate::models::PerformanceSnapshot> {
    state.performance.read().clone()
}

#[tauri::command]
pub fn end_task(pid: u32) -> AppResult<()> {
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, pid)
            .map_err(|_| AppError::AccessDenied(pid.to_string()))?;
        let res = TerminateProcess(handle, 1);
        let _ = windows::Win32::Foundation::CloseHandle(handle);
        res.map_err(|e| AppError::Windows(e.to_string()))?;
        Ok(())
    }
}

#[tauri::command]
pub fn run_new_task(exe: String, args: Option<String>, admin: Option<bool>) -> AppResult<()> {
    let verb = if admin.unwrap_or(false) {
        Some("runas")
    } else {
        Some("open")
    };
    let params = args.filter(|s| !s.is_empty());
    shell(verb, &exe, params.as_deref());
    Ok(())
}

#[tauri::command]
pub fn open_file_location(pid: u32) -> AppResult<()> {
    let path = exe_path_of(pid).ok_or_else(|| AppError::Generic("无法获取进程路径".into()))?;
    shell(None, "explorer.exe", Some(&format!("/select,\"{}\"", path)));
    Ok(())
}

#[tauri::command]
pub fn open_properties(pid: u32) -> AppResult<()> {
    let path = exe_path_of(pid).ok_or_else(|| AppError::Generic("无法获取进程路径".into()))?;
    unsafe {
        let file = crate::util::to_wide(&path);
        let verb = crate::util::to_wide("properties");
        let mut info: SHELLEXECUTEINFOW = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        info.fMask = 0x00000040; // SEE_MASK_INVOKEIDLIST
        info.lpVerb = PCWSTR(verb.as_ptr());
        info.lpFile = PCWSTR(file.as_ptr());
        info.nShow = 1;
        let _ = ShellExecuteExW(&mut info);
    }
    Ok(())
}

#[tauri::command]
pub fn search_online(name: String) -> AppResult<()> {
    let q = urlencoding_like(&name);
    shell(None, &format!("https://www.bing.com/search?q={q}"), None);
    Ok(())
}

fn urlencoding_like(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b' ' => out.push('+'),
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[tauri::command]
pub fn toggle_always_on_top(app: tauri::AppHandle, on: bool) -> AppResult<()> {
    if let Some(win) = app.get_webview_window("main") {
        win.set_always_on_top(on)
            .map_err(|e| AppError::Generic(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn window_minimize(app: tauri::AppHandle) -> AppResult<()> {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.minimize();
    }
    Ok(())
}

#[tauri::command]
pub fn window_toggle_maximize(app: tauri::AppHandle) -> AppResult<()> {
    if let Some(win) = app.get_webview_window("main") {
        if let Ok(maximized) = win.is_maximized() {
            if maximized {
                let _ = win.unmaximize();
            } else {
                let _ = win.maximize();
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn window_close(app: tauri::AppHandle) -> AppResult<()> {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.close();
    }
    Ok(())
}

#[tauri::command]
pub fn set_priority(pid: u32, level: String) -> AppResult<()> {
    let class = match level.as_str() {
        "realtime" => REALTIME_PRIORITY_CLASS,
        "high" => HIGH_PRIORITY_CLASS,
        "above_normal" => ABOVE_NORMAL_PRIORITY_CLASS,
        "normal" => NORMAL_PRIORITY_CLASS,
        "below_normal" => BELOW_NORMAL_PRIORITY_CLASS,
        "low" => IDLE_PRIORITY_CLASS,
        _ => NORMAL_PRIORITY_CLASS,
    };
    unsafe {
        let handle = OpenProcess(PROCESS_SET_INFORMATION, false, pid)
            .map_err(|_| AppError::AccessDenied(pid.to_string()))?;
        let res = SetPriorityClass(handle, class);
        let _ = windows::Win32::Foundation::CloseHandle(handle);
        res.map_err(|e| AppError::Windows(e.to_string()))?;
        Ok(())
    }
}

#[tauri::command]
pub fn set_affinity(pid: u32, mask: usize) -> AppResult<()> {
    unsafe {
        let handle = OpenProcess(PROCESS_SET_INFORMATION, false, pid)
            .map_err(|_| AppError::AccessDenied(pid.to_string()))?;
        let res = SetProcessAffinityMask(handle, mask);
        let _ = windows::Win32::Foundation::CloseHandle(handle);
        res.map_err(|e| AppError::Windows(e.to_string()))?;
        Ok(())
    }
}

#[tauri::command]
pub fn efficiency_mode(pid: u32, on: bool) -> AppResult<()> {
    const EXEC_SPEED: u32 = 0x1;
    unsafe {
        let handle = OpenProcess(PROCESS_SET_INFORMATION, false, pid)
            .map_err(|_| AppError::AccessDenied(pid.to_string()))?;
        let mut state = PROCESS_POWER_THROTTLING_STATE {
            Version: 1,
            ControlMask: EXEC_SPEED,
            StateMask: if on { EXEC_SPEED } else { 0 },
        };
        let res = SetProcessInformation(
            handle,
            PROCESS_INFORMATION_CLASS(ProcessPowerThrottling.0),
            &mut state as *mut _ as *mut _,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
        let _ = windows::Win32::Foundation::CloseHandle(handle);
        res.map_err(|e| AppError::Windows(e.to_string()))?;
        Ok(())
    }
}

#[tauri::command]
pub fn get_services() -> Vec<crate::models::ServiceInfo> {
    services::enumerate_services()
}

#[tauri::command]
pub fn control_service(name: String, action: String) -> AppResult<()> {
    services::control_service(&name, &action).map_err(|e| AppError::Generic(e.to_string()))
}

#[tauri::command]
pub fn get_startup() -> Vec<crate::models::StartupEntry> {
    startup::enumerate_startup()
}

#[tauri::command]
pub fn set_startup_enabled(name: String, source: String, enabled: bool) -> AppResult<()> {
    startup::set_startup_enabled(&name, &source, enabled)
        .map_err(|e| AppError::Generic(e.to_string()))
}

#[tauri::command]
pub fn get_users(state: tauri::State<'_, Arc<AppState>>) -> Vec<crate::models::UserSession> {
    let procs = state.processes.read().clone();
    users::collect_user_sessions(procs)
}

#[tauri::command]
pub fn get_app_history(
    state: tauri::State<'_, Arc<AppState>>,
) -> Vec<crate::models::AppHistoryEntry> {
    state.history.lock().snapshot()
}

#[tauri::command]
pub fn reset_app_history(state: tauri::State<'_, Arc<AppState>>) -> AppResult<()> {
    state.history.lock().reset();
    Ok(())
}

#[tauri::command]
pub fn restart_as_admin(app: tauri::AppHandle) -> AppResult<()> {
    let exe = std::env::current_exe()
        .map_err(|e| AppError::Generic(e.to_string()))?
        .to_string_lossy()
        .to_string();
    unsafe {
        let file = crate::util::to_wide(&exe);
        let verb = crate::util::to_wide("runas");
        let mut info: SHELLEXECUTEINFOW = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        info.lpVerb = PCWSTR(verb.as_ptr());
        info.lpFile = PCWSTR(file.as_ptr());
        info.nShow = 1;
        let _ = ShellExecuteExW(&mut info);
    }
    app.exit(0);
    Ok(())
}

#[allow(dead_code)]
fn _keep() {
    let _ = apphistory::AppHistory::new(1);
}
