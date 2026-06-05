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
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

    let hwnd = unsafe { FindWindowW(PCWSTR::null(), PCWSTR(title_wide.as_ptr())) }
        .map_err(|_| SquireError::WindowNotFound(title.to_string()))?;

    if hwnd.0 == std::ptr::null_mut() {
        return Err(SquireError::WindowNotFound(title.to_string()));
    }

    Ok(hwnd.0 as isize)
}

#[cfg(windows)]
pub fn capture_window(hwnd: isize, ctx: &WgcCaptureContext) -> Result<Image> {
    match capture_window_wgc(hwnd, ctx) {
        Ok(image) => Ok(image),
        Err(e) => {
            tracing::warn!("WGC capture failed ({}), falling back to BitBlt", e);
            capture_window_bitblt(hwnd)
        }
    }
}

/// Persistent D3D11 resources reused across WGC captures.
///
/// Creating a D3D11 device per capture exhausts GPU memory and can cause
/// the DWM to reset window composition (windows minimize). This context
/// holds the heavy resources once so they're shared across captures.
///
/// # Safety
/// D3D11 devices are internally synchronized when created without
/// `D3D11_CREATE_DEVICE_SINGLETHREADED`. The contained COM interfaces
/// are safe to use from multiple threads.
#[cfg(windows)]
pub struct WgcCaptureContext {
    device: windows::Win32::Graphics::Direct3D11::ID3D11Device,
    context: windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext,
    d3d_device: windows::Graphics::DirectX::Direct3D11::IDirect3DDevice,
}

// SAFETY: D3D11 device/context are internally synchronized (created without
// SINGLETHREADED flag). The WinRT IDirect3DDevice wraps the same D3D device.
// All access is read-only after initialization (no mutable state in the context).
#[cfg(windows)]
unsafe impl Send for WgcCaptureContext {}
#[cfg(windows)]
unsafe impl Sync for WgcCaptureContext {}

#[cfg(windows)]
impl WgcCaptureContext {
    pub fn new() -> Result<Self> {
        use windows::core::Interface;
        use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
        use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
        use windows::Win32::Graphics::Direct3D11::{
            D3D11CreateDevice, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
        };
        use windows::Win32::Graphics::Dxgi::IDXGIDevice;
        use windows::Win32::System::WinRT::Direct3D11::CreateDirect3D11DeviceFromDXGIDevice;

        unsafe {
            let mut device = None;
            let mut context = None;
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            ).map_err(|e| SquireError::Vision(format!("D3D11CreateDevice: {}", e)))?;

            let device = device.unwrap();
            let context = context.unwrap();

            let dxgi_device: IDXGIDevice = device.cast()
                .map_err(|e| SquireError::Vision(format!("IDXGIDevice: {}", e)))?;
            let inspectable = CreateDirect3D11DeviceFromDXGIDevice(&dxgi_device)
                .map_err(|e| SquireError::Vision(format!("WinRT device: {}", e)))?;
            let d3d_device: IDirect3DDevice = inspectable.cast()
                .map_err(|e| SquireError::Vision(format!("IDirect3DDevice: {}", e)))?;

            Ok(WgcCaptureContext { device, context, d3d_device })
        }
    }
}

