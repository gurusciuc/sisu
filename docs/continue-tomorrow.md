# Resume Sisu development

**Platform split, 17 November 2026:** This repository now maintains Sisu for Windows (0.1.0-alpha.1). Native macOS development lives in https://github.com/gurusciuc/sisu-macos with AppKit/WebKit and Rust policy. Preserve Windows evidence and keep its extension, memory and hover gates open. Mac results do not establish Windows acceptance. Earlier multi-platform/CEF Mac notes below are historical context.


Paste the following into an agent working in this repository.

---

Continue development of Sisu from the committed checkpoint. Read AGENTS.md, docs/checkpoint.md, docs/architecture.md, docs/plan-alignment.md and docs/foundation-reassessment.md first. Inspect git status and preserve existing work. This checkpoint contains unfinished features. Do not interpret compilation as functional acceptance.

## User intent and design

Build an everyday Rust desktop browser that preserves security, compatibility, normal logins, active drafts and extension workflows while reducing measured total memory. A small feature set with roughly Chrome's memory consumption is not a successful product. CEF is the current feasibility foundation, not a settled production choice. No engine migration has been approved.

The latest visual direction overrides the earlier sage palette. Use monochrome black and white, neutral `oklch(L% 0 0)` tokens in 5% lightness steps. The user likes the Heron AI button example on Details. References and the video review are in docs/design-reference-review.md. Keep familiar top tabs, optional left tabs, one underlying tab model and the same live content views. Use clean Arial/Helvetica-family typography and consistent geometric icons. No extra UI web runtime, decorative blur or expensive animation. Do not build the marketing site.

## Latest checkpoint

Implemented in source:

- Neutral native light/dark palettes and a matching local new-tab page.
- Dark/Light settings, native blocker On/Off, and links to engine data, download, language and accessibility settings.
- Middle mouse tab-close routing to the clicked tab, preserving CEF beforeunload handling.
- Inset active-tab outline and a scoped native-menu rounding hook.
- Hover handoff/reconciliation changes. The final regression STILL FAILS, see below.
- Browser-data chooser, existing HTML bookmarks importer, and a new bounded JSON cookie importer using the real profile CookieManager. Partitioned cookies are rejected rather than losing scope. Entire input is validated before writes. Asynchronous per-record rejection and flush failures are reported. Private imports are disabled. Engine browser/password import pages are links only, not verified migration features.
- A generated monochrome S logo draft at resources/branding/sisu.png. It is NOT yet converted to an application icon or wired into the bootstrap, taskbar, shortcut or installer. The draft has not received user approval.

Locally tested at this checkpoint:

- Windows x64 release library compiles.
- Six release Rust tests pass: policy, HTML imports and cookie parsing/security attributes. A malformed multi-dot cookie domain test failed first, then passed after rejecting that input.
- tools/settings-check.ps1 passes real Settings open/reuse, top/left preference changes, HTML file-picker import, restart and private preference separation.
- tools/tab-paint-check.ps1 passes real foreground screen pixels in top and left layouts after create/select/close, plus Extensions in a Sisu tab, reuse and no extra window. Palette expectations were updated.
- Settings screenshot inspected. Lower settings rows require scrolling. Native scrollbar still uses the system light track in dark mode.
- The final hover test FAILS: previous caption button remains hover-colored during the simulated rapid button handoff. Expected #0b0b0b, observed #3a3a3a. Reproduce using tools/hover-check.ps1. Older pre-monochrome passes do not supersede this failure.

Not yet accepted:

- Actual light/dark switching, ordinary persistence, private ephemerality and page prefers-color-scheme response without reload/draft loss.
- Middle-click background/selected/last-tab closure and beforeunload cancellation.
- Rounded menu shape/accessibility and scaling.
- Actual JSON cookie import, HttpOnly exclusion from document.cookie, scope, restart persistence and private separation. Parser tests alone do not prove CEF integration.
- Actual destinations and behavior of each new engine-settings/import link.
- A full Ponytail audit, fresh complete smoke/capabilities run and final local preview replacement.
- Icon integration, Windows installer, macOS DMG preparation and native signing. Nothing has been publicly released as a finished browser.

## Work order

