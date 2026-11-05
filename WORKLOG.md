# Worklog

## 2026-09-14: Rust 1.98.1 dependency-update build repair

The updated dependency graph resolves wasm-bindgen 0.2.128 and web-sys 0.3.105.
Three independent build failures were reproduced:

- sccache 0.17.0 fails to spawn rustc for web-sys on Windows with os error 206 (command line too long), including with a fresh server.
- The installed wasm-bindgen CLI 0.2.126 rejects the WASM schema produced by wasm-bindgen 0.2.128.
- The global Cargo build target is Windows MSVC, but cargo-leptos expects target/debug/gmr.exe when bin-target-triple is omitted.

Local `.cargo/config.toml` disables the inherited rustc wrapper so Cargo can use response files.
Cargo.toml now explicitly selects the Windows server target so cargo-leptos locates the compiled executable.
Docker removes that local target from its copied manifest before compiling for the container host.
Docker retains its explicit sccache wrapper and already selects wasm-bindgen 0.2.128.
The local Cargo configuration is ignored by the existing dotfile rules.
Update the CLI with `cargo install wasm-bindgen-cli --version 0.2.128 --locked --force`.
Application behavior and the user's dependency/Dockerfile changes are preserved.
The CLI was successfully upgraded to 0.2.128.
Validation: full `cargo leptos build` passed, including client compilation, JS/WASM generation, styles, and server compilation.
Evidence: `target/gmr-build-final.log` and `target/sccache-diagnostic.log`.
Docker source configuration was reviewed; no image build or deployment was performed.

## Current rule

The reliability audit is accepted for fixes as of 2026-06-24.

Do not include these two audit findings in the current demo-hardening work:

- in-memory UI sessions;
- `danger_accept_invalid_certs(true)` in the UI HTTP client.

Both are intentionally deferred until the later pre-production change set.

Current fix priority:

1. Close the `gmr` to `cx58-agent` service boundary: HMAC/sign every non-health agent route used by the UI.
2. Preserve agent SSE completion behavior.
3. Improve safe operator behavior around report upload/delete/date handling.
4. Keep changes narrow and preserve local operator/env files.

Related follow-up:

- `D:\projects\rust\cx\cx58-admin` also calls `cx58-agent`; audit and HMAC-sign those calls later in a separate admin boundary pass.

Completed in this pass:

- UI proxy signs tree, report upload/delete, and chat cancel agent calls.
- Agent protects shared tree/upload/delete/cancel routes with temporary
  compatibility HMAC for the admin migration window.
- Agent SSE keeps `Completed` observable after upstream stream errors.
- Report delete errors surface in the reports panel.
- HMAC body buffering is bounded to 30 MiB.
- Agent S3 health now reports S3 list failures.
- UI token refresh stores real token expiry, not refresh-threshold time.
- Login/logout session cookies use configured cookie policy.
- Agent upload cleans up S3 when DB insert fails.
- Agent delete removes DB row first and logs S3 cleanup failures.
- Agent session load logs DB failures.
- Report upload defaults use Europe/Berlin time instead of browser-local time.
- Agent chat_session now uses `(user_id, chat_id)` as the primary key.
- Agent session saves preserve existing history when a save has no new history.
- Agent Ollama health check uses a 3-second timeout.
- Agent HMAC verification has focused unit coverage for valid, tampered,
  stale, and missing-header cases.
- Agent chat history has focused unit coverage for truncation and malformed
  entries.
