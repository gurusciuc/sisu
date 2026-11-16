Low-memory Rust browser — product and engineering plan

Planning draft, 16 November 2026. Target: Windows, macOS, and Linux. This revision incorporates the requirements for usability, privacy, integrated blocking, extensions, and open-source distribution. Only planning is authorized at this stage; browser implementation and publishing have not started.

1. Build an everyday browser whose memory savings survive normal use.

   The product goal is the lowest measured memory footprint we can achieve while maintaining a good browsing experience. A small number achieved by breaking sites, removing expected features, discarding active work, or weakening security is not a successful result.

   Confirmed requirements are a Rust-based application, all three desktop platforms, strong privacy by default, integrated ad and tracker blocking, selected Chrome-extension compatibility, and public open-source distribution on GitHub with installable releases.

   Keep normal logins, drafts, calls, playback, navigation, text input, downloads, accessibility, and responsiveness working. Privacy should be strong in normal use; a stricter optional profile can trade convenience for additional protections. Claims about privacy and memory must identify the tested configuration and limitations.

   The earlier 300 MiB figure is a workload-specific stretch target, not a universal promise. Establish budgets using actual measurements, including active pages and browser helper processes. Claiming the absolute lowest RAM use of every browser on every site would require evidence we do not have.

2. Use a mature engine for the first product; evaluate a custom engine separately.

   Recommended first-release architecture: a lean Rust browser shell, a mature browser engine, native content blocking, explicit resource policies, and an extension host that supports the selected extensions.

   A Chromium-based browser platform is the leading candidate because of the requested Chrome extensions and mainstream web compatibility. Embedding a Chromium renderer alone does not provide every browser extension feature. The feasibility prototype must validate the complete extension platform and UI integration before selecting a particular embedding framework or maintaining a Chromium-based browser build.

   Wry remains a useful baseline for comparing system webviews, but it is not the assumed solution for uniform Chrome-extension support on all desktop platforms. Servo remains a Rust-engine comparison and research candidate. We will not assume either is smaller or sufficiently compatible without testing.

   A custom Rust renderer remains a separate research track. It can become the product engine only after it meets the same compatibility, responsiveness, security, privacy, and extension gates. This keeps the first usable browser independent of the much longer task of implementing the web platform.

   Do not load two complete engines by default. A dual-engine mode would require a separate justification and memory report; extensions running in one engine do not automatically gain access to pages in another.

3. Prove engine and extension feasibility before investing in the interface.

   Build a small release-mode harness for the leading engine candidate. Test page loading, resizing, focus, clipboard, IME, menus, popup windows, downloads, persistent profiles, private sessions, permissions, crashes, and teardown on native Windows, macOS, and Linux systems. Test Linux with both X11 and Wayland.

   Install the actual chosen extension packages in the laboratory prototype and test complete workflows. Loading a manifest or displaying an icon is not evidence that an extension works. Establish package installation, integrity checks, updates, and any store-distribution constraints separately from API compatibility.

   | Extension target | Required validation | Planning status |
   | --- | --- | --- |
   | uBlock Origin Lite | Manifest V3 lifecycle, request rules, filtering levels, scripts, popup and settings | Minimum extension compatibility target; native blocking is also included |
   | Proton Pass | Sign-in, locking, autofill, save prompts, dynamic forms and frames, relevant privileged APIs | Required compatibility investigation using the actual package |
   | Translation | Selection and page translation, scripts, permissions, popup and requests | Choose the exact extension before declaring support |
   | ChatGPT extensions | Exact package APIs, sign-in, side panel or page integration, permissions | Product names vary; specific packages remain to be selected |
   | Claude in Chrome | Side panel, scripting, debugger/control APIs, tabs and groups, provider support | High-risk compatibility target, not a promised first-release capability |
   | VPN extensions | Proxy configuration, authentication, browser requests, DNS behavior, WebRTC policy, reconnect/failure handling | Select providers and validate actual routing, not only their connected badge |

   uBlock Origin Lite is the accepted minimum; full uBlock Origin is not a first-release requirement. The Lite project targets Manifest V3. Full uBlock Origin uses a different extension model and would need a separate compatibility commitment.

   Anthropic currently documents official Claude in Chrome support for Google Chrome, and explicitly does not support other Chromium-based browsers. Its requested APIs include debugger, sidePanel, scripting and others. We cannot promise official support simply by choosing Chromium. Any native AI sidebar would be a distinct feature, not proof that the official extension works.

   A VPN extension and a device-wide VPN are different products. For example, Proton documents its browser extension as protecting browser traffic. Browser routing and leak tests must match the supported scope.

   Deliverable: supported API and extension matrix, platform findings, architecture recommendation, and initial memory results. Resolve major incompatibilities here, before a polished shell makes changing engines expensive.