#[cfg(windows)]
fn capture_window_wgc(hwnd: isize, ctx: &WgcCaptureContext) -> Result<Image> {
    use windows::core::Interface;
    use windows::Graphics::Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem};
    use windows::Graphics::DirectX::DirectXPixelFormat;
    use windows::Win32::Foundation::{HWND, POINT, RECT, WAIT_OBJECT_0};
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_CPU_ACCESS_READ, D3D11_MAP_READ,
        D3D11_MAPPED_SUBRESOURCE,
        D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
        ID3D11Resource, ID3D11Texture2D,
    };
    use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
    use windows::Win32::System::WinRT::Direct3D11::IDirect3DDxgiInterfaceAccess;
    use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
    use windows::Win32::UI::WindowsAndMessaging::{GetClientRect, GetWindowRect};

    unsafe {
        // 1. Create GraphicsCaptureItem from HWND (per-capture, lightweight)
        let interop: IGraphicsCaptureItemInterop =
            windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
                .map_err(|e| SquireError::Vision(format!("Interop factory: {}", e)))?;
        let item: GraphicsCaptureItem = interop.CreateForWindow(HWND(hwnd as *mut _))
            .map_err(|e| SquireError::Vision(format!("CreateForWindow: {}", e)))?;
        let size = item.Size()
            .map_err(|e| SquireError::Vision(format!("Item size: {}", e)))?;

        // 2. Create frame pool and session (per-capture, lightweight)
        let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &ctx.d3d_device,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            1,
            size,
        ).map_err(|e| SquireError::Vision(format!("FramePool: {}", e)))?;

        let session = frame_pool.CreateCaptureSession(&item)
            .map_err(|e| SquireError::Vision(format!("CaptureSession: {}", e)))?;

        let _ = session.SetIsBorderRequired(false);
        let _ = session.SetIsCursorCaptureEnabled(false);

        // 3. Set up event for synchronization
        let event = CreateEventW(None, true, false, None)
            .map_err(|e| SquireError::Vision(format!("CreateEvent: {}", e)))?;

        let event_ptr = event.0 as usize;
        frame_pool.FrameArrived(&windows::Foundation::TypedEventHandler::new(
            move |_, _| {
                let h = windows::Win32::Foundation::HANDLE(event_ptr as *mut _);
                let _ = windows::Win32::System::Threading::SetEvent(h);
                Ok(())
            },
        )).map_err(|e| SquireError::Vision(format!("FrameArrived: {}", e)))?;

        // 4. Start capture and wait for frame
        session.StartCapture()
            .map_err(|e| SquireError::Vision(format!("StartCapture: {}", e)))?;

        let wait_result = WaitForSingleObject(event, 2000);
        if wait_result != WAIT_OBJECT_0 {
            let _ = session.Close();
            let _ = frame_pool.Close();
            return Err(SquireError::Vision("WGC frame capture timed out".into()));
        }

        // 5. Get frame and extract texture
        let frame = frame_pool.TryGetNextFrame()
            .map_err(|e| SquireError::Vision(format!("TryGetNextFrame: {}", e)))?;
        let surface = frame.Surface()
            .map_err(|e| SquireError::Vision(format!("Frame surface: {}", e)))?;
        let access: IDirect3DDxgiInterfaceAccess = surface.cast()
            .map_err(|e| SquireError::Vision(format!("DxgiAccess: {}", e)))?;
        let texture: ID3D11Texture2D = access.GetInterface()
            .map_err(|e| SquireError::Vision(format!("GetInterface: {}", e)))?;

        // 6. Create staging texture and copy (per-capture — size may change)
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        texture.GetDesc(&mut desc);
        desc.Usage = D3D11_USAGE_STAGING;
        desc.BindFlags = 0;
        desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
        desc.MiscFlags = 0;

        let mut staging: Option<ID3D11Texture2D> = None;
        ctx.device.CreateTexture2D(&desc, None, Some(&mut staging))
            .map_err(|e| SquireError::Vision(format!("Staging texture: {}", e)))?;
        let staging = staging.unwrap();

        let staging_res: ID3D11Resource = staging.cast()
            .map_err(|e| SquireError::Vision(format!("staging cast: {}", e)))?;
        let texture_res: ID3D11Resource = texture.cast()
            .map_err(|e| SquireError::Vision(format!("texture cast: {}", e)))?;
        ctx.context.CopyResource(&staging_res, &texture_res);

        // 7. Map and read pixels
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        ctx.context.Map(&staging_res, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
            .map_err(|e| SquireError::Vision(format!("Map: {}", e)))?;

        let width = desc.Width;
        let height = desc.Height;
        let row_pitch = mapped.RowPitch as usize;
        let mut buffer = vec![0u8; (width * height * 4) as usize];

        let src = mapped.pData as *const u8;
        for y in 0..height as usize {
            let src_row = src.add(y * row_pitch);
            let dst_offset = y * (width as usize) * 4;
            std::ptr::copy_nonoverlapping(
                src_row, buffer.as_mut_ptr().add(dst_offset), (width as usize) * 4,
            );
        }

        ctx.context.Unmap(&staging_res, 0);

        // 8. Cleanup
        let _ = session.Close();
        let _ = frame_pool.Close();
        let _ = windows::Win32::Foundation::CloseHandle(event);

        // 9. Crop to client area — WGC captures the full window including
        //    title bar, but coordinates are used as client-area offsets.
        let hwnd_val = HWND(hwnd as *mut _);
        let mut window_rect = RECT::default();
        let mut client_rect = RECT::default();
        let _ = GetWindowRect(hwnd_val, &mut window_rect);
        let _ = GetClientRect(hwnd_val, &mut client_rect);

        let mut client_origin = POINT { x: 0, y: 0 };
        let _ = windows::Win32::Graphics::Gdi::ClientToScreen(
            hwnd_val, &mut client_origin,
        );

        let offset_x = (client_origin.x - window_rect.left) as u32;
        let offset_y = (client_origin.y - window_rect.top) as u32;
        let client_w = (client_rect.right - client_rect.left) as u32;
        let client_h = (client_rect.bottom - client_rect.top) as u32;

        let client_w = client_w.min(width.saturating_sub(offset_x));
        let client_h = client_h.min(height.saturating_sub(offset_y));

        if client_w == 0 || client_h == 0 {
            return Ok(Image { width, height, data: Arc::new(buffer) });
        }

        let mut cropped = vec![0u8; (client_w * client_h * 4) as usize];
        for y in 0..client_h as usize {
            let src_start = ((y + offset_y as usize) * width as usize
                + offset_x as usize) * 4;
            let dst_start = y * client_w as usize * 4;
            let row_bytes = client_w as usize * 4;
            cropped[dst_start..dst_start + row_bytes]
                .copy_from_slice(&buffer[src_start..src_start + row_bytes]);
        }

        Ok(Image {
            width: client_w,
            height: client_h,
            data: Arc::new(cropped),
        })
    }
}