1. Reproduce and fix the hover regression with tools/hover-check.ps1. Inspect event ordering and invalidation; do not mask the bug by invalidating the whole frame continuously. Screen tests must own focus and restore the cursor. Keep redraw overhead bounded.
2. Add real tests for middle-click closure, theme changes and cookie import before calling those features done. Use isolated BROWSER_TEST_HOME and controlled local fixtures. No personal password vaults, browser profiles or decrypted OS browser databases. Browser cookies may contain bearer login sessions. Preserve Secure, HttpOnly, SameSite, host-only/domain/path and expiry. Reject partitioned data until the engine API can preserve its scope. Importing login cookies cannot promise a valid session at the provider.
3. Check every new settings page actually works. Verify menu rounding and tab clipping at normal and increased display scaling. Fix the Settings document's hard-coded old dark background and theme-transition surfaces if they flash. Recheck trusted Settings routing and private storage boundaries.
4. Run existing keyboard, Settings, frame, address, tab-paint, transition, smoke and capability workflows. Update evidence with actual outcomes and failures. Do not silently overwrite historical measurements with a different workload.
5. Review the repository and changed flows using the Ponytail repository's instructions. Reference commit 96608e078b2d5ea607c5c3b8b5a54d1c1fba6846 from https://github.com/DietrichGebert/ponytail. It supplies review guidance, not a runtime verifier. Do not install global hooks. Produce a ranked, concrete audit report and state unread/unrun areas. Fix actionable defects in a separate implementation pass.
6. Integrate the original monochrome logo draft after checking small-size legibility. Convert to ICO/ICNS with a reproducible asset pipeline, wire window/shortcut branding, and keep engine sandbox bootstrap names and ABI intact.
7. Prepare a clearly labeled unsigned preview Windows installer, per-user by default, Apache license/notices, optional shortcut, no forced default-browser takeover, preserve user profile on uninstall. Build and install/uninstall in a disposable location. Investigate VC++ runtime prerequisites. Keep production signing a release gate. Prepare macOS DMG on a native runner using the complete app/helper bundle; signing/notarization and runtime checks remain required. No production public release from this checkpoint.
8. Update the user's local preview only after the fixes pass, then commit and push reviewed changes. The user already authorized closing Sisu while unused. Gracefully close only the known Sisu executable, preserve the profile, and never kill unrelated Chrome/CEF processes. Do not repeatedly ask for restart permission.

## Architecture and performance gates

The real MV3 fixture observed two distinct Chromium window IDs and two active tabs for two tabs in one Sisu shell. This breaks assumptions behind tabs/groups, active-tab grants, extension popups and side panels. Resolve this before promising general Chrome-extension compatibility. Real uBOL DNR filtering passed earlier. Proton Pass only reached signed-out onboarding in the reference UI; vault workflows are unverified. Claude's provider officially supports Google Chrome. Do not substitute an AI sidebar and call it extension compatibility.

Current follow-up baseline, collected BEFORE these theme/import changes, is under evidence/windows-x64/followup-baseline: two live local pages, median 156.29 MiB private working set and 261.23 MiB private commit, sampled peaks 156.87/261.95 MiB. Full attributable Windows process tree; shared pages and external GPU allocations excluded. The viewport changed from the older 138.56/226.93 baseline. These are not matched comparisons. There is no demonstrated memory advantage over Chrome or Firefox. Measure equivalent active content, blocking, extension state, memory-saver policy, CPU and latency before claiming gains.

Keep sandboxing, TLS, origin boundaries and extension/browser/site privilege separation. Never use no-sandbox or remote debugging as a test shortcut. Private settings/history/session data must stay out of ordinary records. No automatic tab discarding is enabled. Native blocking remains incomplete for generic cosmetics, document-start injection, scriptlets, replacements, signed list updates and broader paths. Cross-platform CI configuration is not runtime verification.

## Build and paths

On the current Windows host, workspace root is `C:/Users/quick/Documents/Codex/2026-11-16/act-as-the-lead-engineer-for`; source is `outputs/browser`. From the workspace root:

```powershell
. ./work/env.ps1
$env:CARGO_TARGET_DIR='R:\build'
$env:CEF_PATH='R:\sdk'
cargo fmt --all -- --check
cargo test --release --locked --no-default-features --test policy --test import --test cookies
cargo build --release --locked --lib
```

env.ps1 changes cwd to outputs/browser. R: is a subst mapping to workspace work; restore that mapping if absent. MSVC, CMake and Ninja paths are added by env.ps1. The matching sandbox bootstrap and CEF runtime are required. Do not launch the DLL. Build output is R:/build/release/desktop_browser.dll. See tools/build-windows.ps1 for a portable clean-host build.

The latest compiled candidate was copied to `outputs/sisu-preview/desktop_browser.dll`. User's installed local preview is `outputs/windows-preview/desktop_browser.exe`, still the older build. Keep them distinct until candidate validation. Test commands from the repo:

```powershell
./tools/settings-check.ps1 -Exe "$PWD/../sisu-preview/desktop_browser.exe" -Out "$PWD/../../work/resume-checks"
./tools/hover-check.ps1 -Exe "$PWD/../sisu-preview/desktop_browser.exe" -Out "$PWD/../../work/resume-checks"
./tools/tab-paint-check.ps1 -Exe "$PWD/../sisu-preview/desktop_browser.exe" -Out "$PWD/../../work/resume-checks"
node tools/keyboard-check.mjs ../sisu-preview/desktop_browser.exe ../../work/resume-keys
```

Inno Setup 7.1.1 is installed locally at R:/inno/ISCC.exe. Official download https://github.com/jrsoftware/issrc/releases/download/is-7_1_1/innosetup-7.1.1-x64.exe was SHA-256 checked (8a5a6711c3766a6e5fb74f280ba0e3551e719ce94538d901fdbd1f8f9920b3f0) and Authenticode valid for Pyrsys B.V. Compiler availability does not mean an installer exists.

Public source repository is https://github.com/gurusciuc/sisu. Normal git push uses the configured credential manager. Do not print credentials. Maintain Apache-2.0, NOTICE, dependency notices, AGENTS.md and honest checkpoint documentation. Public source publication is authorized. Signed production releases remain a later milestone.

Work autonomously on routine reversible fixes. Give concise updates, then report what is implemented, locally tested, configured elsewhere, unverified and blocked. Complete the current defect batch before expanding the interface.

---
