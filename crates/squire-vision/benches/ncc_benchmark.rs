use squire_vision::capture::Image;
use squire_vision::matcher::match_template;
use std::sync::Arc;
use std::time::Instant;

fn create_synthetic_screen(width: u32, height: u32) -> Image {
    // BGRA buffer with a gradient pattern
    let mut data = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            data[idx] = (x % 256) as u8;     // B
            data[idx + 1] = (y % 256) as u8; // G
            data[idx + 2] = ((x + y) % 256) as u8; // R
            data[idx + 3] = 255;              // A
        }
    }
    Image { width, height, data: Arc::new(data) }
}

fn create_roi_screen(full: &Image, x: u32, y: u32, w: u32, h: u32) -> Image {
    let mut data = vec![0u8; (w * h * 4) as usize];
    for row in 0..h {
        for col in 0..w {
            let src_idx = (((y + row) * full.width + (x + col)) * 4) as usize;
            let dst_idx = ((row * w + col) * 4) as usize;
            data[dst_idx..dst_idx + 4].copy_from_slice(&full.data[src_idx..src_idx + 4]);
        }
    }
    Image { width: w, height: h, data: Arc::new(data) }
}

fn main() {
    // Create a 1920x1080 synthetic screen
    let screen = create_synthetic_screen(1920, 1080);

    // Save a 64x64 region as template (at position 500, 300)
    let tmpl_x = 500u32;
    let tmpl_y = 300u32;
    let tmpl_w = 64u32;
    let tmpl_h = 64u32;

    // Extract template pixels and save as PNG (RGBA for image crate)
    let mut tmpl_rgba = vec![0u8; (tmpl_w * tmpl_h * 4) as usize];
    for row in 0..tmpl_h {
        for col in 0..tmpl_w {
            let src_idx = (((tmpl_y + row) * screen.width + (tmpl_x + col)) * 4) as usize;
            let dst_idx = ((row * tmpl_w + col) * 4) as usize;
            // Convert BGRA -> RGBA for the template file
            tmpl_rgba[dst_idx] = screen.data[src_idx + 2];     // R
            tmpl_rgba[dst_idx + 1] = screen.data[src_idx + 1]; // G
            tmpl_rgba[dst_idx + 2] = screen.data[src_idx];     // B
            tmpl_rgba[dst_idx + 3] = screen.data[src_idx + 3]; // A
        }
    }

    let tmpl_path = std::env::temp_dir().join("squire_bench_template.png");
    image::save_buffer(
        &tmpl_path, &tmpl_rgba, tmpl_w, tmpl_h, image::ColorType::Rgba8,
    ).expect("save template");
    let tmpl_path_str = tmpl_path.to_string_lossy().to_string();

    println!("=== NCC Benchmark ===");
    println!("Screen: {}x{}, Template: {}x{}", screen.width, screen.height, tmpl_w, tmpl_h);

    let start = Instant::now();
    let result = match_template(&screen, &tmpl_path_str, 0.8);
    let full_elapsed = start.elapsed();
    match &result {
        Ok(r) => println!("Full: {:?} | ({},{}) conf={:.4}", full_elapsed, r.center.x, r.center.y, r.confidence),
        Err(e) => println!("Full: {:?} | ERR: {}", full_elapsed, e),
    }

    let roi = create_roi_screen(&screen, tmpl_x.saturating_sub(68), tmpl_y.saturating_sub(68), 200, 200);
    let start = Instant::now();
    let r2 = match_template(&roi, &tmpl_path_str, 0.8);
    let roi_elapsed = start.elapsed();
    match &r2 {
        Ok(r) => println!("ROI 200x200: {:?} | ({},{}) conf={:.4}", roi_elapsed, r.center.x, r.center.y, r.confidence),
        Err(e) => println!("ROI 200x200: {:?} | ERR: {}", roi_elapsed, e),
    }

    let roi2 = create_roi_screen(&screen, tmpl_x.saturating_sub(32), tmpl_y.saturating_sub(32), 128, 128);
    let start = Instant::now();
    let r3 = match_template(&roi2, &tmpl_path_str, 0.8);
    let roi2_elapsed = start.elapsed();
    match &r3 {
        Ok(r) => println!("ROI 128x128: {:?} | ({},{}) conf={:.4}", roi2_elapsed, r.center.x, r.center.y, r.confidence),
        Err(e) => println!("ROI 128x128: {:?} | ERR: {}", roi2_elapsed, e),
    }

    println!("\n=== Decision Gate (50ms threshold) ===");
    if roi_elapsed.as_millis() < 50 {
        println!("PASS: ROI NCC fast enough ({:?}). Skip OpenCV.", roi_elapsed);
    } else {
        println!("FAIL: ROI NCC too slow ({:?}). OpenCV required.", roi_elapsed);
    }
    let _ = std::fs::remove_file(&tmpl_path);
}