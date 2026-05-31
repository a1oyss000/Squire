use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    tauri_build::build();
    patch_rc_manifest_for_admin();
    copy_vcpkg_dlls();
}

fn patch_rc_manifest_for_admin() {
    let out_dir = match env::var("OUT_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => return,
    };
    let rc_path = out_dir.join("resource.rc");
    if !rc_path.exists() {
        return;
    }
    let content = match fs::read_to_string(&rc_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    // Replace the inline manifest with one that includes requireAdministrator
    let old_manifest = r#"1 24
{
" <assembly xmlns=""urn:schemas-microsoft-com:asm.v1"" manifestVersion=""1.0""> "
" <dependency> "
" <dependentAssembly> "
" <assemblyIdentity "
" type=""win32"" "
" name=""Microsoft.Windows.Common-Controls"" "
" version=""6.0.0.0"" "
" processorArchitecture=""*"" "
" publicKeyToken=""6595b64144ccf1df"" "
" language=""*"" "
" /> "
" </dependentAssembly> "
" </dependency> "
" </assembly> "
}"#;
    let new_manifest = r#"1 24
{
" <assembly xmlns=""urn:schemas-microsoft-com:asm.v1"" manifestVersion=""1.0""> "
" <trustInfo xmlns=""urn:schemas-microsoft-com:asm.v3""> "
" <security> "
" <requestedPrivileges> "
" <requestedExecutionLevel level=""requireAdministrator"" uiAccess=""false""/> "
" </requestedPrivileges> "
" </security> "
" </trustInfo> "
" <dependency> "
" <dependentAssembly> "
" <assemblyIdentity "
" type=""win32"" "
" name=""Microsoft.Windows.Common-Controls"" "
" version=""6.0.0.0"" "
" processorArchitecture=""*"" "
" publicKeyToken=""6595b64144ccf1df"" "
" language=""*"" "
" /> "
" </dependentAssembly> "
" </dependency> "
" </assembly> "
}"#;
    let patched = content.replace(old_manifest, new_manifest);
    let _ = fs::write(&rc_path, patched);
}

fn copy_vcpkg_dlls() {
    let out_dir = match env::var("OUT_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => return,
    };
    // OUT_DIR is like target/debug/build/squire-app-xxx/out
    // We need target/debug/
    let target_dir = out_dir
        .ancestors()
        .nth(3)
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| out_dir.clone());

    let vcpkg_bin = env::var("VCPKG_ROOT")
        .map(|r| PathBuf::from(r).join("installed/x64-windows/bin"))
        .unwrap_or_else(|_| PathBuf::from(r"C:\Software\vcpkg\installed\x64-windows\bin"));

    if !vcpkg_bin.exists() {
        return;
    }

    let entries = match fs::read_dir(&vcpkg_bin) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".dll") {
            let dest = target_dir.join(&name);
            if !dest.exists() {
                let _ = fs::copy(entry.path(), &dest);
            }
        }
    }
}
