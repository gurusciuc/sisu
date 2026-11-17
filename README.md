# Sisu for Windows

**0.1.0-alpha.1 — an engineering alpha, not a production release.** This repository maintains the Windows implementation: a Rust core, native Win32 controls and the current CEF/Chromium feasibility foundation. It provides real navigation, top/left live tabs, profile settings, bookmarks and native filtering. Extension topology, the caption-hover regression, matched memory savings and distribution signing remain open gates. See [the checkpoint](docs/checkpoint.md).

The native Mac version lives in [gurusciuc/sisu-macos](https://github.com/gurusciuc/sisu-macos). Different platforms may use different engines and native interfaces.

## Run on Windows x64

In the staged preview directory, run **desktop_browser.exe** alongside its DLLs, resources and locales. It opens the ordinary profile. Run `desktop_browser.exe --private` for a fresh private window. The executable is the CEF sandbox bootstrap. The preview is unsigned.

To build from source, install the Rust toolchain via rustup, Visual Studio 2022 C++ desktop build tools with the Windows SDK, CMake, Ninja, curl and tar. Node 22 or later and Windows PowerShell run the browser tests. From the repository root run the following in PowerShell. Allow several GB for the engine and compiler output. A short build path avoids MSVC path-length failures.

```powershell
./tools/build-windows.ps1 -BuildRoot C:/db-build
./dist/preview/desktop_browser.exe
```

The script validates the pinned official x64 CEF archive, builds the locked release library and stages its matching sandbox bootstrap and runtime files. Keep the staging directory together. For a fresh, disposable ordinary profile set `BROWSER_TEST_HOME` to a new directory before launch. Leave existing personal browser profiles outside the test.

## Use the Windows preview

| Action | Where |
| --- | --- |
| Settings | Browser menu → Settings, Ctrl+, or type `#settings` / `browser://settings` in the address bar. Opening Settings again selects an existing Settings tab. Choices take effect immediately. |
| Open or close a tab | +, Ctrl+T, the selected tab’s ×, or Ctrl+W. Ctrl+Tab cycles through the model. |
| Navigate | Ctrl+L, type an address or search, then Enter. Bare domains use HTTPS. Use Alt+Left / Alt+Right for history and Ctrl+R to reload. |
| Address suggestions | Matches recent URLs and bookmark titles/URLs locally. Use Up/Down and Enter to select, or Escape to dismiss. Provider search begins on commit. |
| Move the tab strip | Use the placement icon or Settings → Appearance. Drag the sidebar edge or apply a width in Settings. The overflow menu lists tabs that do not fit. |
| Import bookmarks | Export bookmarks as HTML from Brave, Chrome or Firefox. Choose Settings → Import bookmarks and select that file. Folders are preserved and duplicate URLs are skipped. Bookmarks in the browser menu opens the library. |
| Save or find a page | Ctrl+D saves a web bookmark. Ctrl+F opens Find. F3 finds the next match. |
| Recover a filtered site | Click Blocking on for the current site and reload. Click Paused and reload to resume. Exceptions last for the session. |
| Privacy, downloads and history | Ctrl+Shift+N opens a new private window. Settings → Site permissions opens the engine’s controls. Ctrl+J and Ctrl+H open downloads and history. |

Restart can reopen the last ordinary web page. Multi-tab session restoration, password or history migration, signed updates and a tested everyday extension installation path are still open. Strict private windows use ephemeral site data and deny media/permission requests.

## Checks

```powershell
cargo test --release --locked --no-default-features --test policy --test import --test cookies
./tools/fetch-ubol.ps1 -Directory C:/db-ubol
node tools/smoke.mjs exe=dist/preview/desktop_browser.exe out=dist/results ubol=C:/db-ubol/unpacked
./tools/settings-check.ps1 -Exe "$PWD/dist/preview/desktop_browser.exe" -Out "$PWD/dist/results"
node tools/capabilities.mjs dist/preview/desktop_browser.exe dist/results/capabilities.json
node tools/transition.mjs dist/preview/desktop_browser.exe dist/results/new-tab-transition.json
node tools/address-check.mjs dist/preview/desktop_browser.exe dist/results
node tools/keyboard-check.mjs dist/preview/desktop_browser.exe dist/results
./tools/frame-check.ps1 -Exe "$PWD/dist/preview/desktop_browser.exe" -Out "$PWD/dist/results"
node tools/benchmark.mjs dist/preview/desktop_browser.exe dist/results
```

The runtime can also start with `--reference-ui` for CEF’s native Chrome-style window. The explicitly enabled `--extension-lab=<unpacked-directory>` uses a separate profile, loads one package and turns off native filtering for attribution. Keep the proof-of-concept extension package outside the ordinary profile.

## macOS version

The independent native Mac implementation is maintained in [gurusciuc/sisu-macos](https://github.com/gurusciuc/sisu-macos). It uses AppKit, system WebKit and Rust policy. Mac and Windows have separate engines, installers, runtime evidence and release gates. The earlier CEF Mac reference work remains in history and [the Mac alpha notes](docs/macos-alpha.md).

## Project

AI coding assistants should begin with [AGENTS.md](AGENTS.md). The [plan alignment audit](docs/plan-alignment.md) maps the supplied product plan to implemented behavior, evidence and remaining release gates. The [roadmap](docs/roadmap.md) records the next steps.

[Architecture and engine decision](docs/architecture.md) · [Extension compatibility](docs/extensions.md) · [Benchmarks and raw data](docs/benchmarks.md) · [Contribution guide](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Copyright and attribution](NOTICE) · [Third-party notices](THIRD_PARTY.md)
