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

fn find_vcpkg_base() -> Option<PathBuf> {
    if let Ok(root) = env::var("VCPKG_ROOT") {
        let p = PathBuf::from(&root).join("installed").join("x64-windows");
        if p.exists() {
            return Some(p);
        }
    }
    let default = PathBuf::from(r"C:\Software\vcpkg\installed\x64-windows");
    if default.exists() {
        return Some(default);
    }
    let alt = PathBuf::from(r"C:\vcpkg\installed\x64-windows");
    if alt.exists() {
        return Some(alt);
    }
    None
}

fn main() {
    println!("cargo:rerun-if-env-changed=OPENCV_DIR");
    println!("cargo:rerun-if-env-changed=TESSERACT_DIR");
    println!("cargo:rerun-if-env-changed=VCPKG_ROOT");
    println!("cargo:rerun-if-changed=src/ffi/opencv_wrapper.cpp");

    let vcpkg_base = find_vcpkg_base().expect(
        "vcpkg not found. Set VCPKG_ROOT or install to C:\\Software\\vcpkg",
    );
    let lib_dir = vcpkg_base.join("lib");
    let include_dir = vcpkg_base.join("include");

    // Compile C++ wrapper
    let opencv_include = if include_dir.join("opencv4").exists() {
        include_dir.join("opencv4")
    } else {
        include_dir.clone()
    };

    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .file("src/ffi/opencv_wrapper.cpp")
        .include(&opencv_include)
        .compile("opencv_wrapper");

    // Link libraries
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // OpenCV
    let opencv_world = lib_dir.join("opencv_world4.lib");
    if opencv_world.exists() {
        println!("cargo:rustc-link-lib=dylib=opencv_world4");
    } else {
        println!("cargo:rustc-link-lib=dylib=opencv_core4");
        println!("cargo:rustc-link-lib=dylib=opencv_imgproc4");
    }

    // Tesseract
    let tess_name = find_lib_by_prefix(&lib_dir, "tesseract")
        .unwrap_or_else(|| "tesseract55".to_string());
    println!("cargo:rustc-link-lib=dylib={}", tess_name);

    // Leptonica
    let lept_name = find_lib_by_prefix(&lib_dir, "leptonica")
        .unwrap_or_else(|| "leptonica-1.84.1".to_string());
    println!("cargo:rustc-link-lib=dylib={}", lept_name);
}