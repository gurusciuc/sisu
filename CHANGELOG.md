# Changelog

## 0.1.0-alpha.1 · 17 November 2026

This is an engineering alpha checkpoint, not a production browser release. The existing repository now focuses on Windows. Native Mac development is maintained independently at https://github.com/gurusciuc/sisu-macos. No Windows runtime revalidation was performed on the Mac host during this repository split.

- Build and stage the macOS CEF reference preview using Command Line Tools, Rust, CMake and Ninja. Full Xcode/ibtool is no longer required.
- Create native Cocoa menus, route browser commands to the focused window and handle Command-Q through Sisu's application event path.
- Close reference windows through their native delegates and retain engine unload handling. Foreground Command-Q now exits with code 0 in the controlled fixture.
- Canonicalize profile roots consistently with CEF to prevent `/var` versus `/private/var` from causing an in-memory fallback. A real restart retains fixture cookies and local storage.
- Give sandbox helpers distinct bundle identifiers, include license notices, and generate a locally ad-hoc signed archive. Developer ID signing and notarization are still release gates.
- Record the user's Mac priority: practical memory use can take precedence over Chrome-extension parity. Mac and Windows may use different engines and native shells. WebKit/AppKit is the next Mac candidate; no memory advantage is claimed yet.

The CEF Mac preview still uses Chromium's reference interface. The Windows hover defect and other open acceptance gates remain documented in the continuation notes.
