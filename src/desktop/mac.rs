// Adapted from tauri-apps/cef-rs cefsimple under MIT. See notices/cef-rs-MIT.txt.
use super::BROWSERS;
use cef::application_mac::{CefAppProtocol, CrAppControlProtocol, CrAppProtocol};
use cef::{BrowserHost, ImplBrowser, ImplBrowserHost, WindowOpenDisposition, sys};
use objc2::{
    ClassType, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, extern_methods,
    msg_send,
    rc::Retained,
    runtime::{AnyObject, Bool, NSObject, NSObjectProtocol, ProtocolObject, Sel},
    sel,
};
use objc2_app_kit::{
    NSApp, NSApplication, NSApplicationDelegate, NSApplicationTerminateReply, NSEvent,
    NSEventModifierFlags, NSEventType, NSMenu, NSMenuItem, NSUserInterfaceValidations,
    NSValidatedUserInterfaceItem, NSView,
};
use objc2_foundation::{NSObjectNSThreadPerformAdditions, NSString};
use std::cell::Cell;

define_class! {
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    pub struct SimpleAppDelegate;

    impl SimpleAppDelegate {
        #[unsafe(method(createApplication:))]
        unsafe fn create_application(&self, _object: Option<&AnyObject>) {
            let mtm = MainThreadMarker::new().expect("Not running on the main thread");
            let app = NSApp(mtm);
            // Chrome finishes installing its app controller asynchronously.
            // Match cefsimple's deferred delegate setup after initialization.
            app.setDelegate(Some(ProtocolObject::from_ref(self)));
            setup_native_menu(&app, self, mtm);
        }

        #[unsafe(method(performBrowserCommand:))]
        unsafe fn perform_browser_command(&self, sender: &NSMenuItem) {
            if let Some(host) = focused_host() {
                let command = sender.tag() as i32;
                if host.can_execute_chrome_command(command) != 0 {
                    host.execute_chrome_command(command, WindowOpenDisposition::CURRENT_TAB);
                }
            }
        }
    }

    unsafe impl NSObjectProtocol for SimpleAppDelegate {}

    unsafe impl NSApplicationDelegate for SimpleAppDelegate {
        #[unsafe(method(applicationShouldTerminate:))]
        unsafe fn application_should_terminate(&self, _sender: &NSApplication) -> NSApplicationTerminateReply {
            NSApplicationTerminateReply::TerminateNow
        }

        /// Called when the user clicks the app dock icon while the application is
        /// already running.
        #[unsafe(method(applicationShouldHandleReopen:hasVisibleWindows:))]
        unsafe fn application_should_handle_reopen(&self, _sender: &NSApplication, _has_visible_windows: Bool) -> Bool {
            let browser = BROWSERS.with_borrow(|bs| bs.values().next().cloned()); if let Some(host) = browser.and_then(|b| b.host()) { host.set_focus(1); }
            Bool::NO
        }

        /// Requests that any state restoration archive be created with secure encoding
        /// (macOS 12+ only). See https://crrev.com/c737387656 for details. This also
        /// fixes an issue with macOS default behavior incorrectly restoring windows
        /// after hard reset (holding down the power button).
        #[unsafe(method(applicationSupportsSecureRestorableState:))]
        unsafe fn application_supports_secure_restorable_state(&self, _sender: &NSApplication) -> Bool {
            Bool::YES
        }
    }

    unsafe impl NSUserInterfaceValidations for SimpleAppDelegate {
        #[unsafe(method(validateUserInterfaceItem:))]
        unsafe fn validate_user_interface_item(&self, item: &ProtocolObject<dyn NSValidatedUserInterfaceItem>) -> Bool {
            Bool::new(item.action() == Some(sel!(performBrowserCommand:)) &&
                focused_host().is_some_and(|host| host.can_execute_chrome_command(item.tag() as i32) != 0))
        }
    }
}

impl SimpleAppDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = SimpleAppDelegate::alloc(mtm).set_ivars(());
        unsafe { msg_send![super(this), init] }
    }
}

/// Instance variables of `SimpleApplication`.
#[derive(Default)]
pub struct SimpleApplicationIvars {
    handling_send_event: Cell<Bool>,
}

