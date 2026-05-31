use crate::capture::Image;
use squire_error::Result;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub image_path: String,
    pub threshold: f64,
    pub region: Option<Rect>,
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub center: Point,
    pub confidence: f64,
}

pub fn match_template(screen: &Image, template_path: &str, threshold: f64) -> Result<MatchResult> {
    use crate::ffi::opencv::*;
    use std::ffi::c_int;

    let tmpl_img = image::open(template_path)
        .map_err(|e| squire_error::SquireError::Vision(format!("Failed to load template: {}", e)))?
        .to_rgba8();

    let tw = tmpl_img.width() as c_int;
    let th = tmpl_img.height() as c_int;
    let sw = screen.width as c_int;
    let sh = screen.height as c_int;

    if tw > sw || th > sh {
        return Err(squire_error::SquireError::Vision(
            "Template larger than screen".into(),
        ));
    }

    // Generate binary mask from alpha channel: alpha > 128 → 255, else → 0
    let mask: Vec<u8> = tmpl_img.pixels()
        .map(|p| if p[3] > 128 { 255u8 } else { 0u8 })
        .collect();
    let has_transparent = mask.iter().any(|&v| v == 0);

    // Convert template to BGR for the C++ wrapper
    let tmpl_bgr: Vec<u8> = tmpl_img.pixels()
        .flat_map(|p| [p[2], p[1], p[0]])
        .collect();

    // Screen is already BGRA
    let screen_data = screen.data.as_ref();

    let mut max_val: f64 = 0.0;
    let mut max_x: c_int = 0;
    let mut max_y: c_int = 0;

    let mask_ptr = if has_transparent { mask.as_ptr() } else { std::ptr::null() };

    let ret = unsafe {
        squire_match_template_masked(
            screen_data.as_ptr(), sw, sh, 4,
            tmpl_bgr.as_ptr(), tw, th, 3,
            mask_ptr,
            CV_TM_CCOEFF_NORMED,
            &mut max_val, &mut max_x, &mut max_y,
        )
    };

    if ret != 0 {
        return Err(squire_error::SquireError::Vision(
            "matchTemplate failed".into(),
        ));
    }

    if max_val >= threshold {
        Ok(MatchResult {
            center: Point {
                x: max_x + tw / 2,
                y: max_y + th / 2,
            },
            confidence: max_val,
        })
    } else {
        Err(squire_error::SquireError::Vision(format!(
            "No match found: best confidence {:.3} < threshold {:.3}",
            max_val, threshold
        )))
    }
}