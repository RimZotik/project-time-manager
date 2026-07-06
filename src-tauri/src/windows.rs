use crate::models::WindowObservation;
#[cfg(target_os = "windows")]
use std::path::Path;

#[cfg(target_os = "windows")]
pub fn capture_active_window() -> Option<WindowObservation> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::core::PWSTR;
    use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        let mut title_buffer = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buffer);
        let window_title = String::from_utf16_lossy(&title_buffer[..title_len.max(0) as usize]);

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        let handle: HANDLE = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(handle) => handle,
            Err(_) => return Some(build_observation("unknown", "", window_title, hwnd)),
        };
        if handle.is_invalid() {
            return Some(build_observation("unknown", "", window_title, hwnd));
        }

        let mut buffer = [0u16; 260];
        let mut size = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(handle);

        let process_path = if ok.is_ok() && size > 0 {
            let raw = OsString::from_wide(&buffer[..size as usize]);
            raw.to_string_lossy().to_string()
        } else {
            String::new()
        };
        let process_name = if process_path.is_empty() {
            "unknown".to_string()
        } else {
            path_file_name(&process_path)
        };

        Some(build_observation(
            &process_name,
            &process_path,
            window_title,
            hwnd,
        ))
    }
}

#[cfg(not(target_os = "windows"))]
pub fn capture_active_window() -> Option<WindowObservation> {
    None
}

#[cfg(target_os = "windows")]
fn build_observation(
    process_name: &str,
    process_path: &str,
    window_title: String,
    hwnd: windows::Win32::Foundation::HWND,
) -> WindowObservation {
    let browser_name = classify_browser(process_name);
    let url = browser_name
        .as_ref()
        .and_then(|_| browser_active_url(hwnd))
        .filter(|value| looks_like_url(value));
    let favicon_url = url.as_deref().and_then(favicon_url);
    let tab_title = browser_name
        .as_ref()
        .map(|browser| strip_browser_suffix(&window_title, browser))
        .or_else(|| Some(window_title.clone()));

    WindowObservation {
        process_name: process_name.to_string(),
        process_path: process_path.to_string(),
        icon_data_url: icon_data_url(process_path),
        window_title,
        browser_name,
        tab_title,
        url,
        favicon_url,
    }
}

#[cfg(target_os = "windows")]
fn browser_active_url(hwnd: windows::Win32::Foundation::HWND) -> Option<String> {
    use windows::core::VARIANT;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
    };
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, IUIAutomationValuePattern, TreeScope_Descendants,
        UIA_ControlTypePropertyId, UIA_EditControlTypeId, UIA_ValuePatternId,
    };

    unsafe {
        let initialized =
            CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).is_ok();
        let result = (|| {
            let automation: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()?;
            let root = automation.ElementFromHandle(hwnd).ok()?;
            let condition = automation
                .CreatePropertyCondition(
                    UIA_ControlTypePropertyId,
                    &VARIANT::from(UIA_EditControlTypeId.0),
                )
                .ok()?;
            let element = root.FindFirst(TreeScope_Descendants, &condition).ok()?;
            let value_pattern: IUIAutomationValuePattern =
                element.GetCurrentPatternAs(UIA_ValuePatternId).ok()?;
            let value = value_pattern.CurrentValue().ok()?.to_string();
            normalize_url_value(&value)
        })();
        if initialized {
            CoUninitialize();
        }
        result
    }
}

#[cfg(target_os = "windows")]
fn normalize_url_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("file://")
    {
        return Some(trimmed.to_string());
    }
    if trimmed.contains('.') && !trimmed.contains(' ') {
        return Some(format!("https://{trimmed}"));
    }
    None
}

#[cfg(target_os = "windows")]
fn looks_like_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://") || value.starts_with("file://")
}

#[cfg(target_os = "windows")]
fn favicon_url(url: &str) -> Option<String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return None;
    }
    Some(format!(
        "https://www.google.com/s2/favicons?sz=32&domain_url={}",
        percent_encode(url)
    ))
}

#[cfg(target_os = "windows")]
fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

