use crate::Filter;
use cef::*;
use directories::ProjectDirs;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use url::Url;

#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "macos")]
pub use mac::setup_simple_application as setup_mac_application;
#[cfg(windows)]
mod windows;

thread_local! {
    static BROWSERS: RefCell<HashMap<i32, Browser>> = RefCell::new(HashMap::new());
}

fn string(value: &str) -> CefString {
    CefString::from(value)
}
fn web_host(address: &str) -> Option<String> {
    let u = Url::parse(address).ok()?;
    if !matches!(u.scheme(), "http" | "https") {
        return None;
    }
    u.host_str().map(str::to_owned)
}
fn top_url(browser: Option<&mut Browser>) -> String {
    browser
        .and_then(|b| b.main_frame())
        .map(|f| CefString::from(&f.url()).to_string())
        .unwrap_or_default()
}

struct Session {
    home: PathBuf,
    initial: String,
    private: bool,
    extension: Option<PathBuf>,
    filter: Option<Filter>,
    blocking_enabled: std::sync::atomic::AtomicBool,
    exceptions: Mutex<HashSet<String>>,
    _scratch: Option<tempfile::TempDir>,
}

impl Session {
    fn open() -> Result<Self, String> {
        let args: Vec<_> = std::env::args().skip(1).collect();
        let private = args.iter().any(|s| s == "--private");
        let option = |prefix: &str| args.iter().find_map(|s| s.strip_prefix(prefix));
        let initial = option("--url=").unwrap_or("about:blank").to_owned();
        if initial != "about:blank" && web_host(&initial).is_none() {
            return Err("The start URL must be HTTP or HTTPS".into());
        }
        let base = match std::env::var_os("BROWSER_TEST_HOME") {
            Some(path) => PathBuf::from(path).join("browser-test"),
            None => ProjectDirs::from("org", "BrowserPrototype", "DesktopBrowser")
                .ok_or("No profile directory")?
                .data_local_dir()
                .to_path_buf(),
        };
        let extension = option("--extension-lab=")
            .map(std::fs::canonicalize)
            .transpose()
            .map_err(|e| e.to_string())?;
        if private && extension.is_some() {
            return Err("Extensions require an ordinary lab profile".into());
        }
        if extension.as_ref().is_some_and(|p| {
            p.to_string_lossy().contains(',') || !p.join("manifest.json").is_file()
        }) {
            return Err("Expected an unpacked extension directory".into());
        }
        let scratch = if private {
            Some(
                tempfile::Builder::new()
                    .prefix("desktop-browser-private-")
                    .tempdir()
                    .map_err(|e| e.to_string())?,
            )
        } else {
            None
        };
        let home = scratch
            .as_ref()
            .map(|p| p.path().to_owned())
            .unwrap_or_else(|| {
                base.join(if extension.is_some() {
                    "extension-lab"
                } else {
                    "everyday"
                })
            });
        std::fs::create_dir_all(&home).map_err(|e| e.to_string())?;
        // CEF canonicalizes cache_path before comparing it to root_cache_path.
        // Keep both in the same form (notably /var -> /private/var on macOS),
        // otherwise an ordinary profile silently falls back to memory storage.
        let home = dunce::canonicalize(&home).map_err(|e| e.to_string())?;
        let filter = if extension.is_some() {
            None
        } else {
            Some(Filter::bundled()?)
        };
        Ok(Self {
            home,
            private,
            initial,
            extension,
            filter,
            blocking_enabled: std::sync::atomic::AtomicBool::new(true),
            exceptions: Default::default(),
            _scratch: scratch,
        })
    }

    // Chrome-created private/reference profiles must not consume the everyday
    // profile's session exceptions. Worker requests without a tab stay filtered.
    fn owns_exceptions(&self, b: Option<&Browser>) -> bool {
        if self.private {
            return true;
        }
        b.and_then(|b| b.host())
            .and_then(|h| h.request_context())
            .map(|ctx| {
                PathBuf::from(CefString::from(&ctx.cache_path()).to_string())
                    == self.home.join("Default")
            })
            .unwrap_or(false)
    }
    fn paused(&self, top: &str) -> bool {
        web_host(top).is_some_and(|host| self.exceptions.lock().unwrap().contains(&host))
    }
}

