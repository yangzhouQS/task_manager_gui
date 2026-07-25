use std::path::PathBuf;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::System::Registry::{
    RegCloseKey, RegEnumValueW, RegOpenKeyExW, RegQueryInfoKeyW, RegQueryValueExW, HKEY,
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
    REG_BINARY, REG_EXPAND_SZ, REG_VALUE_TYPE, REG_SZ,
};

use crate::models::StartupEntry;

unsafe fn enum_run_values(root: HKEY, sub: &str, wow64: bool) -> Vec<(String, String)> {
    let wsub = crate::util::to_wide(sub);
    let access = if wow64 {
        KEY_READ | KEY_WOW64_32KEY
    } else {
        KEY_READ | KEY_WOW64_64KEY
    };
    let mut handle = HKEY::default();
    if RegOpenKeyExW(root, PCWSTR(wsub.as_ptr()), 0, access, &mut handle).is_err() {
        return Vec::new();
    }

    let mut values: u32 = 0;
    let mut max_name: u32 = 0;
    let mut max_data: u32 = 0;
    let _ = RegQueryInfoKeyW(
        handle,
        PWSTR::null(),
        None,
        None,
        None,
        None,
        None,
        Some(&mut values),
        Some(&mut max_name),
        Some(&mut max_data),
        None,
        None,
    );
    if values == 0 {
        let _ = RegCloseKey(handle);
        return Vec::new();
    }

    let mut out = Vec::new();
    for i in 0..values {
        let mut name_buf = vec![0u16; (max_name as usize) + 2];
        let mut name_len = name_buf.len() as u32;
        let mut data_buf = vec![0u8; (max_data as usize) + 4];
        let mut data_len = data_buf.len() as u32;
        let mut typ: u32 = 0;
        if RegEnumValueW(
            handle,
            i,
            PWSTR(name_buf.as_mut_ptr()),
            &mut name_len,
            None,
            Some(&mut typ),
            Some(data_buf.as_mut_ptr()),
            Some(&mut data_len),
        )
        .is_ok()
        {
            let name = crate::util::from_wide(&name_buf[..name_len as usize]);
            if typ == REG_SZ.0 || typ == REG_EXPAND_SZ.0 {
                let count = (data_len as usize) / 2;
                let words = std::slice::from_raw_parts(data_buf.as_ptr() as *const u16, count);
                let value = crate::util::from_wide(words);
                out.push((name, value));
            }
        }
    }

    let _ = RegCloseKey(handle);
    out
}

fn startup_approved_disabled(name: &str, root: HKEY, sub: &str) -> Option<bool> {
    unsafe {
        let wsub = crate::util::to_wide(sub);
        let mut handle = HKEY::default();
        if RegOpenKeyExW(root, PCWSTR(wsub.as_ptr()), 0, KEY_READ | KEY_WOW64_64KEY, &mut handle)
            .is_err()
        {
            return None;
        }
        let wname = crate::util::to_wide(name);
        let mut typ = REG_VALUE_TYPE::default();
        let mut buf = [0u8; 16];
        let mut len = buf.len() as u32;
        let ok = RegQueryValueExW(
            handle,
            PCWSTR(wname.as_ptr()),
            None,
            Some(&mut typ),
            Some(buf.as_mut_ptr()),
            Some(&mut len),
        );
        let _ = RegCloseKey(handle);
        if ok.0 != 0 || typ != REG_BINARY || len < 4 {
            return None;
        }
        let first = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        Some(first == 0)
    }
}

fn startup_folders() -> Vec<(String, String, String, bool)> {
    let mut out = Vec::new();
    let dirs: [(Option<&str>, &str); 2] = [
        (None, "Microsoft\\Windows\\Start Menu\\Programs\\Startup"),
        (
            Some("ALLUSERSPROFILE"),
            "Microsoft\\Windows\\Start Menu\\Programs\\Startup",
        ),
    ];
    for (env, sub) in dirs {
        let base = if let Some(e) = env {
            std::env::var(e).unwrap_or_default()
        } else {
            std::env::var("APPDATA").unwrap_or_default()
        };
        if base.is_empty() {
            continue;
        }
        let dir = PathBuf::from(base).join(sub);
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    out.push((
                        name.trim_end_matches(".lnk").to_string(),
                        entry.path().to_string_lossy().to_string(),
                        format!(
                            "Startup ({})",
                            if env.is_some() { "所有用户" } else { "当前用户" }
                        ),
                        true,
                    ));
                }
            }
        }
    }
    out
}

pub fn enumerate_startup() -> Vec<StartupEntry> {
    let mut out = Vec::new();

    let sources: &[(HKEY, &str, &str)] = &[
        (
            HKEY_CURRENT_USER,
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "HKCU\\Run",
        ),
        (
            HKEY_LOCAL_MACHINE,
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "HKLM\\Run",
        ),
        (
            HKEY_LOCAL_MACHINE,
            "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run",
            "HKLM\\WOW6432Node\\Run",
        ),
    ];

    let approved_sub = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";

    for (root, sub, source) in sources {
        let pairs = unsafe { enum_run_values(*root, sub, false) };
        for (name, command) in pairs {
            let disabled = startup_approved_disabled(&name, *root, approved_sub);
            let icon = crate::icon::exe_from_command(&command)
                .as_deref()
                .and_then(crate::icon::extract);
            out.push(StartupEntry {
                name: name.clone(),
                command,
                source: source.to_string(),
                enabled: disabled.unwrap_or(true),
                impact: "Not measured".to_string(),
                publisher: None,
                icon,
            });
        }
    }

    for (name, command, source, enabled) in startup_folders() {
        let icon = crate::icon::extract(&command);
        out.push(StartupEntry {
            name,
            command,
            source,
            enabled,
            impact: "Not measured".to_string(),
            publisher: None,
            icon,
        });
    }

    out
}

#[allow(dead_code)]
pub fn set_startup_enabled(_name: &str, _source: &str, _enabled: bool) -> anyhow::Result<()> {
    Ok(())
}
