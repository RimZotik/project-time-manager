use crate::models::WindowObservation;
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
            Err(_) => return Some(build_observation("unknown", window_title)),
        };
        if handle.is_invalid() {
            return Some(build_observation("unknown", window_title));
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

        let process_name = if ok.is_ok() && size > 0 {
            let raw = OsString::from_wide(&buffer[..size as usize]);
            path_file_name(&raw.to_string_lossy())
        } else {
            "unknown".to_string()
        };

        Some(build_observation(&process_name, window_title))
    }
}

#[cfg(not(target_os = "windows"))]
pub fn capture_active_window() -> Option<WindowObservation> {
    None
}

fn build_observation(process_name: &str, window_title: String) -> WindowObservation {
    let browser_name = classify_browser(process_name);
    let tab_title = browser_name
        .as_ref()
        .map(|browser| strip_browser_suffix(&window_title, browser))
        .or_else(|| Some(window_title.clone()));

    WindowObservation {
        process_name: process_name.to_string(),
        window_title,
        browser_name,
        tab_title,
        url: None,
    }
}

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

fn strip_browser_suffix(title: &str, browser: &str) -> String {
    let suffix = format!(" - {browser}");
    title
        .strip_suffix(&suffix)
        .unwrap_or(title)
        .trim()
        .to_string()
}

fn path_file_name(value: &str) -> String {
    Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(value)
        .to_string()
}