wrap_app! {
    struct Application { state: Arc<Session> }
    impl App {
        fn on_before_command_line_processing(&self, _kind: Option<&CefString>, line: Option<&mut CommandLine>) {
            let Some(line) = line else { return };
            for name in ["disable-background-networking", "disable-sync", "disable-domain-reliability", "no-first-run"] {
                line.append_switch(Some(&string(name)));
            }
            #[cfg(target_os = "linux")]
            if let Ok(backend) = std::env::var("XDG_SESSION_TYPE") && matches!(backend.as_str(),"wayland"|"x11") {
                line.append_switch_with_value(Some(&string("ozone-platform")),Some(&string(&backend)));
            }
            if let Some(dir) = &self.state.extension {
                // Canonical Windows paths may carry a verbatim prefix that Chromium refuses.
                let native = dunce::simplified(dir).to_string_lossy();
                for name in ["disable-extensions-except", "load-extension"] {
                    line.append_switch_with_value(Some(&string(name)), Some(&string(&native)));
                }
            } else {
                line.append_switch(Some(&string("disable-extensions")));
            }
        }
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(Process::new(self.state.clone()))
        }
    }
}

wrap_browser_process_handler! {
    struct Process { state: Arc<Session> }
    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let win = WindowInfo { runtime_style: RuntimeStyle::CHROME, ..Default::default() };
            #[cfg(windows)]
            let win = win.set_as_popup(Default::default(), "Desktop Browser");
            let ctx_settings = RequestContextSettings {
                cache_path: if self.state.private { CefString::default() } else {
                    string(&self.state.home.join("Default").to_string_lossy())
                },
                persist_session_cookies: i32::from(!self.state.private),
                ..Default::default()
            };
            let Some(mut ctx) = request_context_create_context(Some(&ctx_settings), self.default_request_context_handler().as_mut()) else {
                quit_message_loop(); return;
            };
            #[cfg(windows)]
            if !std::env::args().any(|v|v=="--reference-ui") {
                if !windows::open(self.state.clone(),ctx) { quit_message_loop(); }
                return;
            }
            if browser_host_create_browser(Some(&win), self.default_client().as_mut(),
                Some(&string(&self.state.initial)), Some(&BrowserSettings::default()), None, Some(&mut ctx)) == 0 { quit_message_loop(); }
        }
        fn default_client(&self) -> Option<Client> { Some(TabClient::new(self.state.clone())) }
        fn default_request_context_handler(&self) -> Option<RequestContextHandler> { Some(ProfileHooks::new(self.state.clone())) }
    }
}

wrap_client! {
    struct TabClient { state: Arc<Session> }
    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> { Some(Display::new()) }
        fn keyboard_handler(&self) -> Option<KeyboardHandler> { platform_keys() }
        fn life_span_handler(&self) -> Option<LifeSpanHandler> { Some(Lifetime::new()) }
        fn request_handler(&self) -> Option<RequestHandler> { Some(PageHooks::new(self.state.clone())) }
        fn load_handler(&self) -> Option<LoadHandler> { Some(Cosmetics::new(self.state.clone())) }
        fn context_menu_handler(&self) -> Option<ContextMenuHandler> { Some(Menu::new(self.state.clone())) }
        fn permission_handler(&self) -> Option<PermissionHandler> { Some(Permissions::new(self.state.private)) }
    }
}

wrap_life_span_handler! {
    struct Lifetime;
    impl LifeSpanHandler {
        fn on_after_created(&self, b: Option<&mut Browser>) {
            if let Some(b) = b { BROWSERS.with_borrow_mut(|tabs| tabs.insert(b.identifier(), b.clone())); }
        }
        fn on_before_close(&self, b: Option<&mut Browser>) {
            if let Some(b) = b {
                #[cfg(windows)]
                windows::removed(b.identifier());
                if BROWSERS.with_borrow_mut(|tabs| { tabs.remove(&b.identifier()); tabs.is_empty() }) {
                    #[cfg(windows)]
                    if windows::import_active(){return;}
                    quit_message_loop();
                }
            }
        }
    }
}

wrap_permission_handler! {
    struct Permissions { strict: bool }
    impl PermissionHandler {
        fn on_show_permission_prompt(&self, _b: Option<&mut Browser>, _id:u64, _origin:Option<&CefString>, _flags:u32, cb:Option<&mut PermissionPromptCallback>) -> i32 {
            if self.strict && let Some(cb) = cb { cb.cont(PermissionRequestResult::DENY); return 1; }
            0
        }
        fn on_request_media_access_permission(&self, _b:Option<&mut Browser>, _f:Option<&mut Frame>, _origin:Option<&CefString>, _flags:u32, cb:Option<&mut MediaAccessCallback>) -> i32 {
            if self.strict && let Some(cb) = cb { cb.cancel(); return 1; }
            0
        }
    }
}

