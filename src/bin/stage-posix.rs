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
    {
        let destination = out.join("Sisu Preview.app");
        if destination.exists() || out.join("browser-shell.app").exists() {
            return Err("Output app already exists; choose a fresh staging directory".into());
        }
        let app = cef::build_util::mac::bundle(
            &out,
            &target,
            "browser-shell",
            "browser-helper",
            // The native menu is built in Rust; packaging only needs Command Line
            // Tools, not Xcode's ibtool or the old cefsimple XIB.
            None,
            cef::build_util::mac::BundleInfo::new(
                "Sisu Preview",
                "org.browserprototype.desktop",
                "Sisu Preview",
                "English",
                env!("CARGO_PKG_VERSION").parse()?,
            ),
        )?;
        let numeric_version = format!(
            "{}.{}.{}",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH")
        );
        let main_info = app.join("Contents/Info.plist");
        let mut main = plist::Value::from_file(&main_info)?;
        let dict = main.as_dictionary_mut().ok_or("Expected app dictionary")?;
        dict.insert(
            "CFBundleShortVersionString".into(),
            numeric_version.clone().into(),
        );
        dict.insert("CFBundleVersion".into(), numeric_version.clone().into());
        dict.insert("SisuVersion".into(), env!("CARGO_PKG_VERSION").into());
        main.to_file_xml(main_info)?;
        // cef-rs currently assigns the main bundle ID to every helper. Give each
        // helper its own identity while retaining CEF's required executable names.
        for (name, suffix) in [
            ("Helper", "helper"),
            ("Helper (GPU)", "helper.gpu"),
            ("Helper (Renderer)", "helper.renderer"),
            ("Helper (Plugin)", "helper.plugin"),
            ("Helper (Alerts)", "helper.alerts"),
        ] {
            let path = app.join(format!(
                "Contents/Frameworks/browser-shell {name}.app/Contents/Info.plist"
            ));
            let mut info = plist::Value::from_file(&path)?;
            let dict = info
                .as_dictionary_mut()
                .ok_or("Expected helper dictionary")?;
            dict.insert(
                "CFBundleShortVersionString".into(),
                numeric_version.clone().into(),
            );
            dict.insert("CFBundleVersion".into(), numeric_version.clone().into());
            dict.insert("SisuVersion".into(), env!("CARGO_PKG_VERSION").into());
            dict.insert(
                "CFBundleIdentifier".into(),
                format!("org.browserprototype.desktop.{suffix}").into(),
            );
            for key in ["CFBundleName", "CFBundleDisplayName"] {
                dict.insert(key.into(), format!("Sisu Preview {name}").into());
            }
            info.to_file_xml(path)?;
        }
        let resources = app.join("Contents/Resources");
        for name in ["LICENSE", "NOTICE", "THIRD_PARTY.md"] {
            std::fs::copy(name, resources.join(name))?;
        }
        copy_notices(std::path::Path::new("notices"), &resources.join("notices"))?;
        std::fs::rename(app, destination)?;
    }
    #[cfg(target_os = "linux")]
    cef::build_util::linux::bundle(&out, &target, "browser-shell")?;
    Ok(())
}
#[cfg(target_os = "macos")]
fn copy_notices(source: &std::path::Path, destination: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            std::fs::copy(entry.path(), destination.join(entry.file_name()))?;
        }
    }
    Ok(())
}
#[cfg(windows)]
fn main() {
    eprintln!("Use tools/build-windows.ps1");
}
