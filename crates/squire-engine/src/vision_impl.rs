use std::path::{Path, PathBuf};

use squire_error::{Result, SquireError};
use squire_vision::capture::{Image, WgcCaptureContext};

use crate::recognize::{MatchPosition, TextPosition, VisionProvider};

pub struct WgcVisionProvider {
    hwnd: isize,
    resources_dir: PathBuf,
    capture_ctx: WgcCaptureContext,
}

impl WgcVisionProvider {
    pub fn new(hwnd: isize, resources_dir: PathBuf) -> Result<Self> {
        let capture_ctx = WgcCaptureContext::new()?;
        Ok(Self { hwnd, resources_dir, capture_ctx })
    }

    /// Convert a point in client-area coordinates to absolute screen coordinates.
    ///
    /// Template matching returns positions relative to the captured image
    /// (client area), but `SendInput` uses absolute screen coordinates.
    /// `ClientToScreen` translates (0,0) in client area → screen position of
    /// the client area origin.
    fn client_to_screen(&self, cx: i32, cy: i32) -> (i32, i32) {
        #[cfg(windows)]
        {
            use windows::Win32::Foundation::{HWND, POINT};
            use windows::Win32::Graphics::Gdi::ClientToScreen;
            let hwnd_val = HWND(self.hwnd as *mut _);
            let mut pt = POINT { x: cx, y: cy };
            let _ = unsafe { ClientToScreen(hwnd_val, &mut pt) };
            (pt.x, pt.y)
        }
        #[cfg(not(windows))]
        {
            (cx, cy)
        }
    }

    /// Resolve a template path from the YAML to an absolute filesystem path.
    ///
    /// Template paths in YAML may be:
    /// - Absolute paths → used as-is
    /// - Prefixed with `resources/` → strip prefix, resolve against resources_dir
    /// - Other relative paths → resolve against resources_dir
    fn resolve_template_path(&self, raw: &str) -> PathBuf {
        let path = Path::new(raw);
        if path.is_absolute() {
            return path.to_path_buf();
        }
        // Strip optional `resources/` prefix — the YAML convention uses
        // `resources/tasks/templates/...` but our base is already the resources dir.
        let relative = raw
            .strip_prefix("resources/")
            .or_else(|| raw.strip_prefix("resources\\"))
            .unwrap_or(raw);
        self.resources_dir.join(relative)
    }
}

impl VisionProvider for WgcVisionProvider {
    fn capture(&self) -> Result<Image> {
        squire_vision::capture::capture_window(self.hwnd, &self.capture_ctx)
    }

    fn match_template(
        &self,
        image: &Image,
        template_path: &str,
        _roi: Option<[i32; 4]>,
        threshold: f64,
    ) -> Result<Option<MatchPosition>> {
        let resolved = self.resolve_template_path(template_path);
        let resolved_str = resolved.to_string_lossy();
        match squire_vision::matcher::match_template(image, &resolved_str, threshold) {
            Ok(result) => {
                let (sx, sy) = self.client_to_screen(result.center.x, result.center.y);
                Ok(Some(MatchPosition { x: sx, y: sy }))
            }
            Err(SquireError::MatchFailed { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn find_text(
        &self,
        image: &Image,
        pattern: &str,
        roi: Option<[i32; 4]>,
    ) -> Result<Option<TextPosition>> {
        let roi_arr = roi.as_ref().map(|r| r as &[i32; 4]);
        match squire_vision::ocr::find_text(image, pattern, roi_arr) {
            Ok(pt) => {
                let (sx, sy) = self.client_to_screen(pt.x, pt.y);
                Ok(Some(TextPosition { x: sx, y: sy }))
            }
            Err(SquireError::MatchFailed { .. }) => Ok(None),
            Err(SquireError::Ocr(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn check_color(
        &self,
        image: &Image,
        roi: [i32; 4],
        lower: [u8; 3],
        upper: [u8; 3],
        count: u32,
    ) -> Result<bool> {
        let x0 = roi[0].max(0) as u32;
        let y0 = roi[1].max(0) as u32;
        let x1 = (roi[0] + roi[2]).max(0) as u32;
        let y1 = (roi[1] + roi[3]).max(0) as u32;
        let x1 = x1.min(image.width);
        let y1 = y1.min(image.height);

        let mut matched: u32 = 0;
        for y in y0..y1 {
            for x in x0..x1 {
                let i = ((y * image.width + x) * 4) as usize;
                if i + 2 >= image.data.len() {
                    continue;
                }
                // BGRA layout: [B, G, R, A]
                let b = image.data[i];
                let g = image.data[i + 1];
                let r = image.data[i + 2];
                if r >= lower[0] && r <= upper[0]
                    && g >= lower[1] && g <= upper[1]
                    && b >= lower[2] && b <= upper[2]
                {
                    matched += 1;
                    if matched >= count {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(matched >= count)
    }
}