#[cfg(target_os = "windows")]
fn icon_data_url(process_path: &str) -> Option<String> {
    use std::collections::HashMap;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::sync::{Mutex, OnceLock};
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;

    static ICON_CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
    let cache = ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if process_path.trim().is_empty() {
        return None;
    }

    if let Ok(guard) = cache.lock() {
        if let Some(cached) = guard.get(process_path) {
            return cached.clone();
        }
    }

    let extracted = unsafe {
        let wide_path: Vec<u16> = OsStr::new(process_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut file_info = SHFILEINFOW::default();
        let result = SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut file_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );

        if result == 0 || file_info.hIcon.0.is_null() {
            None
        } else {
            let converted = hicon_to_png_data_url(file_info.hIcon);
            let _ = DestroyIcon(file_info.hIcon);
            converted
        }
    };

    if let Ok(mut guard) = cache.lock() {
        guard.insert(process_path.to_string(), extracted.clone());
    }

    extracted
}

#[cfg(target_os = "windows")]
unsafe fn hicon_to_png_data_url(
    hicon: windows::Win32::UI::WindowsAndMessaging::HICON,
) -> Option<String> {
    use base64::Engine;
    use image::ImageEncoder;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
        SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
    };
    use windows::Win32::UI::WindowsAndMessaging::{DrawIconEx, DI_NORMAL};

    const ICON_SIZE: i32 = 32;
    let screen_dc = GetDC(HWND::default());
    if screen_dc.0.is_null() {
        return None;
    }

    let memory_dc = CreateCompatibleDC(screen_dc);
    if memory_dc.0.is_null() {
        let _ = ReleaseDC(HWND::default(), screen_dc);
        return None;
    }

    let mut bits = std::ptr::null_mut();
    let bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: ICON_SIZE,
            biHeight: -ICON_SIZE,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let bitmap = match CreateDIBSection(memory_dc, &bitmap_info, DIB_RGB_COLORS, &mut bits, None, 0)
    {
        Ok(bitmap) => bitmap,
        Err(_) => {
            let _ = DeleteDC(memory_dc);
            let _ = ReleaseDC(HWND::default(), screen_dc);
            return None;
        }
    };
    if bits.is_null() {
        let _ = DeleteDC(memory_dc);
        let _ = ReleaseDC(HWND::default(), screen_dc);
        return None;
    }

    let old_object = SelectObject(memory_dc, HGDIOBJ(bitmap.0));
    let draw_ok = DrawIconEx(
        memory_dc, 0, 0, hicon, ICON_SIZE, ICON_SIZE, 0, None, DI_NORMAL,
    )
    .is_ok();

    let output = if draw_ok {
        let bytes_len = (ICON_SIZE * ICON_SIZE * 4) as usize;
        let bgra = std::slice::from_raw_parts(bits as *const u8, bytes_len);
        let mut rgba = Vec::with_capacity(bytes_len);
        for pixel in bgra.chunks_exact(4) {
            rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }

        let mut png = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png);
        if encoder
            .write_image(
                &rgba,
                ICON_SIZE as u32,
                ICON_SIZE as u32,
                image::ColorType::Rgba8.into(),
            )
            .is_ok()
        {
            Some(format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(png)
            ))
        } else {
            None
        }
    } else {
        None
    };

    let _ = SelectObject(memory_dc, old_object);
    let _ = DeleteObject(HGDIOBJ(bitmap.0));
    let _ = DeleteDC(memory_dc);
    let _ = ReleaseDC(HWND::default(), screen_dc);

    output
}

#[cfg(target_os = "windows")]
fn classify_browser(process_name: &str) -> Option<String> {
    let lower = process_name.to_lowercase();
    if lower.contains("chrome") {
        Some("Google Chrome".to_string())
    } else if lower.contains("msedge") || lower.contains("edge") {
        Some("Microsoft Edge".to_string())
    } else if lower.contains("firefox") {
        Some("Mozilla Firefox".to_string())
    } else if lower.contains("brave") {
        Some("Brave".to_string())
    } else if lower.contains("opera") {
        Some("Opera".to_string())
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
fn strip_browser_suffix(title: &str, browser: &str) -> String {
    let suffix = format!(" - {browser}");
    title
        .strip_suffix(&suffix)
        .unwrap_or(title)
        .trim()
        .to_string()
}

#[cfg(target_os = "windows")]
fn path_file_name(value: &str) -> String {
    Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(value)
        .to_string()
}
