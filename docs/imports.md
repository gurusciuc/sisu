# Browser-data import checkpoint

Bookmarks accept user-selected Brave, Chrome and Firefox HTML exports. The existing importer preserves folders, bounds input, merges duplicates and writes atomically. A native chooser now separates bookmark and cookie formats.

The cookie importer accepts a JSON array, or an object containing a cookies array, with name, value, domain and optional path, secure, httpOnly, hostOnly, session, expirationDate and sameSite fields. Bounds are 8 MiB, 1,000 records and 4 KiB per cookie. It validates the entire input first, rejects unsupported partition scopes, skips expired entries and preserves supported security attributes. Writes use the real CEF profile cookie manager and report individual engine rejections and persistence failures. An accepted import may replace matching cookies. Partial engine acceptance is not transactional rollback.

Cookie files contain sensitive login sessions. Only an explicitly selected export is read. No automatic personal profile access or decryption of Chrome/Brave/Firefox databases is implemented. Browser and OS encryption, partitioned cookie scope and provider-bound sessions prevent a promise of universal session migration. Private windows cannot import cookies. Password and browser-import tools are engine-page links whose practical capabilities remain unverified.

Parser tests pass. Real cookie import, server receipt, HttpOnly isolation, scope, restart and private-session integration tests remain pending. Do not advertise these as validated migration features yet. Follow docs/continue-tomorrow.md.
