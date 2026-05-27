use crate::capture::Image;
use crate::matcher::{Point, Rect};
use squire_error::Result;
use std::path::Path;

#[cfg(windows)]
use windows::{
    Globalization::Language,
    Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap},
    Media::Ocr::OcrEngine as WinOcrEngine,
    Storage::Streams::Buffer,
    Win32::System::WinRT::IBufferByteAccess,
    core::{Interface, HSTRING},
};

pub struct OcrEngine {
    #[cfg(windows)]
    engine: WinOcrEngine,
    #[cfg(not(windows))]
    _private: (),
}

impl OcrEngine {
    pub fn new(lang: &str, _data_path: &Path) -> Result<Self> {
        #[cfg(windows)]
        {
            let engine = if lang.is_empty() {
                WinOcrEngine::TryCreateFromUserProfileLanguages()
                    .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?
            } else {
                let language = Language::CreateLanguage(
                    &HSTRING::from(lang),
                )
                .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;
                WinOcrEngine::TryCreateFromLanguage(&language)
                    .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?
            };
            Ok(Self { engine })
        }
        #[cfg(not(windows))]
        {
            tracing::warn!("OcrEngine::new: not supported on this platform");
            Ok(Self { _private: () })
        }
    }

    pub fn recognize_text(&self, image: &Image, region: Rect) -> Result<String> {
        #[cfg(windows)]
        {
            let bitmap = image_region_to_bitmap(image, region)?;
            let result = self
                .engine
                .RecognizeAsync(&bitmap)
                .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?
                .get()
                .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;
            let text = result
                .Text()
                .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;
            Ok(text.to_string())
        }
        #[cfg(not(windows))]
        {
            let _ = (image, region);
            Err(squire_error::SquireError::Ocr(
                "OCR not supported on this platform".to_string(),
            ))
        }
    }
}

impl Drop for OcrEngine {
    fn drop(&mut self) {}
}

pub fn find_text(screen: &Image, text: &str, region: Option<&[i32; 4]>) -> Result<Point> {
    #[cfg(windows)]
    {
        let engine = WinOcrEngine::TryCreateFromUserProfileLanguages()
            .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;

        let (bitmap, offset_x, offset_y) = match region {
            Some(r) => {
                let rect = Rect {
                    x: r[0],
                    y: r[1],
                    width: r[2] as u32,
                    height: r[3] as u32,
                };
                (image_region_to_bitmap(screen, rect)?, r[0], r[1])
            }
            None => {
                let rect = Rect {
                    x: 0,
                    y: 0,
                    width: screen.width,
                    height: screen.height,
                };
                (image_region_to_bitmap(screen, rect)?, 0, 0)
            }
        };

        let result = engine
            .RecognizeAsync(&bitmap)
            .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?
            .get()
            .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;

        let lines = result
            .Lines()
            .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;
        let count = lines
            .Size()
            .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;

        for i in 0..count {
            let line = lines
                .GetAt(i)
                .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;
            let words = line
                .Words()
                .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;
            let word_count = words
                .Size()
                .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;
            for j in 0..word_count {
                let word = words
                    .GetAt(j)
                    .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;
                let word_text = word
                    .Text()
                    .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;
                if word_text.to_string().to_lowercase().contains(&text.to_lowercase()) {
                    let br = word
                        .BoundingRect()
                        .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;
                    let cx = offset_x + br.X as i32 + (br.Width / 2.0) as i32;
                    let cy = offset_y + br.Y as i32 + (br.Height / 2.0) as i32;
                    return Ok(Point { x: cx, y: cy });
                }
            }
        }

        Err(squire_error::SquireError::Ocr(format!(
            "text '{}' not found",
            text
        )))
    }
    #[cfg(not(windows))]
    {
        let _ = (screen, text, region);
        Err(squire_error::SquireError::Ocr(
            "OCR not supported on this platform".to_string(),
        ))
    }
}

#[cfg(windows)]
fn image_region_to_bitmap(image: &Image, region: Rect) -> Result<SoftwareBitmap> {
    let x = region.x.max(0) as u32;
    let y = region.y.max(0) as u32;
    let w = region.width.min(image.width.saturating_sub(x));
    let h = region.height.min(image.height.saturating_sub(y));

    if w == 0 || h == 0 {
        return Err(squire_error::SquireError::Ocr(
            "empty region".to_string(),
        ));
    }

    // Convert RGBA -> BGRA and crop
    let mut bgra: Vec<u8> = Vec::with_capacity((w * h * 4) as usize);
    for row in y..(y + h) {
        for col in x..(x + w) {
            let idx = ((row * image.width + col) * 4) as usize;
            let r = image.data[idx];
            let g = image.data[idx + 1];
            let b = image.data[idx + 2];
            let a = image.data[idx + 3];
            bgra.push(b);
            bgra.push(g);
            bgra.push(r);
            bgra.push(a);
        }
    }

    let buf = Buffer::Create(bgra.len() as u32)
        .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;
    buf.SetLength(bgra.len() as u32)
        .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;

    // Write bytes via IBufferByteAccess
    unsafe {
        let byte_access: IBufferByteAccess = buf
            .cast()
            .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;
        let ptr = byte_access
            .Buffer()
            .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))?;
        std::ptr::copy_nonoverlapping(bgra.as_ptr(), ptr, bgra.len());
    }

    SoftwareBitmap::CreateCopyFromBuffer(
        &buf,
        BitmapPixelFormat::Bgra8,
        w as i32,
        h as i32,
    )
    .map_err(|e| squire_error::SquireError::Ocr(e.to_string()))
}