fn resource_kind(t: ResourceType) -> &'static str {
    match t {
        ResourceType::MAIN_FRAME | ResourceType::NAVIGATION_PRELOAD_MAIN_FRAME => "document",
        ResourceType::SUB_FRAME | ResourceType::NAVIGATION_PRELOAD_SUB_FRAME => "subdocument",
        ResourceType::SCRIPT
        | ResourceType::WORKER
        | ResourceType::SHARED_WORKER
        | ResourceType::SERVICE_WORKER => "script",
        ResourceType::IMAGE | ResourceType::FAVICON => "image",
        ResourceType::STYLESHEET => "stylesheet",
        ResourceType::FONT_RESOURCE => "font",
        ResourceType::MEDIA => "media",
        ResourceType::OBJECT => "object",
        ResourceType::XHR => "xmlhttprequest",
        ResourceType::PING => "ping",
        _ => "other",
    }
}

fn intercept(
    state: &Arc<Session>,
    browser: Option<&mut Browser>,
    req: Option<&mut Request>,
    origin: Option<&CefString>,
) -> Option<ResourceRequestHandler> {
    state.filter.as_ref()?;
    if !state
        .blocking_enabled
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        return None;
    }
    let req = req?;
    let address = CefString::from(&req.url()).to_string();
    if !address.starts_with("https://") && !address.starts_with("http://") {
        return None;
    }
    let own = state.owns_exceptions(browser.as_deref());
    let top = if req.resource_type() == ResourceType::MAIN_FRAME {
        address.clone()
    } else {
        top_url(browser)
    };
    if own && state.paused(&top) {
        return None;
    }
    let source = origin
        .map(ToString::to_string)
        .filter(|s| web_host(s).is_some())
        .unwrap_or_else(|| if top.is_empty() { address } else { top });
    Some(Network::new(state.clone(), source))
}

wrap_request_handler! {
    struct PageHooks { state: Arc<Session> }
    impl RequestHandler {
        fn resource_request_handler(&self, b:Option<&mut Browser>, _f:Option<&mut Frame>, r:Option<&mut Request>, _nav:i32, _dl:i32, origin:Option<&CefString>, _default:Option<&mut i32>) -> Option<ResourceRequestHandler> {
            intercept(&self.state, b, r, origin)
        }
    }
}
wrap_request_context_handler! {
    struct ProfileHooks { state: Arc<Session> }
    impl RequestContextHandler {
        fn resource_request_handler(&self, b:Option<&mut Browser>, _f:Option<&mut Frame>, r:Option<&mut Request>, _nav:i32, _dl:i32, origin:Option<&CefString>, _default:Option<&mut i32>) -> Option<ResourceRequestHandler> {
            intercept(&self.state, b, r, origin)
        }
    }
}
wrap_resource_request_handler! {
    struct Network { state: Arc<Session>, source: String }
    impl ResourceRequestHandler {
        fn on_before_resource_load(&self, _b:Option<&mut Browser>, _f:Option<&mut Frame>, req:Option<&mut Request>, _cb:Option<&mut Callback>) -> ReturnValue {
            if self.state.blocking_enabled.load(std::sync::atomic::Ordering::Relaxed) && let (Some(filter), Some(r)) = (&self.state.filter, req)
                && filter.blocks(&CefString::from(&r.url()).to_string(), &self.source, resource_kind(r.resource_type()), &CefString::from(&r.method()).to_string()) {
                return ReturnValue::CANCEL;
            }
            ReturnValue::CONTINUE
        }
    }
}
wrap_load_handler! {
    struct Cosmetics { state: Arc<Session> }
    impl LoadHandler {
        fn on_loading_state_change(&self, _b:Option<&mut Browser>, _busy:i32, _back:i32, _forward:i32) {
            #[cfg(windows)]
            if let Some(b)=_b{windows::refresh(b);}
        }
        fn on_load_end(&self, b:Option<&mut Browser>, f:Option<&mut Frame>, _status:i32) {
            #[cfg(windows)]
            if f.as_ref().is_some_and(|f|f.is_main()!=0) && let Some(b)=b.as_ref(){windows::page_ready(b.identifier());}
            let (Some(filter), Some(frame)) = (&self.state.filter, f) else { return };
            if !self.state.blocking_enabled.load(std::sync::atomic::Ordering::Relaxed){return;}
            let url = CefString::from(&frame.url()).to_string();
            if web_host(&url).is_none() || (self.state.owns_exceptions(b.as_deref()) && self.state.paused(&top_url(b))) { return; }
            let css = filter.css(&url);
            if css.is_empty() { return; }
            let js = format!("(()=>{{const s=document.createElement('style');s.textContent={};document.documentElement.append(s)}})()", serde_json::to_string(&css).unwrap());
            frame.execute_java_script(Some(&string(&js)), None, 0);
        }
    }
}