4. Measure memory fairly and continuously.

   Build a reproducible corpus covering an article, documentation, an image gallery, a large document, forms, a long scrolling page, a scripted app, video, and a call. Add real-world sites and the selected extensions. Preserve reproducible local fixtures for regression testing.

   Run three distinct comparisons: equal active-page workloads for engine costs; realistic tab workflows for lifecycle policies; and blocking on/off for the effect of ads and trackers. Document competitor memory-saver settings, extension configurations, enabled content, viewport, display scaling, profile state, and engine versions.

   Record startup, steady state, loading and scrolling peaks, repeated navigation, tab closure and discard recovery, and extended use. Compare equivalent output and successful interactions. Do not attribute a gain from blocked content or sleeping tabs to a Rust language rewrite.

   Measure all attributable browser, renderer and helper processes, rather than just the Rust executable. Avoid counting shared memory repeatedly. Report macOS physical footprint, Windows private working set and commit, and Linux PSS separately, with graphics attribution and uncertainty where appropriate. Shared engine processes may prevent exact per-tab or per-extension attribution; label estimates.

   Record CPU use, startup, first paint, interaction latency, playback/call behavior and compatibility alongside RAM. A lower RAM figure that produces substantial stuttering, network reloads, or broken pages fails the product goal.

   Deliverable: published benchmark scripts, fixture corpus, raw results and a readable report. Set numerical release budgets after the feasibility baseline. Investigate sustained unexplained growth at a fixed workload and add regression thresholds calibrated to the engine's warmed caches.

5. Keep the browser's own architecture small.

   | Component | Responsibility | Footprint approach |
   | --- | --- | --- |
   | Rust core | Tabs, windows, commands, navigation and settings | Small records, explicit ownership, bounded queues |
   | Engine adapter | Content views, events, permissions, profiles and crashes | Shared environments where isolation permits; explicit lifetimes |
   | Interface | Tabs, address/search, navigation, panels and dialogs | Compiled controls; benchmark toolkit cost; avoid an additional web runtime just for controls where practical |
   | Content blocker | Request filtering, cosmetic filtering and resource replacements | Native Rust implementation with shared compiled lists and measured caches |
   | Extension host | Packages, APIs, permissions, UI and lifecycle | Reuse the mature platform where feasible; load on demand where semantics allow |
   | Privacy policy | Site isolation, storage partitioning, exposure rules and network behavior | Central rules and regression tests, implemented at effective engine boundaries |
   | Persistence | Bookmarks, history, permissions and session metadata | SQLite and bounded memory indexes; debounced writes |
   | Diagnostics | Memory categories, process attribution, timing and failures | Local by default; detailed tracing only when requested |

   Avoid a large frontend framework or extra background runtime unless it earns its cost. Reuse browser-engine networking, codecs and security primitives where suitable. Benchmark compiled UI candidates inside the feasibility harness; native integration must still provide text input, accessibility and correct focus.

   Introduce platform-specific engine code when necessary for correct privacy or functionality. A convenient abstraction is not a reason to omit required behavior. Record which features need direct APIs or maintained engine patches and the ongoing maintenance cost.

6. Integrate native ad and tracker blocking.

   The browser blocks ads and trackers out of the box without requiring an extension. Evaluate Brave's adblock-rust library, which implements network blocking, cosmetic filtering, resource replacements and uBlock-style syntax extensions. Reusing it does not automatically provide every uBlock Origin feature.

   Apply network rules before disallowed requests are sent, covering frames, redirects and the relevant request paths. Implement cosmetic filtering so blocked advertising does not leave unnecessary page gaps. Integrate vetted resource replacements or scriptlets only through the appropriate controlled mechanisms.

   Use a small maintained default list set, with EasyList and EasyPrivacy as candidates, and add regional coverage deliberately. Verify list updates, retain a usable previous version on failure, bound parsing and matching allocations, and test malformed input. Share immutable compiled data where isolation allows it; measure the temporary peak during list replacement.

   Provide a per-site control and a clear way to recover a broken page. Default blocking should be strong and usable; more aggressive rules can be optional. Test login, checkout, media and forms, not merely an ad-block test page. Keep extension-based blocking as a compatibility option rather than automatically running two overlapping blockers.

   Publish exactly which filtering features work. Avoid promises such as blocking every advertisement on every site.

