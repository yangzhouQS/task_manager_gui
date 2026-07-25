use std::collections::HashMap;

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{
    GetTokenInformation, LookupAccountSidW, SID_NAME_USE, TokenUser, TOKEN_QUERY, TOKEN_USER,
};
use windows::Win32::System::RemoteDesktop::ProcessIdToSessionId;
use windows::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};

use crate::models::{ProcessInfo, UserSession};

unsafe fn read_owner(handle: HANDLE) -> Option<String> {
    let mut token = HANDLE::default();
    if OpenProcessToken(handle, TOKEN_QUERY, &mut token).is_err() {
        return None;
    }
    let mut ret: Option<String> = None;
    let mut len: u32 = 0;
    let _ = GetTokenInformation(token, TokenUser, None, 0, &mut len);
    if len > 0 {
        let mut buf = vec![0u8; len as usize];
        if GetTokenInformation(
            token,
            TokenUser,
            Some(buf.as_mut_ptr() as *mut _ as *mut core::ffi::c_void),
            len,
            &mut len,
        )
        .is_ok()
        {
            let user = &*(buf.as_ptr() as *const TOKEN_USER);
            let sid = user.User.Sid;
            if !sid.0.is_null() {
                let mut name = [0u16; 256];
                let mut name_len = name.len() as u32;
                let mut domain = [0u16; 256];
                let mut domain_len = domain.len() as u32;
                let mut usage = SID_NAME_USE::default();
                if LookupAccountSidW(
                    None,
                    sid,
                    PWSTR(name.as_mut_ptr()),
                    &mut name_len,
                    PWSTR(domain.as_mut_ptr()),
                    &mut domain_len,
                    &mut usage,
                )
                .is_ok()
                {
                    let n = crate::util::from_wide(&name[..name_len as usize]);
                    if !n.is_empty() {
                        ret = Some(n);
                    }
                }
            }
        }
    }
    let _ = CloseHandle(token);
    ret
}

fn pid_session(pid: u32) -> u32 {
    unsafe {
        let mut sid: u32 = 0;
        let _ = ProcessIdToSessionId(pid, &mut sid);
        sid
    }
}

fn current_session() -> u32 {
    unsafe { pid_session(GetCurrentProcessId()) }
}

pub fn collect_user_sessions(processes: Vec<ProcessInfo>) -> Vec<UserSession> {
    let cur = current_session();
    let mut by_session: HashMap<u32, (String, Vec<ProcessInfo>)> = HashMap::new();

    for mut p in processes {
        if p.pid == 0 {
            continue;
        }
        let sid = pid_session(p.pid);
        let user = unsafe {
            if let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, p.pid) {
                let owner = read_owner(h);
                let _ = CloseHandle(h);
                owner.unwrap_or_else(|| format!("Session {sid}"))
            } else {
                format!("Session {sid}")
            }
        };
        p.user = Some(user.clone());
        by_session
            .entry(sid)
            .or_insert_with(|| (user, Vec::new()))
            .1
            .push(p);
    }

    let mut out: Vec<UserSession> = by_session
        .into_iter()
        .map(|(sid, (user, mut procs))| {
            procs.sort_by(|a, b| b.memory_kb.cmp(&a.memory_kb));
            UserSession {
                user,
                session_id: sid,
                is_current: sid == cur,
                processes: procs,
            }
        })
        .collect();
    out.sort_by(|a, b| b.session_id.cmp(&a.session_id));
    out
}
