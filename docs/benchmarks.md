# Windows release baseline

The November 16, 2026 Windows x64 run used the staged release binary, CEF 156.0.30 / Chromium 156.0.8078.96, an AMD Ryzen 5 5600, 32 GiB of installed RAM and Windows build 22631. Both local reading/form documents were live. The top layout had a 1104 × 642 CSS-pixel viewport at 1× scale. Native filtering used both bundled lists. No extensions or shell-initiated discards were enabled. The profile started empty, then warmed within the run.

| Two-tab workload | Median of 10 samples | Highest observed sample |
| --- | ---: | ---: |
| Private working set | 138.56 MiB | 139.09 MiB |
| Private commit | 226.93 MiB | 227.60 MiB |

The tree contained the browser, GPU, two utility processes and three renderers. Private working set measures resident pages private to these processes. Private commit includes private allocations backed by physical memory or the paging file. Shared engine pages and external OS/GPU allocations are outside these totals. Chromium can share services among tabs, so these numbers support whole-tree attribution only.

The first tab reported first contentful paint at 372 ms from navigation start. The process-launch-to-server-report interval was 984 ms and includes load, the fixture’s deliberate 350 ms wait and the harness poll. Each WMI sweep cost 432–1,971 ms, in addition to the requested one-second interval. Early or brief allocation peaks can fall between samples. The raw per-process counters also include cumulative CPU time. [Run metadata](../evidence/windows-x64/benchmark.json) · [Raw process samples](../evidence/windows-x64/memory.json)

## Reproduce

Run the smoke workflow first. Then run `node tools/benchmark.mjs <staged-exe> <output-directory>`. The harness starts a loopback server on a fresh port, launches the release binary with a fresh profile and opens the two documents through native navigation controls. It waits six seconds after both documents report before sampling. The sampler walks parent IDs from the browser PID and checks that every discovered browser process has counters.

Use `node tools/growth.mjs <staged-exe> <output-directory>` for repeated creation, navigation and closure of 12 tabs. It preserves one anchor document and its draft, and records whole-tree checkpoints after cycles 0, 4, 8 and 12. Browser caches, allocators and reusable renderers can retain memory after a close. A short run cannot establish absence of leaks.

The new-tab transition test switches from a light article to the dark local new-tab page. It captured 65 Windows window-surface samples over about 2.17 seconds at 54 viewport points each. Both endpoint colors appeared. Zero samples were near-solid black or white at the defined 51-of-54 threshold. PrintWindow samples the window on demand. A display compositor, HDR path or different GPU can present an intervening frame. [Transition samples](../evidence/windows-x64/new-tab-transition.json)

## Next comparisons

Record a corpus revision, URLs or recorded fixture hashes, scenario, application version, engine version, scaling, DPI, viewport, warm/cold caches, extension versions, blocking mode, and memory-saver / discard policy. Test equal active-page counts and confirm JS, requests, video, calls and drafts before comparing memory. Add form editing, scrolling, warm tab-switch latency, CPU during sustained media, cold-paint peaks, list-replacement peaks and multi-hour growth. Run unblocked, native-filtered and extension-filtered lanes independently. No competitor ranking or universal RAM target follows from this run.

On Linux, collect process-tree PSS from smaps_rollup. On macOS, collect physical footprint with a consistent process-tree policy. Preserve their native units and semantics when reporting across platforms.
