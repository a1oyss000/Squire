use squire_error::{Result, SquireError};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Arc<Vec<u8>>,
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
