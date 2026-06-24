# CX58 Reliability Plan

Date: 2026-06-24

Scope: `gmr` first, then `cx58-agent`.

Related follow-up: `D:\projects\rust\cx\cx58-admin` also calls `cx58-agent`.
Do not add admin HMAC changes in this iteration; record and handle them later
as a separate boundary-hardening pass.

## Accepted Now

1. Close the service boundary between `gmr` and `cx58-agent`.
   - HMAC-sign every non-health agent route used by the UI.
   - Keep health public.
   - Centralize signing in the UI proxy layer.
   - During the admin migration window, agent routes shared with `cx58-admin`
     may accept unsigned legacy requests with warnings; invalid signed requests
     must still be rejected.

2. Keep SSE behavior explicit and testable.
   - `Completed` must remain observable after successful agent flows.
   - Error and cancel terminal semantics must be consistent between agent and UI.

3. Improve report operation reliability.
   - Upload/update/delete errors must surface to the operator.
   - Berlin-time input behavior must be explicit.
   - Storage and DB consistency risks should be reduced in the agent.

4. Improve observable health.
   - DB, S3, and Ollama health should not hide failures.
   - Request IDs and boundary failures should be visible in logs.

## Deferred Until Pre-Production

1. Replace in-memory `gmr` OIDC sessions with persistent/distributed storage.
2. Remove or gate `danger_accept_invalid_certs(true)` for production TLS.

## Deferred Follow-Up

1. Audit and HMAC-sign `cx58-admin` calls to `cx58-agent` in a separate pass.
   Admin has its own proxy/backend contract, so it should not be changed as part
   of the current `gmr` demo-hardening patch.

## Guardrails

- Preserve local env/operator files and dirty worktrees.
- Keep user-facing strings localized.
- Keep code comments and code-facing docs in English.
- Do not broaden the auth crate extraction yet; first stabilize the actual contracts.

## Completed In This Pass

1. `gmr` now HMAC-signs the UI proxy calls for agent tree, report upload,
   report delete, and chat cancel.
2. `cx58-agent` now protects tree, upload, delete, and cancel routes with
   temporary compatibility HMAC: unsigned legacy requests are warned and
   accepted during the admin migration window; invalid signed requests are
   rejected.
3. Agent SSE handling no longer stops before the top-level `Completed` event
   after an upstream stream error.
4. Report delete errors now surface in the `gmr` reports panel instead of being
   ignored.
5. HMAC middleware now bounds request body buffering to 30 MiB.
6. Agent S3 health now fails closed on S3 list errors instead of hiding them.
7. `gmr` token refresh now stores the real ID-token expiry time after refresh,
   not the next refresh threshold time.
8. `gmr` login/logout session cookies now use the configured cookie policy, and
   logout still clears the cookie when external OIDC logout URL construction
   falls back.
9. `cx58-agent` upload now removes the uploaded S3 object if the DB insert
   fails, reducing orphaned storage after partial failures.
10. `cx58-agent` image delete now removes the DB row first and then performs
   best-effort S3 cleanup with structured error logs for cleanup failures.
11. `cx58-agent` chat session loading now logs DB failures instead of silently
   treating them as a missing session.
12. `gmr` report upload defaults now generate `berlin_datetime` in
   Europe/Berlin time instead of the browser's local timezone.
13. `cx58-agent` chat sessions now use a `(user_id, chat_id)` primary key via
   migration, matching the existing load contract and preventing one user's
   chats from overwriting each other.
14. `cx58-agent` session saves without new history now preserve existing
   history instead of replacing it with an empty array.
15. `cx58-agent` Ollama health checks now use a 3-second HTTP timeout.
16. `cx58-agent` now has focused unit coverage for HMAC verification success,
   tampered body rejection, stale timestamp rejection, and missing header
   rejection.
17. `cx58-agent` now has focused unit coverage for chat history truncation and
   malformed history entry handling.
