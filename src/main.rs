#[cfg(windows)]
fn main() {
    eprintln!("Use the sandbox bootstrap desktop_browser.exe from the staged directory.");
    std::process::exit(2);
}
#[cfg(not(windows))]
fn main() {
    let args = cef::args::Args::new();
    #[cfg(target_os = "macos")]
    let _loader = {
        let loader =
            cef::library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), false);
        assert!(loader.load());
        desktop_browser::desktop::setup_mac_application();
        loader
    };
    let code = desktop_browser::desktop::run(args.as_main_args(), std::ptr::null_mut());
    std::process::exit(code);
}
