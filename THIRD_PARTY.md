# Third-party software and data

Desktop Browser contributors license the original shell, browser policy code, tests and documentation under Apache License 2.0. The runtime directory includes LICENSE, NOTICE, this inventory and notices/. The browser relies on the fonts installed on the host. No font files ship in the preview.

| Component | Version in the Windows preview | Terms and attribution |
| --- | --- | --- |
| Chromium Embedded Framework | 156.0.30+g6028375 | CEF Authors, BSD-style license in notices/CEF.txt. Upstream's runtime CREDITS.html contains Chromium and bundled component notices. |
| Chromium | 156.0.8078.96 | Bundled by the official CEF binary distribution. Consult the accompanying CREDITS.html. |
| cef Rust bindings | 156.6.0 | MIT or Apache-2.0. The sandbox entry points and Cocoa helpers adapt cef-rs example code. See notices/cef-rs-MIT.txt. |
| adblock-rust | 0.13.5 | Brave Software and contributors, MPL-2.0. Unmodified crate source is available from crates.io. See notices/adblock-MPL-2.0.txt. |
| EasyList and EasyPrivacy | 202611161103 | The EasyList authors. The bundled source lists and their compiled representation use CC BY-SA 3.0. The project compiles the lists and includes separate local fixture rules. See notices/EasyList-CC-BY-SA-3.0.txt. |
| Other Rust crates | Cargo.lock | Package versions, declared licenses, source locations and available upstream notice texts are in notices/rust-dependencies.txt. Regenerate with `node tools/cargo-notices.mjs`. |

The extension packages used for compatibility testing are downloaded separately. Their publishers retain their respective rights. Product and vendor names identify the tested software.

[CEF](https://github.com/chromiumembedded/cef) · [cef-rs](https://github.com/tauri-apps/cef-rs) · [adblock-rust source](https://crates.io/crates/adblock/0.13.5) · [EasyList license](https://easylist.to/pages/licence.html)
