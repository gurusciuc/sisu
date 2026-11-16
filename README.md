# Sisu

An open-source desktop browser prototype with a Rust core, the Chromium Embedded Framework (CEF) and native Windows controls. The working Windows preview has real web navigation, live top or left tabs, profile settings, bookmarks and native ad and tracker filtering. Linux and macOS have build definitions and an engine reference shell. Start with the [milestone report](docs/checkpoint.md) for tested features and open release gates.

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
| Move the tab strip | Use the placement icon or Settings → Appearance. Drag the sidebar edge or apply a width in Settings. The overflow menu lists tabs that do not fit. |
| Import bookmarks | Export bookmarks as HTML from Brave, Chrome or Firefox. Choose Settings → Import bookmarks and select that file. Folders are preserved and duplicate URLs are skipped. Bookmarks in the browser menu opens the library. |
| Save or find a page | Ctrl+D saves a web bookmark. Ctrl+F opens Find. F3 finds the next match. |
| Recover a filtered site | Click Blocking on for the current site and reload. Click Paused and reload to resume. Exceptions last for the session. |
| Privacy, downloads and history | Ctrl+Shift+N opens a new private window. Settings → Site permissions opens the engine’s controls. Ctrl+J and Ctrl+H open downloads and history. |

Restart can reopen the last ordinary web page. Multi-tab session restoration, password or history migration, signed updates and a tested everyday extension installation path are still open. Strict private windows use ephemeral site data and deny media/permission requests.

## Checks

```powershell
cargo test --release --locked --no-default-features --test policy --test import
./tools/fetch-ubol.ps1 -Directory C:/db-ubol
node tools/smoke.mjs exe=dist/preview/desktop_browser.exe out=dist/results ubol=C:/db-ubol/unpacked
./tools/settings-check.ps1 -Exe "$PWD/dist/preview/desktop_browser.exe" -Out "$PWD/dist/results"
node tools/capabilities.mjs dist/preview/desktop_browser.exe dist/results/capabilities.json
node tools/transition.mjs dist/preview/desktop_browser.exe dist/results/new-tab-transition.json
node tools/benchmark.mjs dist/preview/desktop_browser.exe dist/results
```

The runtime can also start with `--reference-ui` for CEF’s native Chrome-style window. The explicitly enabled `--extension-lab=<unpacked-directory>` uses a separate profile, loads one package and turns off native filtering for attribution. Keep the proof-of-concept extension package outside the ordinary profile.

## Linux and macOS engineering builds

The [check workflow](.github/workflows/check.yml) specifies runner dependencies and separate Linux X11, Linux Wayland and macOS jobs. From the repository root run `cargo build --locked --release --features packaging --bins`, followed by `cargo run --locked --release --features packaging --bin stage-posix -- target/release dist/platform`. Linux uses the staged `browser-shell` with its runtime libraries. macOS uses the staged application bundle and helper processes. Native acceptance on these hosts is pending.

## Project

AI coding assistants should begin with [AGENTS.md](AGENTS.md). The [plan alignment audit](docs/plan-alignment.md) maps the supplied product plan to implemented behavior, evidence and remaining release gates. The [roadmap](docs/roadmap.md) records the next steps.

[Architecture and engine decision](docs/architecture.md) · [Extension compatibility](docs/extensions.md) · [Benchmarks and raw data](docs/benchmarks.md) · [Contribution guide](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Copyright and attribution](NOTICE) · [Third-party notices](THIRD_PARTY.md)
