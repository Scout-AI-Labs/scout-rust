# Changelog

All notable changes to this project are documented here. This project adheres
to [Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-06-21

Initial release.

- Async client built on `reqwest` + `serde` + `tokio` (minimal, audited core).
- Full coverage of the Scout REST API: `search`, `page`, `extract`, `company`, `lists`, `products`, `site`, `jobs`, `monitors`, `chat`.
- `thiserror`-based `Error` enum with `is_rate_limited`/`is_authentication`/... helpers and `status()`/`request_id()`.
- Automatic retries with exponential backoff + jitter, honoring `Retry-After`.
- Idempotency keys on writes.
- `list_all()` helpers that walk every page.
