use windows::core::{PCWSTR, PWSTR};
use windows::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCollectQueryData, PdhExpandWildCardPathW, PdhGetFormattedCounterValue,
    PdhOpenQueryW, PDH_FMT, PDH_FMT_COUNTERVALUE,
};

const FMT: u32 = 0x0000_0200 | 0x0000_8000; // PDH_FMT_DOUBLE | PDH_FMT_NOCAP100

pub struct Pdh {
    query: isize,
}

impl Pdh {
    pub fn new() -> anyhow::Result<Self> {
        unsafe {
            let mut query: isize = 0;
            let status = PdhOpenQueryW(PCWSTR::null(), 0, &mut query);
            if status != 0 {
                anyhow::bail!("PdhOpenQuery 失败 {status:#x}");
            }
            Ok(Pdh { query })
        }
    }

    pub fn add(&self, path: &str) -> anyhow::Result<isize> {
        unsafe {
            let w = crate::util::to_wide(path);
            let mut counter: isize = 0;
            let status = PdhAddEnglishCounterW(self.query, PCWSTR(w.as_ptr()), 0, &mut counter);
            if status != 0 {
                anyhow::bail!("PdhAddCounter 失败 {status:#x} 路径 {path}");
            }
            Ok(counter)
        }
    }

    pub fn collect(&self) {
        unsafe {
            let _ = PdhCollectQueryData(self.query);
        }
    }

    pub fn value(&self, counter: isize) -> Option<f64> {
        unsafe {
            let mut value = PDH_FMT_COUNTERVALUE::default();
            let mut typ: u32 = 0;
            let status = PdhGetFormattedCounterValue(
                counter,
                PDH_FMT(FMT),
                Some(&mut typ as *mut u32),
                &mut value,
            );
            if status != 0 || value.CStatus != 0 {
                return None;
            }
            Some(value.Anonymous.doubleValue)
        }
    }
}

pub fn expand(path: &str) -> Vec<String> {
    unsafe {
        let w = crate::util::to_wide(path);
        let mut size: u32 = 0;
        PdhExpandWildCardPathW(PCWSTR::null(), PCWSTR(w.as_ptr()), PWSTR::null(), &mut size, 0);
        if size == 0 {
            return Vec::new();
        }
        let mut buf = vec![0u16; size as usize];
        let status = PdhExpandWildCardPathW(
            PCWSTR::null(),
            PCWSTR(w.as_ptr()),
            PWSTR(buf.as_mut_ptr()),
            &mut size,
            0,
        );
        if status != 0 {
            return Vec::new();
        }
        split_multi_sz(&buf[..size as usize])
    }
}

fn split_multi_sz(buf: &[u16]) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0;
    for i in 0..buf.len() {
        if buf[i] == 0 {
            if i == start {
                break;
            }
            out.push(String::from_utf16_lossy(&buf[start..i]));
            start = i + 1;
        }
    }
    out
}

pub fn parse_instance(full: &str) -> Option<String> {
    if let (Some(open), Some(close)) = (full.find('('), full.find(')')) {
        if open < close {
            return Some(full[open + 1..close].to_string());
        }
    }
    None
}
