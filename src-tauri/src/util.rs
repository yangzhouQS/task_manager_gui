use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::SystemInformation::GetSystemInfo;
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::System::WindowsProgramming::QueryIdleProcessorCycleTime;

pub fn from_wide(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn logical_processor_count() -> u32 {
    unsafe {
        let mut info = std::mem::zeroed();
        GetSystemInfo(&mut info);
        info.dwNumberOfProcessors.max(1)
    }
}

pub fn query_idle_cycles(num_cores: u32) -> u64 {
    unsafe {
        let mut buf = vec![0u64; num_cores as usize];
        let mut bytes = (buf.len() * std::mem::size_of::<u64>()) as u32;
        let _ = QueryIdleProcessorCycleTime(&mut bytes, Some(buf.as_mut_ptr()));
        buf.iter().sum()
    }
}

pub fn is_elevated() -> bool {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut 0u32,
        );
        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}
