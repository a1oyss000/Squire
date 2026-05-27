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

fn to_gray(r: u8, g: u8, b: u8) -> f32 {
    0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
}

pub fn match_template(screen: &Image, template_path: &str, threshold: f64) -> Result<MatchResult> {
    let tmpl_img = image::open(template_path)
        .map_err(|e| squire_error::SquireError::Vision(format!("Failed to load template: {}", e)))?
        .to_rgba8();

    let tw = tmpl_img.width();
    let th = tmpl_img.height();
    let sw = screen.width;
    let sh = screen.height;

    if tw > sw || th > sh {
        return Err(squire_error::SquireError::Vision(
            "Template larger than screen".to_string(),
        ));
    }

    // Precompute template grayscale and stats
    let tmpl_gray: Vec<f32> = tmpl_img
        .pixels()
        .map(|p| to_gray(p[0], p[1], p[2]))
        .collect();

    let tmpl_len = (tw * th) as f32;
    let tmpl_mean = tmpl_gray.iter().sum::<f32>() / tmpl_len;
    let tmpl_centered: Vec<f32> = tmpl_gray.iter().map(|&v| v - tmpl_mean).collect();
    let tmpl_norm: f32 = tmpl_centered.iter().map(|&v| v * v).sum::<f32>().sqrt();

    if tmpl_norm < 1e-6 {
        return Err(squire_error::SquireError::Vision(
            "Template has zero variance".to_string(),
        ));
    }

    let screen_data = &screen.data;
    let cols = sw - tw;
    let rows = sh - th;

    let mut best_conf = f64::NEG_INFINITY;
    let mut best_x = 0u32;
    let mut best_y = 0u32;

    for sy in 0..=rows {
        for sx in 0..=cols {
            // Compute mean of screen patch
            let mut patch_sum = 0f32;
            for ty in 0..th {
                for tx in 0..tw {
                    let px = sx + tx;
                    let py = sy + ty;
                    let idx = ((py * sw + px) * 4) as usize;
                    patch_sum += to_gray(screen_data[idx], screen_data[idx + 1], screen_data[idx + 2]);
                }
            }
            let patch_mean = patch_sum / tmpl_len;

            // Compute NCC
            let mut cross = 0f32;
            let mut patch_sq = 0f32;
            for ty in 0..th {
                for tx in 0..tw {
                    let px = sx + tx;
                    let py = sy + ty;
                    let idx = ((py * sw + px) * 4) as usize;
                    let pv = to_gray(screen_data[idx], screen_data[idx + 1], screen_data[idx + 2]) - patch_mean;
                    let tv = tmpl_centered[(ty * tw + tx) as usize];
                    cross += pv * tv;
                    patch_sq += pv * pv;
                }
            }

            let patch_norm = patch_sq.sqrt();
            let denom = tmpl_norm * patch_norm;
            if denom < 1e-6 {
                continue;
            }
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
            center: Point {
                x: (best_x + tw / 2) as i32,
                y: (best_y + th / 2) as i32,
            },
            confidence: best_conf,
        })
    } else {
        Err(squire_error::SquireError::Vision(format!(
            "No match found: best confidence {:.3} < threshold {:.3}",
            best_conf, threshold
        )))
    }
}