7. Make strong privacy the default while preserving everyday use.

   Core requirements: no browsing telemetry or automatic page-content uploads; HTTPS by default with clear handling of insecure pages; isolated profiles; partitioned third-party storage and relevant caches; tracker blocking; conservative permissions; minimized unnecessary background connections; and privacy-preserving network and DNS behavior consistent with the user's VPN/proxy configuration.

   Preserve ordinary first-party logins and sessions in the everyday profile. A separate strict private profile uses ephemeral site data and a more standardized configuration. Bookmarks and explicit browser preferences can persist there; private history and automatic page restoration do not. Do not put private browsing records into the ordinary session database.

   Provide script-control and stricter protections as intentional choices rather than disabling JavaScript throughout the everyday profile. Avoid speculative connections to sites before user intent when they reveal browsing information. Search suggestions that transmit typing are optional. Review update, certificate, filter-list and other background requests and publish their purpose and destinations.

   Reduce fingerprinting through coherent policies for fonts, language, timezone, screen and hardware exposure, timing, canvas and graphics APIs. Study standardization and appropriately scoped randomization rather than spoofing unrelated values inconsistently. Test their effects on text, maps, media, calls and rendering. Letterboxing and aggressive restrictions belong in the strict-profile evaluation because they can affect normal use.

   A custom engine can expose a distinctive fingerprint through behavior even if reported attributes resemble another browser. A small new user population cannot be assumed to have Mullvad's anonymity properties. Engine choice, API exposure and fingerprinting resistance need independent evaluation.

   Extra extensions remain available in the everyday profile with explicit permissions. The strict profile begins with a consistent built-in configuration and no optional third-party extensions enabled. A user can opt into changes, but those configurations receive their own published test results. Mullvad itself cautions that extra extensions can affect fingerprinting.

   AI and translation services can receive content when the user authorizes their features. State those flows accurately. Keep native blocking and core privacy protections fully operational without accounts or AI services.

8. Publish evidence for privacy rather than a guaranteed maximum score.

   Use PrivacyTests.org as an initial automated reference and Mullvad Browser as a comparison configuration. Run the tests against defaults, the strict profile, and the extension-enabled configuration separately. Include storage and cache isolation, tracking channels, fingerprint surfaces, and network leakage. Add controlled tests for DNS, WebRTC, proxy bypass, restart persistence and extension permissions.

   Adapt test automation to the selected engine without changing the tested protections. Publish browser and test versions, operating system, configuration, raw results, failures and unsupported features. Missing APIs must be labeled; they must not be presented as proof of a fully compatible browser's protection. Passing every measured case does not establish protection against every tracking technique.

   If the browser has its own privacy panel, show concrete protections and permission state. If a numerical score is added, publish its formula, coverage and limitations, and distinguish it from external test results. Do not display an unearned 100/100 or imply official inclusion on an external site before it happens.

   Invite external review before making strong equivalence claims about Mullvad. Our goal is comparable tested protections with a documented usability tradeoff, rather than an unsupported claim of perfect anonymity.

9. Manage memory without interrupting work.

   First optimize the shell, duplicate environments, leaks, queues, caches, image handling and blocked content. Tab discarding is an additional tool rather than the sole source of savings.

   Retain a small measured set of recently used pages so ordinary tab switching stays immediate. Start automatic discarding as opt-in while validating protection rules. Protect visible tabs, drafts where detected, active calls/capture, playback and relevant downloads. Support an explicit keep-awake override. Pinning a tab and keeping its page loaded are separate choices.

   No generic detector can recognize every application's unsaved state. Treat uncertain cases conservatively and clearly explain the behavior. Do not promise arbitrary in-page state survives discarding. A memory budget is soft when preserving work requires it, and a process count is not the same as a tab count.

   Restore ordinary sessions lazily instead of opening all pages at startup. Bound session metadata, closed-tab records and image thumbnails. Reuse engine navigation state where supported; do not claim that a saved URL list preserves a complete live-page history.

   Prefer responsive policy changes and safe cache eviction over an arbitrary hard RAM limit that causes crashes. Measure switching latency, reload frequency and restored-work failures together with the savings.

