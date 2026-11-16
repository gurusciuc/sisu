# Windows feasibility checkpoint · 16 November 2026

## Follow-up after local testing

The revised local Windows build uses Arial and geometric icons, an integrated draggable tab/caption row, and local address suggestions. Repeated Ctrl+T/Ctrl+W cycles passed in top and left layouts while preserving the anchor document and draft. Caption alignment, dragging hit tests, minimize/maximize/restore, local suggestion selection, recent-URL persistence and private separation have targeted tests. View positioning now skips unchanged geometry, closing selects a survivor before removing the old parent, and chrome painting uses temporary double buffers. Create-and-close window-surface samples contained zero near-solid black/white frames. Hardware presentation remains a separate observation. [Follow-up evidence](../evidence/windows-x64/ui-fix/build.json)

The user observed little memory advantage over Chrome with substantially fewer features. There is no verified matched-workload advantage for the current foundation, and Firefox has no established memory advantage in this project. Extension/window topology and equal-functionality comparisons remain architecture gates. [Foundation reassessment](foundation-reassessment.md)

## Delivered

The runnable x64 preview opens a shared-profile CEF Chrome runtime under a native shell. Browser controls use a restrained dark palette, Helvetica/Arial-family typography and Windows glyphs. Settings opens in a normal tab from the menu, Ctrl+, `#settings` or `browser://settings`. It includes tab placement and width, search provider, startup behavior, bookmark import and a link to engine site permissions. Private preference changes remain in memory.

The same ordered tab model drives the top strip and adjustable left rail. The Windows flow tests change placement, check actual viewport size, confirm document IDs and preserve an unsubmitted draft. They check back/forward destinations, reload, per-site blocking exceptions, disk-backed cookies and local storage, and two independent private launches. [Recorded flows](../evidence/windows-x64/flows.json)

Native adblocking runs before disallowed network requests in the tested paths. Fixture tests cover an allowed script, a cancelled script, a permitted exception, a redirect, a nested frame, dedicated-worker and service-worker fetches, and host-specific cosmetic hiding. A synthetic video frame and a loopback WebRTC data channel worked. Camera, microphone and routed calls require hardware/provider acceptance. [Capability evidence](../evidence/windows-x64/capabilities.json)

A real uBlock Origin Lite package enforced a shipped network rule in an extension-only profile. The actual Proton Pass package reached signed-out onboarding under the engine reference UI. Popups, vault workflows, signing and updates have separate gates. [Extension matrix](extensions.md)

Privacy boundaries, navigation policy and bookmark parser tests run in release mode. The HTML importer accepts Brave, Chrome and Firefox bookmark exports, preserves folders, deduplicates HTTP(S) entries, bounds input and atomically replaces the profile library on success. It does not migrate passwords, cookies, history or whole profiles. Settings has a Windows regression flow for opening, reuse, preference storage and private separation. [Settings results](../evidence/windows-x64/settings.json)

## Fixed during integration

- Closing a Chrome-style child could send a close to the shell root. A retired child now receives an independent teardown root after CEF accepts the close. A regression test checks that closing Settings leaves the original web page alive.
- Switching tabs could expose an unpainted native surface. The incoming local page now gets a matching starting background and staged presentation. The window-sampling test observed zero near-solid black samples in 65 captures. The exact frames shown by a user's display still warrant compositor-level testing. [Raw samples](../evidence/windows-x64/new-tab-transition.json)
- Long MSVC output paths interfered with CEF linking. The build instructions use a short build directory. An early premature parent-window destruction at shutdown was corrected and the browser tests assert a clean process exit.

## Measured

With two live local article/form pages, the release build used a median 138.56 MiB private working set and 226.93 MiB private commit across seven processes. The highest observed samples were 139.09 and 227.60 MiB respectively. [Methodology and raw data](benchmarks.md)

## Configured for other hosts

The GitHub Actions workflow defines Windows, macOS and separate Linux X11/Wayland jobs. Linux and macOS currently use the reference browser UI. Only Windows x64 was built, run and visually inspected locally. Hosted CI results and platform signing have their own acceptance gates. [Build definitions](../.github/workflows/check.yml)

## Open release gates

| Area | Work still required |
| --- | --- |
| Browser and extension platform | Reconcile shell and Chromium tab/window identity. Verify extension actions, permission UX, side panels, tab groups and controlled Proton sign-in/lock/autofill/save flows. Select a CEF patch or a Chromium fork if the topology cannot support them. |
| Everyday usability | Complete lazy multi-tab restore, multi-window coordination, popup interception and crash recovery. Validate real-site checkout, form edits, downloads, permission prompts, clipboard, screen readers, IME, scaling, video codecs and conferencing. |
| Blocking | Generic cosmetics, document-start timing, resource replacements and scriptlets, wider request-path coverage, signed list distribution and rollback. Measure list-swap peaks. |
| Privacy | Confirm partitioned storage/cache behavior and idle egress, perform external privacy tests, measure fingerprint exposure across layouts, DNS/WebRTC/proxy routing and strict-profile behavior. Measure cleanup after abnormal exit. |
| Memory and latency | Match real active content, collect platform-native metrics, multi-hour growth, tab-switch/input latency and sampling overhead. Shell-initiated discarding is disabled. |
| Distribution | License and copyright are in place. Verify notices for the final platform-specific artifacts, enable private security reporting and produce reproducible signed packages and verified updates. |

## Next implementation step

Build a two-tab extension acceptance test that records what `chrome.tabs.query` and `chrome.windows` report and opens an action popup on the selected custom-shell tab. Run that alongside the full engine reference window, then exercise Proton Pass using a dedicated test account. Resolve this architectural difference before investing in further cross-platform shell work.

## Local release preparation

The version is in Cargo.toml. The Windows staging script includes the engine’s runtime credits, Apache license, project NOTICE and upstream licenses. `tools/archive-preview.ps1 -Stage <directory> -Destination <directory>` creates a versioned local preview ZIP and SHA-256 file without publishing.

For a public GitHub release, build from a reviewed tag on native runners, record dependency and binary hashes and build provenance, and run the security and platform suites. Sign Windows binaries with Authenticode. Sign the macOS application and all helpers in dependency order, notarize and staple the package. Exercise Linux packages on real X11 and Wayland desktops with their sandbox configuration. Authenticate future update manifests, including version monotonicity and artifact hashes, with an offline-rooted signing policy. A checksum beside an unsigned archive alone provides no publisher authentication.
