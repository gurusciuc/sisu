# Working on Sisu

Read this file first, followed by docs/checkpoint.md, docs/architecture.md and docs/plan-alignment.md. The project name is Sisu and the repository is https://github.com/gurusciuc/sisu. The current package, executable and profile identifiers still use the original Desktop Browser working name. Original code uses Apache-2.0. Preserve NOTICE and third-party licenses.

## Product intent

Build an everyday desktop browser with the lowest practical measured memory use while preserving website compatibility, security, privacy, input correctness and active user work. Rust owns core state and application policy. The first product uses one mature engine. A custom Rust renderer is a separate research track.

The default layout has horizontal tabs at the top. Left tabs share the same model and live views. Keep the placement design extensible to right and bottom. Switching layout must preserve order, selection, navigation and drafts. The latest visual direction is monochrome black and white with neutral OKLCH lightness in 5% steps, clean Helvetica/Arial-family text and conventional geometric icons. The Heron example on Details informs restraint, fine outlines and spacing. This replaces the earlier sage palette. Ardia and ArtCraft are references for a future marketing site.

## Implementation map

- src/desktop.rs owns CEF process setup, request context policy, callbacks, request filtering and private boundaries.
- src/desktop/windows.rs owns tab order and selection, commands, settings and shell persistence.
- native/shell.cpp hosts Chrome-style child views in Win32 controls.
- native/settings.inl draws settings inside a regular tab. browser://settings and #settings are shell-address aliases. The settings controls have native command routing. Web content has no privileged JavaScript bridge.
- src/desktop/windows/library.rs and src/bookmarks.rs implement bounded HTML bookmark imports and profile storage.
- src/lib.rs exposes adblock-rust filtering and address resolution.
- build.rs validates and compiles pinned EasyList/EasyPrivacy inputs.
- fixtures and tools provide real browser workflows and process-tree measurement.
- Linux and macOS currently use CEF's reference UI. Their native runtime acceptance is pending.

## Architecture gate

CEF 156.0.30 and cef-rs 156.6.0 are the current feasibility foundation. Each native-hosted Chrome child has a separate underlying Chromium browser window. The actual MV3 fixture observed two different window IDs and two active tabs within one visible shell window. This mismatch is recorded in evidence/windows-x64/flows.json. It affects tabs, groups, action popups and side panels. Resolve it before declaring general Chrome-extension support. Investigate a CEF patch or maintained Chromium fork against the same tests if necessary.

Real uBOL DNR filtering was tested. Proton Pass reached signed-out onboarding in the reference UI. Sign-in, locking, autofill, save prompts and frames remain unverified. The small API fixture proves only its exercised APIs. Claude officially targets Google Chrome. Provider restrictions are independent of engine API support.

## Build and validate

Use rust-toolchain.toml and Cargo.lock. On Windows, install MSVC C++ desktop tools and Windows SDK, CMake and Ninja. Keep the CEF build root short. Run tools/build-windows.ps1 -BuildRoot C:/db-build. Launch the staged desktop_browser.exe sandbox bootstrap with all its adjacent runtime files. Never launch the library directly.

Run cargo fmt --all -- --check and cargo test --release --locked --no-default-features --test policy --test import --test cookies. Then run the README's smoke, settings, capability and transition checks through the staged browser. Use Node 22+ and Windows PowerShell for these scripts. Keep test profiles separate with BROWSER_TEST_HOME. Repeat memory measurements with release builds and equivalent live workloads.

The recorded two-tab baseline is 138.56 MiB median private working set and 226.93 MiB median private commit over the full Windows process tree. Shared pages and external GPU memory are excluded. These figures are a baseline, never a claim of universal superiority.

The user observed approximately equal memory to Chrome with substantially fewer features. Read docs/foundation-reassessment.md. No matched result establishes a benefit from CEF or an engine switch to Firefox. Keep this product concern central to further architecture decisions. Firefox is being evaluated as an extension-capable alternative, with no approved engine migration.

## Invariants

- Preserve TLS validation, sandboxing, origin isolation and browser/site/extension privilege separation. Never add no-sandbox or remote debugging to make the app pass.
- Keep first-party ordinary logins persistent. Private history, storage and automatic restore must stay out of ordinary records. Optional extensions start disabled in private browsing.
- Run native blocking and extension-blocker compatibility in separate lanes.
- Keep uncertain unsaved work loaded. Shell auto-discard is disabled. Add opt-in discard only after protected-work tests pass.
- Use atomic bounded writes for profile data. Avoid overwriting unreadable existing data.
- Keep native content hooks before network sends. Report cosmetic, scriptlet, cache and request-path gaps.
- Record the host, engine, configuration, exact supported feature and evidence. Configured CI does not prove a native runtime passed.
- Avoid cloud sync, accounts, built-in AI, elaborate workspaces and the marketing website during feasibility.
- Preserve existing user work and personal profiles. Use controlled fixtures and dedicated test accounts.
- Do not publish binaries as production-ready or use an AI fixture as proof a commercial extension works.

## Open priorities

The user requested a checkpoint and continuation prompt. Read docs/continue-tomorrow.md before further changes. Source for themes, broader settings, middle-click close, rounded menus and JSON cookie import is committed but not fully accepted. The final hover regression still fails on rapid caption-button handoff. The Settings and foreground tab-paint checks pass. Six release Rust tests and the Windows release library build pass. The logo PNG is a draft, not integrated. The newer follow-up baseline is 156.29 MiB private working set / 261.23 MiB private commit and predates the latest feature changes. No installer was built. Do not claim these unfinished items are validated.

Consult docs/plan-alignment.md for the section-by-section audit. Resolve extension tab topology first, then complete safe session/crash recovery and cross-platform shell acceptance. Partitioning, fingerprint policy, DNS/proxy/WebRTC, background egress, generic cosmetics, verified list updates, accessibility, IME and real-site workflows remain release gates. Source publication and installable signed releases are separate milestones.

Keep this context and the checkpoint current when behavior or validation changes. Preserve decisions and tests that reveal unsupported requirements.

The latest Windows UI uses explicit Arial and GDI+ geometric icons. Keep icons independent of installed symbol-font mappings. The tab row includes minimize/maximize/close and caption hit testing. The shell address dropdown reads a bounded local recent-URL cache and bookmarks. Typing alone must not trigger provider autocomplete. The package/executable/profile identifiers retain their original working names for compatibility.