define_class!(
    /// A `NSApplication` subclass that implements the required CEF protocols.
    ///
    /// This class provides the necessary `CefAppProtocol` conformance to
    /// ensure that events are handled correctly by the Chromium framework on macOS.
    #[unsafe(super(NSApplication))]
    #[ivars = SimpleApplicationIvars]
    pub struct SimpleApplication;

    impl SimpleApplication {
        #[unsafe(method(sendEvent:))]
        unsafe fn send_event(&self, event: &NSEvent) {
            let was_sending_event = self.is_handling_send_event();
            if !was_sending_event {
                self.set_handling_send_event(true);
            }

            // Chrome's reference-window accelerator path can consume Cmd-Q
            // before it reaches our application-menu target. Handle the app's
            // standard quit equivalent here, without a global event monitor.
            let flags = event.modifierFlags();
            let quit = event.r#type() == NSEventType::KeyDown
                && flags.contains(NSEventModifierFlags::Command)
                && !flags.intersects(NSEventModifierFlags::Control | NSEventModifierFlags::Option)
                && event.charactersIgnoringModifiers().is_some_and(|s| s.to_string().eq_ignore_ascii_case("q"));
            if quit { request_termination(self); }
            else { let _: () = msg_send![super(self), sendEvent:event]; }

            if !was_sending_event {
                self.set_handling_send_event(false);
            }
        }

        /// |-terminate:| is the entry point for orderly "quit" operations in Cocoa. This
        /// includes the application menu's quit menu item and keyboard equivalent, the
        /// application's dock icon menu's quit menu item, "quit" (not "force quit") in
        /// the Activity Monitor, and quits triggered by user logout and system restart
        /// and shutdown.
        ///
        /// The default |-terminate:| implementation ends the process by calling exit(),
        /// and thus never leaves the main run loop. This is unsuitable for Chromium
        /// since Chromium depends on leaving the main run loop to perform an orderly
        /// shutdown. We support the normal |-terminate:| interface by overriding the
        /// default implementation. Our implementation, which is very specific to the
        /// needs of Chromium, works by asking the application delegate to terminate
        /// using its |-tryToTerminateApplication:| method.
        ///
        /// |-tryToTerminateApplication:| differs from the standard
        /// |-applicationShouldTerminate:| in that no special event loop is run in the
        /// case that immediate termination is not possible (e.g., if dialog boxes
        /// allowing the user to cancel have to be shown). Instead, this method tries to
        /// close all browsers by calling CloseBrowser(false) via
        /// ClientHandler::CloseAllBrowsers. Calling CloseBrowser will result in a call
        /// to ClientHandler::DoClose and execution of |-performClose:| on the NSWindow.
        /// DoClose sets a flag that is used to differentiate between new close events
        /// (e.g., user clicked the window close button) and in-progress close events
        /// (e.g., user approved the close window dialog). The NSWindowDelegate
        /// |-windowShouldClose:| method checks this flag and either calls
        /// CloseBrowser(false) in the case of a new close event or destructs the
        /// NSWindow in the case of an in-progress close event.
        /// ClientHandler::OnBeforeClose will be called after the CEF NSView hosted in
        /// the NSWindow is dealloc'ed.
        ///
        /// After the final browser window has closed ClientHandler::OnBeforeClose will
        /// begin actual tear-down of the application by calling CefQuitMessageLoop.
        /// This ends the NSApplication event loop and execution then returns to the
        /// main() function for cleanup before application termination.
        ///
        /// The standard |-applicationShouldTerminate:| is not supported, and code paths
        /// leading to it must be redirected.
        #[unsafe(method(terminate:))]
        unsafe fn terminate(&self, _sender: Option<&AnyObject>) {
            request_termination(self);
        }
    }

    unsafe impl CrAppControlProtocol for SimpleApplication {
        #[unsafe(method(setHandlingSendEvent:))]
        unsafe fn _set_handling_send_event(&self, handling_send_event: Bool) {
            self.ivars().handling_send_event.set(handling_send_event);
        }
    }

    unsafe impl CrAppProtocol for SimpleApplication {
        #[unsafe(method(isHandlingSendEvent))]
        unsafe fn _is_handling_send_event(&self) -> Bool {
            self.ivars().handling_send_event.get()
        }
    }

    unsafe impl CefAppProtocol for SimpleApplication {}
);

fn request_termination(app: &NSApplication) {
    let tabs = BROWSERS.with_borrow(|bs| bs.values().cloned().collect::<Vec<_>>());
    for browser in tabs {
        if let Some(host) = browser.host() {
            host.close_browser(0);
        }
    }
    // Close Chrome-created reference windows through their native delegates;
    // these can exist outside the embedder-created browser registry.
    for window in app.windows() {
        window.performClose(None);
    }
}

impl SimpleApplication {
    extern_methods! {
        #[unsafe(method(sharedApplication))]
        fn shared_application() -> Retained<Self>;

        #[unsafe(method(setHandlingSendEvent:))]
        fn set_handling_send_event(&self, handling_send_event: bool);

        #[unsafe(method(isHandlingSendEvent))]
        fn is_handling_send_event(&self) -> bool;
    }
}

pub fn setup_simple_application() {
    // Initialize the SimpleApplication instance.
    // SAFETY: mtm ensures that here is the main thread.
    let _ = SimpleApplication::shared_application();

    // If there was an invocation to NSApp prior to here,
    // then the NSApp will not be a SimpleApplication.
    // The following assertion ensures that this doesn't happen.
    assert!(
        NSApp(MainThreadMarker::new().expect("Not running on the main thread"))
            .isKindOfClass(SimpleApplication::class())
    );
}

