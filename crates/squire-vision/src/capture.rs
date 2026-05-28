use serde::Serialize;
use squire_error::{Result, SquireError};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Arc<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub process_name: String,
}

#[cfg(windows)]
pub fn find_window(title: &str) -> Result<isize> {
    use windows::core::PCSTR;
    use windows::Win32::UI::WindowsAndMessaging::FindWindowA;

    let title_cstr = std::ffi::CString::new(title)
        .map_err(|e| SquireError::WindowNotFound(e.to_string()))?;

    let hwnd = unsafe { FindWindowA(PCSTR::null(), PCSTR(title_cstr.as_ptr() as *const u8)) }
        .map_err(|_| SquireError::WindowNotFound(title.to_string()))?;

    if hwnd.0 == std::ptr::null_mut() {
        return Err(SquireError::WindowNotFound(title.to_string()));
    }

    Ok(hwnd.0 as isize)
}

#[cfg(windows)]
pub fn capture_window(hwnd: isize) -> Result<Image> {
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC,
        DeleteObject, GetDIBits, GetDC, ReleaseDC, SelectObject,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

    let hwnd_val = HWND(hwnd as *mut _);
    let mut rect = RECT::default();
    unsafe { GetClientRect(hwnd_val, &mut rect) }
        .map_err(|e| SquireError::Vision(format!("GetClientRect: {}", e)))?;

    let width = (rect.right - rect.left) as u32;
    let height = (rect.bottom - rect.top) as u32;

    if width == 0 || height == 0 {
        return Err(SquireError::Vision("Window has zero size".to_string()));
    }

    let hdc_window = unsafe { GetDC(hwnd_val) };
    let hdc_mem = unsafe { CreateCompatibleDC(hdc_window) };
    let hbm = unsafe { CreateCompatibleBitmap(hdc_window, width as i32, height as i32) };

    let old_obj = unsafe { SelectObject(hdc_mem, hbm) };

    let blt_result = unsafe {
        BitBlt(
            hdc_mem,
            0, 0,
            width as i32, height as i32,
            hdc_window,
            0, 0,
            SRCCOPY,
        )
    };

    if let Err(e) = blt_result {
        unsafe {
            SelectObject(hdc_mem, old_obj);
            let _ = DeleteObject(hbm);
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(hwnd_val, hdc_window);
        }
        return Err(SquireError::Vision(format!("BitBlt: {}", e)));
    }

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut buffer: Vec<u8> = vec![0u8; (width * height * 4) as usize];
    unsafe {
        GetDIBits(
            hdc_mem,
            hbm,
            0,
            height,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        )
    };

    unsafe {
        SelectObject(hdc_mem, old_obj);
        let _ = DeleteObject(hbm);
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(hwnd_val, hdc_window);
    }

    Ok(Image {
        width,
        height,
        data: Arc::new(buffer),
    })
}

#[cfg(not(windows))]
pub fn find_window(_title: &str) -> Result<isize> {
    Err(SquireError::Vision("Not supported on this platform".to_string()))
}

#[cfg(not(windows))]
pub fn capture_window(_hwnd: isize) -> Result<Image> {
    Err(SquireError::Vision("Not supported on this platform".to_string()))
}

#[cfg(windows)]
pub fn list_windows() -> Vec<WindowInfo> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::sync::Mutex;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        IsWindowVisible,
    };

    let results: Arc<Mutex<Vec<WindowInfo>>> = Arc::new(Mutex::new(Vec::new()));
    let results_clone = results.clone();

    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let results = &*(lparam.0 as *const Mutex<Vec<WindowInfo>>);

        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }

        let title_len = GetWindowTextLengthW(hwnd);
        if title_len == 0 {
            return BOOL(1);
        }

        let mut title_buf = vec![0u16; (title_len + 1) as usize];
        GetWindowTextW(hwnd, &mut title_buf);
        let title = OsString::from_wide(&title_buf[..title_len as usize])
            .to_string_lossy()
            .to_string();

        let skip_titles = ["Default IME", "MSCTFIME UI", "Program Manager"];
        if skip_titles.iter().any(|s| title == *s) {
            return BOOL(1);
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        let process_name = get_process_name(pid).unwrap_or_default();

        if let Ok(mut list) = results.lock() {
            list.push(WindowInfo {
                hwnd: hwnd.0 as isize,
                title,
                process_name,
            });
        }

        BOOL(1)
    }

    unsafe {
        let _ = EnumWindows(
            Some(enum_callback),
            LPARAM(&*results_clone as *const Mutex<Vec<WindowInfo>> as isize),
        );
    }

    Arc::try_unwrap(results)
        .unwrap_or_else(|arc| (*arc).lock().unwrap().clone().into())
        .into_inner()
        .unwrap_or_default()
}

#[cfg(windows)]
fn get_process_name(pid: u32) -> Option<String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::core::PWSTR;
    use windows::Win32::Foundation::MAX_PATH;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = vec![0u16; MAX_PATH as usize];
        let mut len = buf.len() as u32;
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
        .ok()?;
        let _ = windows::Win32::Foundation::CloseHandle(handle);
        let path = OsString::from_wide(&buf[..len as usize])
            .to_string_lossy()
            .to_string();
        path.rsplit('\\').next().map(|s| s.to_string())
    }
}

#[cfg(not(windows))]
pub fn list_windows() -> Vec<WindowInfo> {
    Vec::new()
}