10. Do not shortcut browser fundamentals.

    Release gates include correct page rendering and JavaScript for the supported web workload; maintained TLS and certificate validation; sandboxing and origin boundaries; isolation between extensions, sites and browser privileges; safe permissions; reliable downloads and file handling; stable profiles; keyboard and IME input; clipboard; accessibility; media and calls; and crash recovery.

    Preserve the engine's necessary security processes. Verify extension content-script isolation and privileged APIs, not just whether scripts execute. Keep engine security updates and dependency maintenance in the release process. A Rust shell does not make all embedded code memory-safe or remove the need for updates.

    Add cancellation, bounded input handling and resource-pressure recovery. A broken renderer should not terminate the entire browser or corrupt user data. Avoid reducing the headline number by disabling sandboxing, suppressing content silently, or omitting engine helper processes from measurements.

11. Keep the first release focused on complete everyday workflows.

    Include tabs and windows; an address/search field; back/forward; reload/stop; titles and loading state; find and zoom; bookmarks and history; ordinary session restore; private browsing; downloads; permissions; native blocking controls; the validated extension manager and necessary extension UI; and basic memory/privacy diagnostics.

    Validate a set of everyday sites and workflows covering search, email, documents, login and password autofill, video and calls, translation, downloads and checkout using test accounts or controlled fixtures. Publish limitations for unsupported capabilities, including any provider-dependent features or DRM requirements.

    Keep cloud sync, accounts, elaborate workspaces, custom themes, a proprietary extension store and integrated AI out of the first-release critical path. Supporting chosen AI extensions remains a distinct compatibility target. Add optional features only after their memory, privacy and maintenance cost is measured.

    The browser uses a familiar top tab strip by default, with conventional address and navigation controls. Users can choose vertical tabs on the left; the layout system should also accommodate right and bottom placement. Tab placement is a core customization requirement and is separate from optional themes and elaborate workspaces. Prove top and left layouts before locking the UI toolkit, and share one tab model across all placements.

    Switching layouts preserves order, selection, navigation and live page state without reloading pages or losing drafts. Persist the appropriate profile preference. Include keyboard access, focus indication, accessibility, display scaling, overflow behavior, and an adjustable/compact vertical strip. Reuse views and controls to keep customization inexpensive. Test viewport/fingerprinting effects rather than assuming interface customization cannot be detected by websites.

    The presentation website is a later deliverable. The user's design references are [Ardia](https://ardia.dev/) and [ArtCraft](https://getartcraft.com/). Review their visual direction when website work begins and develop an original design informed by those preferences. Do not build the website during the engine feasibility milestone or infer that its references require an Arc-style browser sidebar by default.

12. Deliver open-source code and installable desktop releases.

    Plan a public GitHub repository with an explicit project license, dependency and engine notices, reproducible development instructions, architecture decisions, a roadmap, issue templates, contribution guidance, a security-reporting channel and benchmark documentation. Resolve dependency and bundled-resource licensing before public distribution.

    Use automated checks and builds on Windows, macOS and Linux. Test native behavior rather than relying on cross-compilation alone. Initial architecture targets are Windows x64, macOS Apple silicon and Intel, and Linux x64; finalize minimum OS versions and distributions after the platform prototype.

    Publish versioned release binaries and checksums, with Windows signing and macOS signing/notarization planned for normal installation. Select a Linux distribution format after verifying graphics, sandboxing, fonts and packaging dependencies. Provide an update path that verifies integrity and can recover from a bad update.

    GitHub source availability and downloadable installers are separate deliverables. No repository creation or public publishing happens during this planning stage.