pub fn setup_simple_app_delegate() -> Retained<SimpleAppDelegate> {
    let mtm = MainThreadMarker::new().expect("Not running on the main thread");

    // Create the application delegate.
    let simple_delegate = SimpleAppDelegate::new(mtm);
    let delegate_proto =
        ProtocolObject::<dyn NSApplicationDelegate>::from_retained(simple_delegate.clone());
    let app = NSApp(MainThreadMarker::new().expect("Not running on the main thread"));
    assert!(app.isKindOfClass(SimpleApplication::class()));
    app.setDelegate(Some(&delegate_proto));
    assert!(
        app.delegate()
            .unwrap()
            .isKindOfClass(SimpleAppDelegate::class())
    );

    unsafe {
        simple_delegate.performSelectorOnMainThread_withObject_waitUntilDone(
            sel!(createApplication:),
            None,
            false,
        );
    }

    simple_delegate
}

// Construct ordinary Cocoa menus without an Interface Builder resource, so a
// local engineering build only needs Apple's Command Line Tools. Browser
// commands remain in CEF's reference toolbar and its keyboard handling.
fn setup_native_menu(app: &NSApplication, delegate: &SimpleAppDelegate, mtm: MainThreadMarker) {
    fn item(mtm: MainThreadMarker, title: &str, key: &str, action: Sel) -> Retained<NSMenuItem> {
        // SAFETY: all callers supply standard Cocoa responder selectors. A nil
        // target deliberately uses the first responder chain.
        unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(title),
                Some(action),
                &NSString::from_str(key),
            )
        }
    }
    fn submenu(bar: &NSMenu, title: &str, mtm: MainThreadMarker) -> Retained<NSMenu> {
        let menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &NSString::from_str(title));
        let top = NSMenuItem::new(mtm);
        top.setSubmenu(Some(&menu));
        bar.addItem(&top);
        menu
    }
    let browser_command = |menu: &NSMenu, title: &str, key: &str, command| {
        let entry = item(mtm, title, key, sel!(performBrowserCommand:));
        entry.setTag(command as isize);
        // SAFETY: the delegate is retained until the CEF message loop exits.
        unsafe {
            entry.setTarget(Some(delegate));
        }
        menu.addItem(&entry);
    };

    let bar = NSMenu::new(mtm);
    let application = submenu(&bar, "Sisu Preview", mtm);
    for (title, key, action) in [
        (
            "About Sisu Preview",
            "",
            sel!(orderFrontStandardAboutPanel:),
        ),
        ("Hide Sisu Preview", "h", sel!(hide:)),
        ("Show All", "", sel!(unhideAllApplications:)),
        ("Quit Sisu Preview", "q", sel!(terminate:)),
    ] {
        let command = item(mtm, title, key, action);
        // SAFETY: NSApplication lives for the complete menu lifetime.
        unsafe {
            command.setTarget(Some(app));
        }
        application.addItem(&command);
    }
    browser_command(&application, "Settings…", ",", sys::IDC_OPTIONS);
    let file = submenu(&bar, "File", mtm);
    browser_command(&file, "New Tab", "t", sys::IDC_NEW_TAB);
    browser_command(&file, "New Window", "n", sys::IDC_NEW_WINDOW);
    browser_command(&file, "Close Tab", "w", sys::IDC_CLOSE_TAB);
    let edit = submenu(&bar, "Edit", mtm);
    for (title, key, action) in [
        ("Undo", "z", sel!(undo:)),
        ("Redo", "Z", sel!(redo:)),
        ("Cut", "x", sel!(cut:)),
        ("Copy", "c", sel!(copy:)),
        ("Paste", "v", sel!(paste:)),
        ("Select All", "a", sel!(selectAll:)),
    ] {
        edit.addItem(&item(mtm, title, key, action));
    }
    browser_command(&edit, "Find…", "f", sys::IDC_FIND);
    let view = submenu(&bar, "View", mtm);
    browser_command(&view, "Open Location…", "l", sys::IDC_FOCUS_LOCATION);
    browser_command(&view, "Reload", "r", sys::IDC_RELOAD);
    browser_command(&view, "Actual Size", "0", sys::IDC_ZOOM_NORMAL);
    browser_command(&view, "Zoom In", "+", sys::IDC_ZOOM_PLUS);
    browser_command(&view, "Zoom Out", "-", sys::IDC_ZOOM_MINUS);
    let window = submenu(&bar, "Window", mtm);
    window.addItem(&item(mtm, "Minimize", "m", sel!(performMiniaturize:)));
    window.addItem(&item(mtm, "Zoom", "", sel!(performZoom:)));
    app.setWindowsMenu(Some(&window));
    app.setMainMenu(Some(&bar));
}

fn focused_host() -> Option<BrowserHost> {
    let mtm = MainThreadMarker::new()?;
    let key = NSApp(mtm).keyWindow()?;
    // Release the RefCell borrow before crossing into CEF/Cocoa callbacks.
    let browsers = BROWSERS.with_borrow(|bs| bs.values().cloned().collect::<Vec<_>>());
    browsers.into_iter().find_map(|browser| {
        let host = browser.host()?;
        // CEF's macOS window handle is an NSView owned by the live browser.
        // This runs on the UI thread; OnBeforeClose removes dead browsers.
        let view = unsafe { host.window_handle().cast::<NSView>().as_ref()? };
        let window = view.window()?;
        std::ptr::eq(&*window, &*key).then_some(host)
    })
}
