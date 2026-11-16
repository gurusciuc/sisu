# Foundation reassessment after local feedback

The user's Windows Task Manager screenshot showed CEF Bootstrap Application at 1,109.6 MB and Google Chrome at 1,127.5 MB. CEF showed higher instantaneous CPU usage. That is about a 1.6% memory difference in the displayed numbers. The screenshot does not establish matched profiles, page state, extensions, viewport, memory-saver policy or measurement timing. It supports the user's practical observation that the current preview offers little evident memory advantage while missing many everyday-browser features.

The small release fixture baseline measures a different workload. It cannot establish a benefit over the user's normal Chrome session. Product decisions must give full functionality and comparable active content the same weight as private working set, private commit, CPU and latency.

## What is known

The current custom shell gives each CEF Chrome child a separate underlying Chromium browser window. The extension topology test confirms that behavior. Its incremental memory and CPU cost still needs quantifying against the engine's full native reference window. Rust state records and drawn controls alone cannot remove rendering, JavaScript, networking, media and GPU allocation costs.

Local feedback also identified repeated tab flicker, poor font/glyph rendering and a separate title bar. The updated Windows shell avoids redundant view resizes, selects the surviving page before retiring a closing parent and buffers chrome drawing. It uses Arial, geometric icons and an integrated draggable tab row with native window commands. These changes address usability. They do not demonstrate a large engine-memory reduction.

Address suggestions now use bounded in-memory recent URLs and profile bookmarks. Search input is submitted to the selected provider on commit. Ordinary recent URLs are kept out of private suggestions. The engine's separate network behavior still needs an egress audit.

## Firefox as a candidate

Firefox has a mature extension platform. Proton Pass and uBlock Origin Lite publish Firefox versions. Chrome packages can have incompatible manifest fields, APIs, worker semantics or provider restrictions. Claude in Chrome officially supports Google Chrome only. A Firefox derivative would require maintaining the complete desktop browser platform and the same native-platform acceptance suite.

Both Gecko and Chromium use multiprocess site isolation and allocate page, JS, graphics and media resources. There is no matched measurement establishing that a Firefox foundation would reduce Sisu's memory use. Replacing the engine is an experiment with compatibility and maintenance costs.

[Proton Pass platforms](https://proton.me/support/pass-browser-extension) · [uBOL releases](https://github.com/uBlockOrigin/uBOL-home/releases) · [Chrome-to-Firefox porting](https://extensionworkshop.com/documentation/develop/porting-a-google-chrome-extension/) · [Claude provider policy](https://support.claude.com/en/articles/12012173-get-started-with-claude-in-chrome) · [Gecko architecture](https://firefox-source-docs.mozilla.org/overview/gecko.html) · [Chromium isolation](https://chromium.googlesource.com/chromium/src/+/main/docs/process_model_and_site_isolation.md)

## Decision gate

Preserve the current prototype as evidence. Compare the custom CEF shell, CEF's full reference browser, stock Chromium/Chrome and Firefox on identical active workloads and separately documented blocking modes. Test selected extensions and normal browser workflows as part of that comparison. Choose the next foundation only when the extension requirements and measured costs support it. The user is considering whether Firefox editions of the required extensions are sufficient, and has requested evidence of memory savings before deciding.