const TOGGLE: i32 = 26_501;
wrap_context_menu_handler! {
    struct Menu { state: Arc<Session> }
    impl ContextMenuHandler {
        fn on_before_context_menu(&self,b:Option<&mut Browser>,_f:Option<&mut Frame>,_p:Option<&mut ContextMenuParams>,model:Option<&mut MenuModel>) {
            if !self.state.owns_exceptions(b.as_deref()) {return;}
            let top = top_url(b);
            if self.state.filter.is_some() && web_host(&top).is_some() && let Some(menu) = model {
                let label = if self.state.paused(&top) { "Resume blocking for this site" } else { "Pause blocking for this site" };
                menu.add_separator();
                menu.add_item(TOGGLE, Some(&string(label)));
            }
        }
        fn on_context_menu_command(&self,b:Option<&mut Browser>,_f:Option<&mut Frame>,_p:Option<&mut ContextMenuParams>,cmd:i32,_flags:EventFlags) -> i32 {
            if cmd != TOGGLE || !self.state.owns_exceptions(b.as_deref()) { return 0; }
            if let Some(host) = web_host(&top_url(b)) {
                let mut set = self.state.exceptions.lock().unwrap();
                if !set.remove(&host) && set.len() < 1024 { set.insert(host); }
            }
            1
        }
    }
}

pub fn run(args: &MainArgs, sandbox: *mut u8) -> i32 {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let code = execute_process(Some(args), None, sandbox);
    if code >= 0 {
        return code;
    }
    let state = match Session::open() {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let settings = Settings {
        root_cache_path: string(&state.home.to_string_lossy()),
        cache_path: if state.private {
            CefString::default()
        } else {
            string(&state.home.join("Default").to_string_lossy())
        },
        persist_session_cookies: i32::from(!state.private),
        command_line_args_disabled: 1,
        log_severity: LogSeverity::DISABLE,
        ..Default::default()
    };
    let mut app = Application::new(state);
    if initialize(Some(args), Some(&settings), Some(&mut app), sandbox) == 0 {
        return get_exit_code();
    }
    #[cfg(target_os = "macos")]
    let _delegate = mac::setup_simple_app_delegate();
    run_message_loop();
    drop(app);
    shutdown();
    #[cfg(windows)]
    windows::finish();
    0
}

#[cfg(windows)]
#[unsafe(no_mangle)]
unsafe extern "C" fn RunWinMain(
    instance: sys::HINSTANCE,
    _: *mut u16,
    _: i32,
    sandbox: *mut u8,
    _: *mut std::ffi::c_void,
) -> i32 {
    if sandbox.is_null() {
        return 2;
    }
    run(&MainArgs { instance }, sandbox)
}

wrap_display_handler! {
    struct Display;
    impl DisplayHandler {
        fn on_title_change(&self, _b: Option<&mut Browser>, _title: Option<&CefString>) {
            #[cfg(windows)]
            if let (Some(b),Some(t))=(_b,_title){windows::title(b,&t.to_string());}
        }
        fn on_address_change(&self,_b: Option<&mut Browser>,_f:Option<&mut Frame>,_url:Option<&CefString>) {
            #[cfg(windows)]
            if let Some(b)=_b{windows::refresh(b);}
        }
    }
}
#[cfg(not(windows))]
fn platform_keys() -> Option<KeyboardHandler> {
    None
}
#[cfg(windows)]
fn platform_keys() -> Option<KeyboardHandler> {
    Some(Keys::new())
}
#[cfg(windows)]
wrap_keyboard_handler! {
    struct Keys;
    impl KeyboardHandler {
        fn on_pre_key_event(&self, _b:Option<&mut Browser>,_e:Option<&KeyEvent>,_msg:Option<&mut sys::MSG>,_shortcut:Option<&mut i32>)->i32{
            #[cfg(windows)]
            if let (Some(b),Some(e))=(_b,_e){return windows::key(b,e);}
            0
        }
    }
}
