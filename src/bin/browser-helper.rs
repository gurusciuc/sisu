#[cfg(target_os = "macos")]
fn main() {
    use cef::*;
    let args = args::Args::new();
    let mut sandbox = sandbox::Sandbox::new();
    sandbox.initialize(args.as_main_args());
    let loader = library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), true);
    assert!(loader.load());
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let code = execute_process(Some(args.as_main_args()), None, std::ptr::null_mut());
    drop(loader);
    drop(sandbox);
    std::process::exit(code);
}
#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("The helper is used by the macOS application bundle.");
}
