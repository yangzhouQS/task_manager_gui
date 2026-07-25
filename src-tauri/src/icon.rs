use std::collections::HashMap;
use std::sync::Mutex;

use base64::prelude::*;
use once_cell::sync::Lazy;
use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC,
    SelectObject, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, HBITMAP,
};
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

static CACHE: Lazy<Mutex<HashMap<String, Option<String>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// 从已缓存的结果中取图标 data URL；未提取过返回 None。
pub fn get(path: &str) -> Option<String> {
    CACHE
        .lock()
        .ok()
        .and_then(|c| c.get(path).cloned().flatten())
}

/// 是否已尝试提取（成功或失败）。
pub fn is_known(path: &str) -> bool {
    CACHE.lock().map(|c| c.contains_key(path)).unwrap_or(false)
}

/// 提取并缓存图标；返回 data URL（失败则缓存 None 并返回 None）。
pub fn extract(path: &str) -> Option<String> {
    if let Ok(c) = CACHE.lock() {
        if let Some(v) = c.get(path) {
            return v.clone();
        }
    }
    let result = extract_inner(path);
    if let Ok(mut c) = CACHE.lock() {
        c.insert(path.to_string(), result.clone());
    }
    result
}

fn extract_inner(path: &str) -> Option<String> {
    unsafe {
        let wpath = crate::util::to_wide(path);
        let mut shfi = SHFILEINFOW::default();
        let flags = SHGFI_ICON | SHGFI_LARGEICON;
        let ok = SHGetFileInfoW(
            PCWSTR(wpath.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            flags,
        );
        if ok == 0 || shfi.hIcon.is_invalid() {
            return None;
        }
        let rgba = icon_to_rgba(shfi.hIcon);
        let _ = DestroyIcon(shfi.hIcon);
        let (w, h, data) = rgba?;
        let png = encode_png(w, h, &data)?;
        Some(format!(
            "data:image/png;base64,{}",
            BASE64_STANDARD.encode(&png)
        ))
    }
}

unsafe fn icon_to_rgba(hicon: HICON) -> Option<(u32, u32, Vec<u8>)> {
    let mut ii = ICONINFO::default();
    if GetIconInfo(hicon, &mut ii).is_err() {
        return None;
    }

    let result = read_color(ii.hbmColor, ii.hbmMask);
    cleanup(ii);
    result
}

unsafe fn read_color(hbm_color: HBITMAP, hbm_mask: HBITMAP) -> Option<(u32, u32, Vec<u8>)> {
    let mut bmp = BITMAP::default();
    if GetObjectW(
        hbm_color,
        std::mem::size_of::<BITMAP>() as i32,
        Some(&mut bmp as *mut _ as *mut _),
    ) == 0
    {
        return None;
    }
    let w = bmp.bmWidth as u32;
    let h = bmp.bmHeight as u32;
    if w == 0 || h == 0 {
        return None;
    }

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w as i32,
            biHeight: -(h as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0,
            ..std::mem::zeroed()
        },
        ..std::mem::zeroed()
    };

    let hdc = GetDC(None);
    let mdc = CreateCompatibleDC(hdc);
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    let old = SelectObject(mdc, hbm_color);
    let got = GetDIBits(
        mdc,
        hbm_color,
        0,
        h,
        Some(pixels.as_mut_ptr() as *mut _),
        &mut bmi,
        DIB_RGB_COLORS,
    );
    SelectObject(mdc, old);
    let _ = DeleteDC(mdc);
    let _ = ReleaseDC(None, hdc);
    if got == 0 {
        return None;
    }

    let has_alpha = pixels.chunks_exact(4).any(|c| c[3] != 0);
    if !has_alpha && !hbm_mask.is_invalid() {
        apply_mask(hbm_mask, w, h, &mut pixels);
    }

    for px in pixels.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
    Some((w, h, pixels))
}

unsafe fn apply_mask(hbm_mask: HBITMAP, w: u32, h: u32, pixels: &mut [u8]) {
    let row = ((w + 31) / 32) * 4;
    let mut mask = vec![0u8; (row * h) as usize];

    let mut header = [0u8; 40 + 8];
    header[0..4].copy_from_slice(&40u32.to_le_bytes());
    header[4..8].copy_from_slice(&(w as i32).to_le_bytes());
    header[8..12].copy_from_slice(&(-(h as i32)).to_le_bytes());
    header[12..14].copy_from_slice(&1u16.to_le_bytes()); // planes
    header[14..16].copy_from_slice(&1u16.to_le_bytes()); // bitcount

    let hdc = GetDC(None);
    let mdc = CreateCompatibleDC(hdc);
    let old = SelectObject(mdc, hbm_mask);
    let _ = GetDIBits(
        mdc,
        hbm_mask,
        0,
        h,
        Some(mask.as_mut_ptr() as *mut _),
        header.as_mut_ptr() as *mut BITMAPINFO,
        DIB_RGB_COLORS,
    );
    SelectObject(mdc, old);
    let _ = DeleteDC(mdc);
    let _ = ReleaseDC(None, hdc);

    for y in 0..h {
        for x in 0..w {
            let byte = mask[(y * row + x / 8) as usize];
            let transparent = (byte >> (7 - (x % 8))) & 1;
            let alpha = if transparent == 0 { 255 } else { 0 };
            pixels[((y * w + x) * 4 + 3) as usize] = alpha;
        }
    }
}

unsafe fn cleanup(ii: ICONINFO) {
    if !ii.hbmColor.is_invalid() {
        let _ = DeleteObject(ii.hbmColor);
    }
    if !ii.hbmMask.is_invalid() {
        let _ = DeleteObject(ii.hbmMask);
    }
}

fn encode_png(w: u32, h: u32, data: &[u8]) -> Option<Vec<u8>> {
    let img = image::RgbaImage::from_raw(w, h, data.to_vec())?;
    let dyn_img = image::DynamicImage::ImageRgba8(img);
    let mut out = Vec::new();
    dyn_img
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .ok()?;
    Some(out)
}

/// 从命令行/快捷方式路径解析出用于取图标的可执行路径。
pub fn exe_from_command(cmd: &str) -> Option<String> {
    let s = cmd.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(stripped) = s.strip_prefix('"') {
        if let Some(end) = stripped.find('"') {
            return Some(stripped[..end].to_string());
        }
    }
    s.split_whitespace().next().map(|t| t.to_string())
}