#[cfg(windows)]
fn capture_window_bitblt(hwnd: isize) -> Result<Image> {
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
        return Err(SquireError::Vision("Window has zero size".into()));
    }

    let hdc_window = unsafe { GetDC(hwnd_val) };
    let hdc_mem = unsafe { CreateCompatibleDC(hdc_window) };
    let hbm = unsafe {
        CreateCompatibleBitmap(hdc_window, width as i32, height as i32)
    };
    let old_obj = unsafe { SelectObject(hdc_mem, hbm) };

    let blt_result = unsafe {
        BitBlt(hdc_mem, 0, 0, width as i32, height as i32,
               hdc_window, 0, 0, SRCCOPY)
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

    let mut buffer = vec![0u8; (width * height * 4) as usize];
    let bmi = BITMAPINFO {
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

    let lines = unsafe {
        GetDIBits(
            hdc_mem, hbm, 0, height,
            Some(buffer.as_mut_ptr() as *mut _),
            &bmi as *const _ as *mut _,
            DIB_RGB_COLORS,
        )
    };

    unsafe {
        SelectObject(hdc_mem, old_obj);
        let _ = DeleteObject(hbm);
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(hwnd_val, hdc_window);
    }

    if lines == 0 {
        return Err(SquireError::Vision("GetDIBits returned 0 lines".into()));
    }

    Ok(Image {
        width,
        height,
        data: Arc::new(buffer),
    })
}

#[cfg(windows)]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
    };

    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);
        if IsWindowVisible(hwnd).as_bool() {
            let len = GetWindowTextLengthW(hwnd);
            if len > 0 {
                let mut buf = vec![0u16; (len + 1) as usize];
                let actual_len = GetWindowTextW(hwnd, &mut buf);
                let title = String::from_utf16_lossy(&buf[..actual_len as usize]);
                if !title.is_empty() {
                    let process_name = get_process_name(hwnd.0 as isize)
                        .unwrap_or_default();
                    windows.push(WindowInfo {
                        hwnd: hwnd.0 as isize,
                        title,
                        process_name,
                    });
                }
            }
        }
        BOOL(1)
    }

    let mut windows: Vec<WindowInfo> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(enum_callback),
            LPARAM(&mut windows as *mut _ as isize),
        );
    }
    Ok(windows)
}

#[cfg(windows)]
pub fn get_process_name(hwnd: isize) -> Result<String> {
    use windows::core::PSTR;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameA,
        PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(HWND(hwnd as *mut _), Some(&mut pid)) };

    if pid == 0 {
        return Ok(String::new());
    }

    let handle = unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
    }.map_err(|e| SquireError::Vision(format!("OpenProcess: {}", e)))?;

    let mut buf = vec![0u8; 260];
    let mut len = buf.len() as u32;
    let ok = unsafe {
        QueryFullProcessImageNameA(
            handle,
            PROCESS_NAME_FORMAT(0),
            PSTR(buf.as_mut_ptr()),
            &mut len,
        )
    };

    unsafe { let _ = windows::Win32::Foundation::CloseHandle(handle); }

    if ok.is_err() {
        return Ok(String::new());
    }

    let path = String::from_utf8_lossy(&buf[..len as usize]).to_string();
    Ok(path.rsplit('\\').next().unwrap_or("").to_string())
}

#[cfg(not(windows))]
pub fn find_window(_title: &str) -> Result<isize> {
    Err(SquireError::Vision("Not supported on this platform".into()))
}

#[cfg(not(windows))]
pub struct WgcCaptureContext;

#[cfg(not(windows))]
impl WgcCaptureContext {
    pub fn new() -> Result<Self> {
        Err(SquireError::Vision("Not supported on this platform".into()))
    }
}

#[cfg(not(windows))]
pub fn capture_window(_hwnd: isize, _ctx: &WgcCaptureContext) -> Result<Image> {
    Err(SquireError::Vision("Not supported on this platform".into()))
}

#[cfg(not(windows))]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    Ok(Vec::new())
}

#[cfg(not(windows))]
pub fn get_process_name(_hwnd: isize) -> Result<String> {
    Ok(String::new())
}
