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

#[cfg(feature = "ffi")]
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

    if tw > sw || th > sh {
        return Err(squire_error::SquireError::Vision(
            "Template larger than screen".into(),
        ));
    }

    unsafe {
        // Wrap template RGB data as CvMat, convert to grayscale
        let mut tmpl_data = tmpl_img.into_raw();
        let tmpl_mat = HeaderCvMat::new(
            th, tw, CV_8UC3,
            tmpl_data.as_mut_ptr() as *mut std::ffi::c_void,
            tw * 3,
        ).ok_or_else(|| squire_error::SquireError::Vision("Failed to create tmpl mat".into()))?;

        let mut tmpl_gray = OwnedCvMat::from_raw(cvCreateMat(th, tw, CV_8UC1))
            .ok_or_else(|| squire_error::SquireError::Vision("Failed to allocate tmpl_gray".into()))?;
        cvCvtColor(tmpl_mat.as_ptr(), tmpl_gray.as_mut_ptr(), COLOR_BGR2GRAY);

        // Wrap screen BGRA buffer as CvMat, convert to grayscale
        let mut screen_data = screen.data.as_ref().clone();
        let screen_mat = HeaderCvMat::new(
            sh, sw, CV_8UC4,
            screen_data.as_mut_ptr() as *mut std::ffi::c_void,
            sw * 4,
        ).ok_or_else(|| squire_error::SquireError::Vision("Failed to create screen mat".into()))?;

        let mut screen_gray = OwnedCvMat::from_raw(cvCreateMat(sh, sw, CV_8UC1))
            .ok_or_else(|| squire_error::SquireError::Vision("Failed to allocate screen_gray".into()))?;
        cvCvtColor(screen_mat.as_ptr(), screen_gray.as_mut_ptr(), COLOR_BGRA2GRAY);

        // Allocate result matrix
        let result_rows = sh - th + 1;
        let result_cols = sw - tw + 1;
        let mut result_mat = OwnedCvMat::from_raw(cvCreateMat(result_rows, result_cols, CV_32FC1))
            .ok_or_else(|| squire_error::SquireError::Vision("Failed to allocate result mat".into()))?;

        cvMatchTemplate(screen_gray.as_ptr(), tmpl_gray.as_ptr(), result_mat.as_mut_ptr(), CV_TM_CCOEFF_NORMED);

        let mut max_val: f64 = 0.0;
        let mut max_loc = CvPoint { x: 0, y: 0 };
        cvMinMaxLoc(
            result_mat.as_ptr(),
            std::ptr::null_mut(),
            &mut max_val,
            std::ptr::null_mut(),
            &mut max_loc,
            std::ptr::null(),
        );

        if max_val >= threshold {
            Ok(MatchResult {
                center: Point {
                    x: max_loc.x + tw / 2,
                    y: max_loc.y + th / 2,
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
}

#[cfg(not(feature = "ffi"))]
pub fn match_template(screen: &Image, template_path: &str, threshold: f64) -> Result<MatchResult> {
    fn to_gray(r: u8, g: u8, b: u8) -> f32 {
        0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
    }

    let tmpl_img = image::open(template_path)
        .map_err(|e| squire_error::SquireError::Vision(format!("Failed to load template: {}", e)))?
        .to_rgba8();

    let tw = tmpl_img.width();
    let th = tmpl_img.height();
    let sw = screen.width;
    let sh = screen.height;

    if tw > sw || th > sh {
        return Err(squire_error::SquireError::Vision("Template larger than screen".into()));
    }

    let tmpl_gray: Vec<f32> = tmpl_img.pixels().map(|p| to_gray(p[0], p[1], p[2])).collect();
    let tmpl_len = (tw * th) as f32;
    let tmpl_mean = tmpl_gray.iter().sum::<f32>() / tmpl_len;
    let tmpl_centered: Vec<f32> = tmpl_gray.iter().map(|&v| v - tmpl_mean).collect();
    let tmpl_norm: f32 = tmpl_centered.iter().map(|&v| v * v).sum::<f32>().sqrt();

    if tmpl_norm < 1e-6 {
        return Err(squire_error::SquireError::Vision("Template has zero variance".into()));
    }

    let screen_data = &screen.data;
    let mut best_conf = f64::NEG_INFINITY;
    let mut best_x = 0u32;
    let mut best_y = 0u32;

    for sy in 0..=(sh - th) {
        for sx in 0..=(sw - tw) {
            let mut patch_sum = 0f32;
            for ty in 0..th {
                for tx in 0..tw {
                    let idx = (((sy + ty) * sw + (sx + tx)) * 4) as usize;
                    patch_sum += to_gray(screen_data[idx + 2], screen_data[idx + 1], screen_data[idx]);
                }
            }
            let patch_mean = patch_sum / tmpl_len;
            let mut cross = 0f32;
            let mut patch_sq = 0f32;
            for ty in 0..th {
                for tx in 0..tw {
                    let idx = (((sy + ty) * sw + (sx + tx)) * 4) as usize;
                    let pv = to_gray(screen_data[idx + 2], screen_data[idx + 1], screen_data[idx]) - patch_mean;
                    let tv = tmpl_centered[(ty * tw + tx) as usize];
                    cross += pv * tv;
                    patch_sq += pv * pv;
                }
            }
            let denom = tmpl_norm * patch_sq.sqrt();
            if denom < 1e-6 { continue; }
            let ncc = (cross / denom) as f64;
            if ncc > best_conf {
                best_conf = ncc;
                best_x = sx;
                best_y = sy;
            }
        }
    }

    if best_conf >= threshold {
        Ok(MatchResult {
            center: Point { x: (best_x + tw / 2) as i32, y: (best_y + th / 2) as i32 },
            confidence: best_conf,
        })
    } else {
        Err(squire_error::SquireError::Vision(format!(
            "No match found: best confidence {:.3} < threshold {:.3}", best_conf, threshold
        )))
    }
}