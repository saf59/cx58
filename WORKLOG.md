# Worklog

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
