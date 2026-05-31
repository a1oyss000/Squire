pub mod capture;
pub mod ffi;
pub mod matcher;
pub mod ocr;

#[cfg(windows)]
pub fn check_dependencies() -> squire_error::Result<()> {
    tracing::info!("Vision dependencies available (OpenCV + Tesseract via vcpkg)");
    Ok(())
}

#[cfg(not(windows))]
pub fn check_dependencies() -> squire_error::Result<()> {
    Ok(())
}
