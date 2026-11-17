# macOS alpha checkpoint · 17 November 2026

Version: **0.1.0-alpha.1**. This source checkpoint is an engineering alpha. No production binary release is implied.

## Working CEF reference preview

The Apple Silicon release build runs with CEF's Chromium reference interface, Sisu's Rust request filtering and a persistent ordinary profile. `tools/build-macos.sh` builds the complete application/framework/helper bundle, includes license notices, applies local ad-hoc signatures, verifies nested signatures and writes a ZIP. Its temporary staging directory avoids signing-incompatible Finder metadata added by the Documents file provider. It does not remove quarantine or disable the engine sandbox.

Native menus no longer depend on the old cefsimple XIB or Xcode's ibtool. Browser menu commands use CEF's command IDs and the focused native window. Command-Q is handled in the app's own event dispatch, requests non-forced browser closure and closes reference windows through their native delegates. It never kills unrelated Chrome/CEF processes.

CEF canonicalizes cache paths. The profile root now uses the same canonical representation, fixing the observed `/var` and `/private/var` mismatch and in-memory storage fallback.

## Actual local results

Host: Apple Silicon, Darwin 27.2.0. Rust 1.100.0, locked release dependencies, CEF 156.0.30 / cef-rs 156.6.0.

- Release application and sandbox helper build passed.
- All six existing release policy/import/cookie tests passed.
- Local ad-hoc framework/helper/app signature verification passed.
- A fresh isolated HTTP fixture loaded; the control and explicit exception reached the server, the direct blocked script did not, and the cosmetic fixture was hidden.
- A real restart with the same isolated profile observed `prior=mac-first` in local storage and `fixture=mac-first` in cookies. No invalid-cache/in-memory-fallback diagnostic occurred after canonicalization.
- A foreground Command-Q on the restarted fixture caused the application to exit with code 0. The process exit was observed without reopening the app during shutdown.
- Native new-tab/close commands were exercised during the earlier Mac integration pass. Broader command, multiple-window and unload-cancellation coverage remains open.

Raw browser reports and exits: [CEF profile evidence](../evidence/mac-arm64/cef-profile-check.json). The file contains only controlled fixture state. Private separation, arbitrary beforeunload cancellation, real-site logins, extension workflows and comprehensive sandbox inspection have not passed the full Mac acceptance suite.

## Foundation direction

The user observed lower memory use in Safari and explicitly chose **lowest practical memory with WebKit extension limits** for Mac. They also authorized different engines and native shells for Mac and Windows. These decisions supersede the earlier assumption that one identical engine must serve all platforms. Each process will still load one engine.

The next Mac candidate uses AppKit controls, system WebKit, and Rust tab/address policy. Keep the CEF preview as a comparison and preserve its profiles. The candidate must prove live tabs, drafts, persistent/private boundaries, security, blocking coverage and full attributable physical footprint before promotion. A smaller archive, a small UI process or compilation alone is not a memory benchmark. No verified Safari/CEF/WebKit matched comparison has been collected at this checkpoint.

Windows retains its existing foundation until Windows-native extension and resource measurements justify a change. This Mac choice does not establish WebView2, Firefox or a Chromium fork as the Windows winner. Repository separation is an organizational choice; it is independent of engine selection.
