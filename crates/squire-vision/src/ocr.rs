use crate::capture::Image;
use crate::matcher::{Point, Rect};
use squire_error::Result;
use std::path::Path;

pub struct OcrEngine {
    #[cfg(windows)]
    _marker: std::marker::PhantomData<*mut ()>,
    #[cfg(not(windows))]
    _private: (),
}

impl OcrEngine {
    pub fn new(lang: &str, data_path: &Path) -> Result<Self> {
        #[cfg(windows)]
        {
            let _ = get_or_init_tess(lang, data_path)?;
            Ok(Self { _marker: std::marker::PhantomData })
        }
        #[cfg(not(windows))]
        {
            let _ = (lang, data_path);
            Ok(Self { _private: () })
        }
    }

    pub fn recognize_text(&self, image: &Image, region: Rect) -> Result<String> {
        #[cfg(windows)]
        { recognize_text_tess(image, region) }
        #[cfg(not(windows))]
        { let _ = (image, region); Err(squire_error::SquireError::Ocr("Not supported".into())) }
    }
}

impl Drop for OcrEngine { fn drop(&mut self) {} }

pub fn find_text(screen: &Image, text: &str, region: Option<&[i32; 4]>) -> Result<Point> {
    #[cfg(windows)]
    { find_text_tess(screen, text, region) }
    #[cfg(not(windows))]
    { let _ = (screen, text, region); Err(squire_error::SquireError::Ocr("Not supported".into())) }
}

// === Tesseract FFI backend ===
#[cfg(windows)]
use std::sync::{Mutex, OnceLock};

#[cfg(windows)]
static TESS_ENGINE: OnceLock<Mutex<crate::ffi::tesseract::OcrHandle>> = OnceLock::new();

#[cfg(windows)]
fn get_or_init_tess(lang: &str, data_path: &Path) -> Result<()> {
    use std::ffi::CString;
    use squire_error::SquireError;

    if TESS_ENGINE.get().is_some() {
        return Ok(());
    }

    let c_data = CString::new(data_path.to_string_lossy().as_ref())
        .map_err(|_| SquireError::Ocr("Invalid tessdata path".into()))?;
    let c_lang = CString::new(lang)
        .map_err(|_| SquireError::Ocr("Invalid language".into()))?;
    let handle = unsafe { crate::ffi::tesseract::OcrHandle::new(c_data.as_ptr(), c_lang.as_ptr()) }
        .ok_or_else(|| SquireError::Ocr(
            format!("Tesseract init failed: lang='{}' path='{}'", lang, data_path.display())
        ))?;
    let _ = TESS_ENGINE.set(Mutex::new(handle));
    Ok(())
}

#[cfg(windows)]
fn recognize_text_tess(image: &Image, region: Rect) -> Result<String> {
    use crate::ffi::tesseract::*;
    use std::ffi::CStr;
    let guard = TESS_ENGINE.get().unwrap().lock().unwrap();
    let (buf, w, h) = crop_bgra(image, region);
    unsafe {
        TessBaseAPISetImage(guard.as_ptr(), buf.as_ptr(), w, h, 4, w * 4);
        TessBaseAPISetSourceResolution(guard.as_ptr(), 72);
        TessBaseAPIRecognize(guard.as_ptr(), std::ptr::null_mut());
        let raw = TessBaseAPIGetUTF8Text(guard.as_ptr());
        if raw.is_null() { return Err(squire_error::SquireError::Ocr("No text".into())); }
        let text = CStr::from_ptr(raw).to_string_lossy().to_string();
        TessDeleteText(raw);
        Ok(text.trim().to_string())
    }
}

#[cfg(windows)]
fn find_text_tess(screen: &Image, text: &str, region: Option<&[i32; 4]>) -> Result<Point> {
    use crate::ffi::tesseract::*;
    use std::ffi::CStr;
    let guard = TESS_ENGINE.get().unwrap().lock().unwrap();
    let (buf, w, h, offset_x, offset_y) = match region {
        Some(r) => {
            let rect = Rect { x: r[0], y: r[1], width: r[2] as u32, height: r[3] as u32 };
            let (b, w, h) = crop_bgra(screen, rect);
            (b, w, h, r[0], r[1])
        }
        None => {
            let rect = Rect { x: 0, y: 0, width: screen.width, height: screen.height };
            let (b, w, h) = crop_bgra(screen, rect);
            (b, w, h, 0, 0)
        }
    };
    unsafe {
        TessBaseAPISetImage(guard.as_ptr(), buf.as_ptr(), w, h, 4, w * 4);
        TessBaseAPISetSourceResolution(guard.as_ptr(), 72);
        TessBaseAPIRecognize(guard.as_ptr(), std::ptr::null_mut());
        let iter = TessBaseAPIGetIterator(guard.as_ptr());
        if iter.is_null() {
            return Err(squire_error::SquireError::Ocr(format!("text '{}' not found", text)));
        }
        loop {
            let raw = TessResultIteratorGetUTF8Text(iter, RIL_WORD);
            if !raw.is_null() {
                let word = CStr::from_ptr(raw).to_string_lossy();
                let matches = word.to_lowercase().contains(&text.to_lowercase());
                TessDeleteText(raw);
                if matches {
                    let page_iter = TessResultIteratorGetPageIterator(iter);
                    let (mut l, mut t, mut r, mut b) = (0i32, 0i32, 0i32, 0i32);
                    TessPageIteratorBoundingBox(page_iter, RIL_WORD, &mut l, &mut t, &mut r, &mut b);
                    TessResultIteratorDelete(iter);
                    let cx = offset_x + (l + r) / 2;
                    let cy = offset_y + (t + b) / 2;
                    return Ok(Point { x: cx, y: cy });
                }
            }
            if TessResultIteratorNext(iter, RIL_WORD) == 0 { break; }
        }
        TessResultIteratorDelete(iter);
        Err(squire_error::SquireError::Ocr(format!("text '{}' not found", text)))
    }
}

#[cfg(windows)]
fn crop_bgra(image: &Image, region: Rect) -> (Vec<u8>, i32, i32) {
    let x = region.x.max(0) as u32;
    let y = region.y.max(0) as u32;
    let w = region.width.min(image.width.saturating_sub(x));
    let h = region.height.min(image.height.saturating_sub(y));
    let mut buf = Vec::with_capacity((w * h * 4) as usize);
    for row in y..(y + h) {
        let start = ((row * image.width + x) * 4) as usize;
        let end = start + (w * 4) as usize;
        buf.extend_from_slice(&image.data[start..end]);
    }
    (buf, w as i32, h as i32)
}
