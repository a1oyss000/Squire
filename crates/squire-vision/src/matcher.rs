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
        .to_rgb8();

    let tw = tmpl_img.width() as c_int;
    let th = tmpl_img.height() as c_int;
    let sw = screen.width as c_int;
    let sh = screen.height as c_int;

    tracing::debug!(
        "match_template: screen={}x{} ({} bytes), template={}x{}, path={}",
        sw, sh, screen.data.len(), tw, th, template_path
    );

    // Check if screen data is all zeros (common with DirectX games and BitBlt)
    let non_zero = screen.data.iter().take(1000).filter(|&&b| b != 0).count();
    if non_zero == 0 {
        tracing::warn!("match_template: screen data appears to be all zeros (capture may have failed)");
    }

    if tw > sw || th > sh {
        return Err(squire_error::SquireError::Vision(
            "Template larger than screen".into(),
        ));
    }

    // Convert template to BGR for the C++ wrapper
    let tmpl_bgr: Vec<u8> = tmpl_img.pixels()
        .flat_map(|p| [p[2], p[1], p[0]])
        .collect();

    // Convert screen from BGRA to BGR (strip alpha)
    let screen_bgr: Vec<u8> = screen.data.chunks_exact(4)
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect();

    let mut max_val: f64 = 0.0;
    let mut max_x: c_int = 0;
    let mut max_y: c_int = 0;

    let ret = unsafe {
        squire_match_template_masked(
            screen_bgr.as_ptr(), sw, sh, 3,
            tmpl_bgr.as_ptr(), tw, th, 3,
            std::ptr::null(),
            CV_TM_CCOEFF_NORMED,
            &mut max_val, &mut max_x, &mut max_y,
        )
    };

    if ret != 0 {
        return Err(squire_error::SquireError::Vision(
            "matchTemplate failed".into(),
        ));
    }

    tracing::debug!("match_template: result max_val={:.4}, pos=({},{})", max_val, max_x, max_y);

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