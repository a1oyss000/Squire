pub mod capture;
pub mod ffi;
pub mod matcher;
pub mod ocr;

#[cfg(all(feature = "ffi", windows))]
pub fn check_dependencies() -> squire_error::Result<()> {
    use squire_error::SquireError;
    use windows::core::HSTRING;
    use windows::Win32::System::LibraryLoader::LoadLibraryW;

    let dlls = &["opencv_world4100.dll", "tesseract53.dll"];
    for name in dlls {
        let wide = HSTRING::from(*name);
        if unsafe { LoadLibraryW(&wide) }.is_err() {
            return Err(SquireError::DllNotFound(format!(
                "{} not found in PATH. Install via vcpkg or place DLLs alongside the executable.",
                name
            )));
        }
    }
    tracing::info!("Vision FFI dependencies verified");
    Ok(())
}

#[cfg(not(all(feature = "ffi", windows)))]
pub fn check_dependencies() -> squire_error::Result<()> {
    tracing::info!("Vision dependencies check passed (no FFI)");
    Ok(())
}
