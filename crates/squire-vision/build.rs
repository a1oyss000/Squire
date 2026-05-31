use std::env;
use std::fs;
use std::path::PathBuf;

fn find_lib_by_prefix(lib_dir: &PathBuf, prefix: &str) -> Option<String> {
    let entries = fs::read_dir(lib_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(prefix) && name.ends_with(".lib") {
            return Some(name.trim_end_matches(".lib").to_string());
        }
    }
    None
}

fn find_lib_dir(env_var: &str, _lib_name: &str) -> Option<PathBuf> {
    if let Ok(dir) = env::var(env_var) {
        let path = PathBuf::from(&dir).join("lib");
        if path.exists() {
            return Some(path);
        }
        let path = PathBuf::from(&dir);
        if path.exists() {
            return Some(path);
        }
    }

    if let Ok(vcpkg_root) = env::var("VCPKG_ROOT") {
        let path = PathBuf::from(&vcpkg_root)
            .join("installed")
            .join("x64-windows")
            .join("lib");
        if path.exists() {
            return Some(path);
        }
    }

    let default = PathBuf::from(r"C:\vcpkg\installed\x64-windows\lib");
    if default.exists() {
        return Some(default);
    }

    None
}

fn main() {
    println!("cargo:rerun-if-env-changed=OPENCV_DIR");
    println!("cargo:rerun-if-env-changed=TESSERACT_DIR");
    println!("cargo:rerun-if-env-changed=VCPKG_ROOT");

    if env::var("CARGO_FEATURE_FFI").is_err() {
        return;
    }

    let opencv_lib = match find_lib_dir("OPENCV_DIR", "opencv") {
        Some(p) => p,
        None => {
            println!(
                "cargo:warning=OpenCV library not found. Set OPENCV_DIR or VCPKG_ROOT, \
                 or install via: vcpkg install opencv4:x64-windows"
            );
            return;
        }
    };

    let tess_lib = match find_lib_dir("TESSERACT_DIR", "tesseract") {
        Some(p) => p,
        None => {
            println!(
                "cargo:warning=Tesseract library not found. Set TESSERACT_DIR or VCPKG_ROOT, \
                 or install via: vcpkg install tesseract:x64-windows"
            );
            return;
        }
    };

    println!("cargo:rustc-link-search=native={}", opencv_lib.display());
    println!("cargo:rustc-link-search=native={}", tess_lib.display());

    // OpenCV: try opencv_world first (single DLL), fall back to individual modules
    let opencv_world = opencv_lib.join("opencv_world4.lib");
    if opencv_world.exists() {
        println!("cargo:rustc-link-lib=dylib=opencv_world4");
    } else {
        println!("cargo:rustc-link-lib=dylib=opencv_core4");
        println!("cargo:rustc-link-lib=dylib=opencv_imgproc4");
        println!("cargo:rustc-link-lib=dylib=opencv_imgcodecs4");
    }

    // Tesseract: detect version from lib directory
    let tess_name = find_lib_by_prefix(&tess_lib, "tesseract")
        .unwrap_or_else(|| "tesseract55".to_string());
    println!("cargo:rustc-link-lib=dylib={}", tess_name);

    // Leptonica: detect version from lib directory
    let lept_name = find_lib_by_prefix(&tess_lib, "leptonica")
        .unwrap_or_else(|| "leptonica-1.84.1".to_string());
    println!("cargo:rustc-link-lib=dylib={}", lept_name);
}