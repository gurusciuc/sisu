mod library;
use super::*;
use std::{
    ffi::{OsString, c_void},
    io::Write,
    os::windows::ffi::OsStringExt,
    sync::OnceLock,
};

unsafe extern "C" {
    fn shell_open(vertical: bool, width: i32, private: bool, extension: bool) -> *mut c_void;
    fn shell_add(id: i32, page: *mut c_void, local_home: bool);
    fn shell_ready(id: i32);
    fn shell_remove(id: i32);
    fn shell_select(id: i32);
    fn shell_title(id: i32, title: *const u16);
    fn shell_status(url: *const u16, back: bool, forward: bool, busy: bool, enabled: bool);
    fn shell_layout(vertical: bool);
    fn shell_post(command: i32);
    fn shell_destroy();
    fn shell_sync_settings(id: i32, visible: bool, prefs: NativePrefs);
    fn shell_focus_settings(id: i32);
    fn shell_width(width: i32);
    fn shell_suggestions(query: *const u16, rows: *const *const u16, count: i32);
    fn shell_library(rows: *const *const u16, count: i32, selected: *mut i32) -> i32;
    fn shell_find(buffer: *mut u16) -> i32;
    fn shell_pick_file(buffer: *mut u16, size: i32) -> bool;
    fn shell_notice(title: *const u16, message: *const u16);
    fn shell_prepare_page(width: *mut i32, height: *mut i32) -> *mut c_void;
    fn shell_discard_stage(stage: *mut c_void);
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NativePrefs {
    vertical: i32,
    width: i32,
    google: i32,
    reopen: i32,
}
fn notice(title: &str, message: &str) {
    unsafe { shell_notice(wide(title).as_ptr(), wide(message).as_ptr()) }
}
struct Window {
    session: Arc<Session>,
    context: RequestContext,
    google: bool,
    reopen: bool,
    last: String,
    find: String,
    settings_tabs: HashSet<i32>,
    recent: Vec<String>,
    bookmarks: Vec<crate::bookmarks::Bookmark>,
    order: Vec<i32>,
    current: Option<i32>,
    vertical: bool,
    width: i32,
}
thread_local! { static WINDOW: RefCell<Option<Window>> = const { RefCell::new(None) }; }

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}
fn home(google: bool) -> &'static str {
    static DDG: OnceLock<String> = OnceLock::new();
    static GOOGLE: OnceLock<String> = OnceLock::new();
    (if google { &GOOGLE } else { &DDG }).get_or_init(|| {
        let html = include_str!("newtab.html");
        let html = if google {
            html.replace("https://duckduckgo.com", "https://www.google.com")
                .replace("www.google.com/\" method", "www.google.com/search\" method")
                .replace("DuckDuckGo", "Google")
        } else {
            html.to_string()
        };
        format!(
            "data:text/html;charset=utf-8,{}",
            url::form_urlencoded::byte_serialize(html.as_bytes())
                .collect::<String>()
                .replace('+', "%20")
        )
    })
}
// Only a tab opened by browser chrome may mount native preferences. The document
// is a history entry. It contains no script, privileged links, or renderer bridge.
const SETTINGS_DOCUMENT: &str = "data:text/html,%3Cmeta%20charset=utf-8%3E%3Ctitle%3ESettings%3C/title%3E%3Cstyle%3Ehtml%7Bbackground:%23101113%7D%3C/style%3E";
const SETTINGS_ADDRESS: &str = "browser://settings";
fn is_settings_input(input: &str) -> bool {
    matches!(
        input
            .trim()
            .trim_end_matches('/')
            .to_ascii_lowercase()
            .as_str(),
        "#settings" | SETTINGS_ADDRESS
    )
}
impl Window {
    fn prefs(&self) -> NativePrefs {
        NativePrefs {
            vertical: self.vertical.into(),
            width: self.width,
            google: self.google.into(),
            reopen: self.reopen.into(),
        }
    }
}
fn open_settings() {
    let candidates = WINDOW.with_borrow(|slot| {
        slot.as_ref()
            .map(|w| w.settings_tabs.clone())
            .unwrap_or_default()
    });
    let existing = BROWSERS.with_borrow(|bs| {
        candidates
            .iter()
            .find(|id| {
                bs.get(id).and_then(|b| b.main_frame()).is_some_and(|f| {
                    let u = CefString::from(&f.url()).to_string();
                    u.is_empty() || u == SETTINGS_DOCUMENT
                })
            })
            .copied()
    });
    if let Some(id) = existing {
        select(id);
    } else {
        new_page(SETTINGS_DOCUMENT);
    }
}
fn settings_document(id: i32, address: &str) -> bool {
    address == SETTINGS_DOCUMENT
        && WINDOW.with_borrow(|slot| slot.as_ref().is_some_and(|w| w.settings_tabs.contains(&id)))
}
fn display_url(url: &str) -> &str {
    if url == "about:blank" || url == home(false) || url == home(true) {
        ""
    } else {
        url
    }
}
pub fn open(state: Arc<Session>, context: RequestContext) -> bool {
    let pref = state.home.join("layout.json");
    let prefs = if !state.private {
        std::fs::read(&pref)
            .ok()
            .filter(|b| b.len() <= 512 * 1024)
            .and_then(|buf| serde_json::from_slice::<serde_json::Value>(&buf).ok())
    } else {
        None
    };
    let vertical = prefs.as_ref().and_then(|v| v["placement"].as_str()) == Some("left");
    let width = prefs
        .as_ref()
        .and_then(|v| v["width"].as_i64())
        .unwrap_or(216)
        .clamp(92, 350) as i32;
    let handle = unsafe { shell_open(vertical, width, state.private, state.extension.is_some()) };
    if handle.is_null() {
        return false;
    }
    let google = prefs
        .as_ref()
        .and_then(|v| v["google"].as_bool())
        .unwrap_or(false);
    let reopen = prefs
        .as_ref()
        .and_then(|v| v["reopen"].as_bool())
        .unwrap_or(false)
        && !state.private;
    let last = prefs
        .as_ref()
        .and_then(|v| v["last"].as_str())
        .filter(|u| web_host(u).is_some())
        .unwrap_or_default()
        .to_owned();
    let initial = if state.initial == "about:blank" && reopen && !last.is_empty() {
        last.clone()
    } else {
        state.initial.clone()
    };
    let recent = prefs
        .as_ref()
        .and_then(|v| v["recent"].as_array())
        .map(|rows| {
            rows.iter()
                .take(64)
                .filter_map(|r| r.as_str())
                .filter(|u| u.len() <= 2048 && web_host(u).is_some())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let bookmarks = if state.private {
        vec![]
    } else {
        library::library(&state).unwrap_or_default()
    };
    WINDOW.with_borrow_mut(|slot| {
        *slot = Some(Window {
            session: state,
            context,
            google,
            reopen,
            last,
            find: String::new(),
            settings_tabs: HashSet::new(),
            recent,
            bookmarks,
            order: Vec::new(),
            current: None,
            vertical,
            width,
        })
    });
    new_page(&initial)
}

fn new_page(url: &str) -> bool {
    let setup = WINDOW.with_borrow(|w| {
        w.as_ref()
            .map(|s| (s.context.clone(), s.session.clone(), s.google))
    });
    let Some((mut ctx, state, google)) = setup else {
        return false;
    };
    let (mut width, mut height) = (0, 0);
    let stage = unsafe { shell_prepare_page(&mut width, &mut height) };
    if stage.is_null() {
        return false;
    }
    let parent = sys::HWND(stage.cast());
    let win = WindowInfo {
        runtime_style: RuntimeStyle::CHROME,
        ..Default::default()
    }
    .set_as_child(
        parent,
        &Rect {
            x: 0,
            y: 0,
            width,
            height,
        },
    );
    let target = if url == "about:blank" {
        home(google)
    } else {
        url
    };
    let Some(b) = browser_host_create_browser_sync(
        Some(&win),
        Some(&mut TabClient::new(state)),
        Some(&string(target)),
        Some(&BrowserSettings {
            background_color: 0xff101113,
            ..Default::default()
        }),
        None,
        Some(&mut ctx),
    ) else {
        unsafe { shell_discard_stage(stage) };
        return false;
    };
    let id = b.identifier();

    WINDOW.with_borrow_mut(|w| {
        if let Some(s) = w {
            if url == SETTINGS_DOCUMENT {
                s.settings_tabs.insert(id);
            }
            s.order.push(id);
            s.current = Some(id)
        }
    });
    unsafe {
        shell_add(id, stage, url == "about:blank");
    }
    refresh(&b);
    if url == "about:blank" {
        unsafe { shell_post(14) }
    }
    true
}
fn selected(id: i32) -> bool {
    WINDOW.with_borrow(|w| w.as_ref().and_then(|s| s.current) == Some(id))
}
pub fn refresh(b: &Browser) {
    let address = b
        .main_frame()
        .map(|f| CefString::from(&f.url()).to_string())
        .unwrap_or_default();
    let internal = settings_document(b.identifier(), &address);
    if let Some(prefs) = WINDOW.with_borrow(|w| w.as_ref().map(Window::prefs)) {
        unsafe {
            shell_sync_settings(b.identifier(), internal, prefs);
        }
    }
    if !selected(b.identifier()) {
        return;
    }
    if let Some(ordinary) =
        web_host(&address).and_then(|_| crate::bookmarks::Bookmark::new("", &address))
    {
        WINDOW.with_borrow_mut(|slot| {
            if let Some(w) = slot
                && !w.session.private
            {
                w.last = ordinary.url.clone();
                if ordinary.url.len() <= 2048 {
                    w.recent.retain(|u| u != &ordinary.url);
                    w.recent.insert(0, ordinary.url);
                    w.recent.truncate(64);
                }
            }
        });
    }
    let enabled = WINDOW.with_borrow(|w| {
        w.as_ref()
            .is_some_and(|s| s.session.filter.is_some() && !s.session.paused(&address))
    });
    unsafe {
        shell_status(
            wide(if internal {
                SETTINGS_ADDRESS
            } else {
                display_url(&address)
            })
            .as_ptr(),
            b.can_go_back() != 0,
            b.can_go_forward() != 0,
            b.is_loading() != 0,
            enabled,
        )
    }
}
pub fn title(b: &Browser, t: &str) {
    unsafe { shell_title(b.identifier(), wide(t).as_ptr()) }
}
fn select(id: i32) {
    let b = BROWSERS.with_borrow(|bs| bs.get(&id).cloned());
    if let Some(b) = b {
        WINDOW.with_borrow_mut(|w| {
            if let Some(s) = w {
                s.current = Some(id)
            }
        });
        unsafe { shell_select(id) };
        refresh(&b);
        if b.main_frame()
            .is_some_and(|f| settings_document(id, &CefString::from(&f.url()).to_string()))
        {
            unsafe {
                shell_focus_settings(id);
            }
        } else if let Some(h) = b.host() {
            h.set_focus(1);
        }
    }
}
pub fn removed(id: i32) {
    let change = WINDOW.with_borrow_mut(|slot| {
        let w = slot.as_mut()?;
        let index = w.order.iter().position(|v| *v == id)?;
        w.order.remove(index);
        w.settings_tabs.remove(&id);
        Some((
            w.current == Some(id),
            w.order.get(index).or_else(|| w.order.last()).copied(),
        ))
    });
    if let Some((was_current, next)) = change {
        // Present the surviving view before retiring the closing parent.
        if was_current && let Some(next) = next {
            select(next);
        }
        unsafe { shell_remove(id) };
        if next.is_none() {
            WINDOW.with_borrow_mut(|w| *w = None);
        }
    }
}
fn save(w: &Window) {
    if w.session.private {
        return;
    }
    if let Ok(mut tmp) = tempfile::NamedTempFile::new_in(&w.session.home) {
        let prefs = serde_json::json!({"placement":if w.vertical{"left"}else{"top"},"width":w.width,"google":w.google,"reopen":w.reopen,"recent":w.recent,"last":if w.reopen {w.last.as_str()}else{""}});
        if write!(tmp, "{prefs}").is_ok() && tmp.as_file().sync_all().is_ok() {
            let _ = tmp.persist(w.session.home.join("layout.json"));
        }
    }
}
pub fn key(b: &Browser, e: &KeyEvent) -> i32 {
    if !selected(b.identifier()) || e.type_ != KeyEventType::RAWKEYDOWN {
        return 0;
    }
    let cmd = match (e.windows_key_code, e.modifiers & 0x0e) {
        (188, 4) => 18,
        (78, 6) => 19,
        (68, 4) => 21,
        (72, 4) => 23,
        (74, 4) => 24,
        (70, 4) => 25,
        (187, 4) | (107, 4) => 26,
        (189, 4) | (109, 4) => 27,
        (48, 4) => 28,
        (114, 0) => 29,
        (76, 4) => 14,
        (84, 4) => 4,
        (87, 4) => 5,
        (82, 4) | (116, 0) => 3,
        (9, 4) => 15,
        (37, 8) => 1,
        (39, 8) => 2,
        _ => 0,
    };
    if cmd == 0 {
        return 0;
    }
    unsafe { shell_post(cmd) };
    1
}
#[unsafe(no_mangle)]
unsafe extern "C" fn browser_action(command: i32, tab: i32, text: *const u16) {
    let Some((state, mut ctx, order, id)) = WINDOW.with_borrow(|slot| {
        slot.as_ref().map(|s| {
            (
                s.session.clone(),
                s.context.clone(),
                s.order.clone(),
                s.current,
            )
        })
    }) else {
        return;
    };
    let browser = id.and_then(|id| BROWSERS.with_borrow(|bs| bs.get(&id).cloned()));
    if command == 10 || (command == 5 && order.len() == 1) {
        WINDOW.with_borrow(|w| {
            if let Some(w) = w {
                save(w);
            }
        });
    }
    match command {
        41 => {
            if text.is_null() {
                return;
            }
            let mut n = 0;
            while n < 8192 && unsafe { *text.add(n) } != 0 {
                n += 1;
            }
            let query = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(text, n) });
            let lower = query.trim().to_lowercase();
            let matches = WINDOW.with_borrow(|slot| {
                let Some(w) = slot.as_ref() else {
                    return vec![];
                };
                let mut rows = vec![];
                if !lower.is_empty() {
                    for b in &w.bookmarks {
                        if b.title.to_lowercase().contains(&lower)
                            || b.url.to_lowercase().contains(&lower)
                        {
                            rows.push(b.url.clone());
                            if rows.len() == 5 {
                                break;
                            }
                        }
                    }
                    for url in &w.recent {
                        if url.to_lowercase().contains(&lower) && !rows.contains(url) {
                            rows.push(url.clone());
                            if rows.len() >= 7 {
                                break;
                            }
                        }
                    }
                }
                rows
            });
            let values: Vec<_> = matches.iter().map(|s| wide(s)).collect();
            let ptrs: Vec<_> = values.iter().map(|s| s.as_ptr()).collect();
            unsafe {
                shell_suggestions(wide(&query).as_ptr(), ptrs.as_ptr(), ptrs.len() as i32);
            }
        }
        18 => open_settings(),
        30..=37 => {
            let trusted = browser
                .as_ref()
                .and_then(|b| {
                    b.main_frame().map(|f| {
                        settings_document(b.identifier(), &CefString::from(&f.url()).to_string())
                    })
                })
                .unwrap_or(false);
            if !trusted {
                return;
            }
            if command == 37 {
                new_page("chrome://settings/content");
                return;
            }
            let prefs = WINDOW.with_borrow_mut(|slot| {
                let w = slot.as_mut()?;
                match command {
                    30 | 31 => w.vertical = command == 31,
                    32 | 33 => w.google = command == 33,
                    34 | 35 => w.reopen = command == 35 && !w.session.private,
                    36 => w.width = tab.clamp(92, 350),
                    _ => return None,
                }
                save(w);
                Some(w.prefs())
            });
            if let (Some(p), Some(id)) = (prefs, id) {
                unsafe {
                    shell_width(p.width);
                    shell_layout(p.vertical != 0);
                    shell_sync_settings(id, true, p);
                }
            }
        }
        19 => {
            if let Err(e) = std::env::current_exe()
                .and_then(|exe| std::process::Command::new(exe).arg("--private").spawn())
            {
                notice("Private window", &e.to_string());
            }
        }
        20..=22 => {
            if let Err(e) = library::action(command, &state, browser.as_ref()) {
                notice("Bookmarks", &e);
            }
            if let Ok(records) = library::library(&state) {
                WINDOW.with_borrow_mut(|slot| {
                    if let Some(w) = slot {
                        w.bookmarks = records;
                    }
                });
            }
        }
        23 | 24 => {
            if command == 23 && state.private {
                notice(
                    "Private browsing",
                    "This private window does not save browsing history.",
                );
            } else {
                new_page(if command == 23 {
                    "chrome://history/"
                } else {
                    "chrome://downloads/"
                });
            }
        }
        25 | 29 => {
            if let Some(h) = browser.as_ref().and_then(|b| b.host()) {
                let previous = WINDOW.with_borrow(|w| w.as_ref().unwrap().find.clone());
                if command == 25 || previous.is_empty() {
                    let mut buf = wide(&previous);
                    buf.resize(513, 0);
                    if unsafe { shell_find(buf.as_mut_ptr()) } == 1 {
                        let query = String::from_utf16_lossy(
                            &buf[..buf.iter().position(|v| *v == 0).unwrap_or(512)],
                        );
                        if query.is_empty() {
                            h.stop_finding(1);
                        } else {
                            h.find(Some(&string(&query)), 1, 0, 0);
                        }
                        WINDOW.with_borrow_mut(|w| {
                            if let Some(w) = w {
                                w.find = query;
                            }
                        });
                    }
                } else {
                    h.find(Some(&string(&previous)), 1, 0, 1);
                }
            }
        }
        26..=28 => {
            if let Some(h) = browser.as_ref().and_then(|b| b.host()) {
                let z = match command {
                    26 => h.zoom_level() + 1.0,
                    27 => h.zoom_level() - 1.0,
                    _ => 0.0,
                };
                h.set_zoom_level(z.clamp(-6., 9.));
            }
        }
        4 => {
            new_page("about:blank");
        }
        8 => select(tab),
        6 | 13 => {
            let layout = WINDOW.with_borrow_mut(|slot| {
                slot.as_mut().map(|w| {
                    if command == 6 {
                        w.vertical = !w.vertical;
                    } else {
                        w.width = tab.clamp(92, 350)
                    }
                    save(w);
                    w.vertical
                })
            });
            if command == 6
                && let Some(left) = layout
            {
                unsafe { shell_layout(left) }
            }
            if let Some(b) = browser.as_ref() {
                refresh(b);
            }
        }
        10 => {
            for id in order {
                let b = BROWSERS.with_borrow(|bs| bs.get(&id).cloned());
                if let Some(h) = b.and_then(|b| b.host()) {
                    h.close_browser(0);
                }
            }
        }
        15 => {
            if let Some(idx) = order.iter().position(|v| Some(*v) == id) {
                select(order[(idx + 1) % order.len()]);
            }
        }
        16 | 17 => {
            let info = WindowInfo {
                runtime_style: RuntimeStyle::CHROME,
                ..Default::default()
            }
            .set_as_popup(Default::default(), "Desktop Browser");
            let dest = if command == 16 {
                "chrome://extensions/"
            } else {
                "about:blank"
            };
            browser_host_create_browser(
                Some(&info),
                Some(&mut TabClient::new(state)),
                Some(&string(dest)),
                Some(&BrowserSettings::default()),
                None,
                Some(&mut ctx),
            );
        }
        _ => {
            if let Some(b) = browser {
                match command {
                    1 => b.go_back(),
                    2 => b.go_forward(),
                    3 => {
                        if b.is_loading() != 0 {
                            b.stop_load()
                        } else {
                            b.reload()
                        }
                    }
                    5 => {
                        if let Some(h) = b.host() {
                            h.close_browser(0);
                        }
                    }
                    7 => {
                        if !text.is_null() {
                            let mut n = 0;
                            while n < 8192 && unsafe { *text.add(n) } != 0 {
                                n += 1;
                            }
                            let input =
                                OsString::from_wide(unsafe { std::slice::from_raw_parts(text, n) })
                                    .to_string_lossy()
                                    .into_owned();
                            if is_settings_input(&input) {
                                open_settings();
                                return;
                            }
                            if let Some(f) = b.main_frame()
                                && let Some(destination) = crate::destination_for(
                                    &input,
                                    WINDOW.with_borrow(|w| w.as_ref().is_some_and(|s| s.google)),
                                )
                            {
                                f.load_url(Some(&string(&destination)));
                                if let Some(h) = b.host() {
                                    h.set_focus(1);
                                }
                            }
                        }
                    }
                    9 if state.extension.is_some() => unsafe { shell_post(11) },
                    9 => {
                        if let Some(f) = b.main_frame() {
                            let u = CefString::from(&f.url()).to_string();
                            if state.filter.is_some()
                                && let Some(host) = web_host(&u)
                            {
                                let mut set = state.exceptions.lock().unwrap();
                                if !set.remove(&host) && set.len() < 1024 {
                                    set.insert(host);
                                }
                                drop(set);
                                refresh(&b);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn finish() {
    unsafe { shell_destroy() }
}

pub fn page_ready(id: i32) {
    unsafe { shell_ready(id) }
}