13. Execute in stages with clear exit gates.

    | Stage | Deliverable | Required evidence |
    | --- | --- | --- |
    | A. Requirements and feasibility | Engine harness, extension/API matrix, platform checks, initial privacy boundaries | No unresolved blocker hidden behind a generic compatibility claim |
    | B. Baseline and foundation | Reproducible workloads, Rust core, release-mode measurements and UI toolkit decision | Full-process memory report and a working top/left tab-layout switch preserving page state |
    | C. Everyday browser alpha | Navigation, windows/tabs, profiles, storage, downloads and validated extension UI | End-to-end everyday workflows on every supported OS |
    | D. Blocking and privacy alpha | Native blocker, partitioning, permissions, private profile and background-request policy | Functional blocking and privacy tests alongside site-breakage results |
    | E. Memory tuning | Bounded caches, environment reuse, safe lifecycle policy and diagnostics | Lower RAM at matched functionality without unacceptable reloads or latency |
    | F. Hardened beta | Crash recovery, accessibility, security/update processes and broader site matrix | Long-session, protected-work, native-platform and privacy regression tests |
    | G. Open-source release | Repository, documentation, installers, checksums and repeatable builds | Published scope, limitations and independently reproducible results |

    Privacy, security, memory measurement and compatibility constraints begin in stage A and remain active in every stage; their named alpha stages add broader user-facing implementation. Calibrate performance thresholds and schedule after the feasibility results instead of promising dates before the integration risks are known.

    Any architecture change is reviewed against the same priorities: preserve expected functionality and security, deliver strong tested privacy, and reduce memory where it improves the product. If a low-memory technique causes recurring data loss, major site breakage, heavy CPU use or noticeable delays, revise or remove it.

14. Keep a separate custom Rust engine research roadmap.

    Build a headless document renderer, then an interactive cross-platform harness, then richer CSS and a measured JavaScript VM integration. Reuse mature parsers, TLS, fonts and codecs where appropriate. Candidate parsing components include html5ever and cssparser; a Rust VM candidate is Boa. These are components, not complete browser platforms.

    Experiment with compact index-based DOM arenas, shared immutable styles, explicit scratch-buffer lifetimes, incremental layout, bounded image/glyph caches and tiled painting. Account for cycles between JavaScript and DOM objects, graphics surfaces, layout correctness and script execution cost. Rust ownership does not eliminate JavaScript garbage collection.

    Include selected standards tests, visual reference tests, parser fuzzing, lifetime and teardown stress tests, accessibility, isolation and the selected WebExtensions APIs. Script engines do not automatically provide DOM, service workers, browser APIs or extension execution worlds. A custom extension host is a major project of its own.

    Measure research results separately while the feature set is smaller. Replace the product engine only after representative sites, selected extensions, input/media, security, privacy and memory gates pass. A renderer that draws basic pages is a useful research result but is not yet an everyday replacement.

Reference material informing this plan:

- [WebKit memory categories](https://webkit.org/blog/6425/memory-debugging-with-web-inspector/).
- [V8 pointer compression](https://v8.dev/blog/pointer-compression) and [memory optimization](https://v8.dev/blog/optimizing-v8-memory).
- [Chromium process architecture](https://www.chromium.org/developers/design-documents/multi-process-architecture/) and [WebView2 process sharing](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-model).
- [Chrome extension content scripts](https://developer.chrome.com/docs/extensions/develop/concepts/content-scripts), [extension service workers](https://developer.chrome.com/docs/extensions/develop/concepts/service-workers), and [declarative request rules](https://developer.chrome.com/docs/extensions/reference/api/declarativeNetRequest).
- [Mozilla's Chrome-extension porting guidance](https://extensionworkshop.com/documentation/develop/porting-a-google-chrome-extension/).
- [uBlock Origin Lite](https://github.com/uBlockOrigin/uBOL-home) and [Brave's native Rust blocker](https://github.com/brave/adblock-rust).
- [Proton Pass extension behavior](https://proton.me/support/pass-browser-extension) and [Proton VPN extension scope](https://protonvpn.com/support/browser-extensions).
- [Claude in Chrome support and permissions](https://support.claude.com/en/articles/12012173-get-started-with-claude-in-chrome).
- [Mullvad Browser's privacy design](https://mullvad.net/en/browser/hard-facts) and [extension/fingerprinting guidance](https://mullvad.net/en/help/faq).
- [PrivacyTests.org methodology](https://privacytests.org/about.html).
- [Servo](https://servo.org/), [SpiderMonkey](https://spidermonkey.dev/), [Boa](https://boajs.dev/), [html5ever](https://github.com/servo/html5ever), and [cssparser](https://github.com/servo/rust-cssparser).
