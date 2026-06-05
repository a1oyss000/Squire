use squire_error::Result;
use squire_vision::capture::Image;

use crate::recognize::{MatchPosition, TextPosition, VisionProvider};

pub struct WgcVisionProvider {
    hwnd: isize,
}

impl WgcVisionProvider {
    pub fn new(hwnd: isize) -> Self {
        Self { hwnd }
    }
}

impl VisionProvider for WgcVisionProvider {
    fn capture(&self) -> Result<Image> {
        squire_vision::capture::capture_window(self.hwnd)
    }

    fn match_template(
        &self,
        image: &Image,
        template_path: &str,
        _roi: Option<[i32; 4]>,
        threshold: f64,
    ) -> Result<Option<MatchPosition>> {
        match squire_vision::matcher::match_template(image, template_path, threshold) {
            Ok(result) => Ok(Some(MatchPosition {
                x: result.center.x,
                y: result.center.y,
            })),
            Err(_) => Ok(None),
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
            Ok(pt) => Ok(Some(TextPosition { x: pt.x, y: pt.y })),
            Err(_) => Ok(None),
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
