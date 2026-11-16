#[cfg(any(target_os = "macos", target_os = "linux"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = std::path::PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("Expected release target directory")?,
    );
    let out = std::path::PathBuf::from(
        std::env::args_os()
            .nth(2)
            .ok_or("Expected output directory")?,
    );
    std::fs::create_dir_all(&out)?;
    #[cfg(target_os = "macos")]
    cef::build_util::mac::bundle(
        &out,
        &target,
        "browser-shell",
        "browser-helper",
        Some("resources".into()),
        cef::build_util::mac::BundleInfo::new(
            "Desktop Browser",
            "org.browserprototype.desktop",
            "Desktop Browser",
            "English",
            env!("CARGO_PKG_VERSION").parse()?,
        ),
    )?;
    #[cfg(target_os = "linux")]
    cef::build_util::linux::bundle(&out, &target, "browser-shell")?;
    Ok(())
}
#[cfg(windows)]
fn main() {
    eprintln!("Use tools/build-windows.ps1");
}
