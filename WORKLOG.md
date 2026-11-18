# Worklog

## 2026-11-18: Cloud photo description diagnosis

The operator reproduced the empty-description error after deploying
`alsh/cx58a:rc1-cxid-search-20261118` on the Kitchen / k-1 report captured
31 May 2026 at 00:00 Berlin time. Through an SSH tunnel, replayed requests to
Ollama 0.24.0 using the original photo resized to 900 × 1200 and the current
English prompts. The probe used a JPEG re-encode and a reconstructed name
path. No photo or reasoning text was stored by the probe.

The cloud `qwen3-vl:8b` tag uses the `qwen3-vl-thinking` renderer and parser.
With `think: false`, its primary and structured retry requests both ended
with `done_reason=length`, empty `message.content`, and nonempty
`message.thinking`. They consumed 1,024 and 2,048 generation tokens in
16.55 and 31.76 seconds respectively.

The installed `qwen3.5:4b` reports the vision capability. With the primary
request on the same photo, it returned valid description JSON in 4.18 seconds
with 161 generated tokens and `done_reason=stop`. The measured workaround is
to select it under Models → Vision for the affected account. The operator
confirmed that `qwen3.5:4b` works in CX58 and that `qwen3-vl:4b` fails there.
A second direct test of `qwen3.5:4b` returned valid description JSON in
15.93 seconds. The cause of the specific `qwen3-vl:4b` failure is unverified.
The explicit `qwen3-vl:8b-instruct` tag is absent from the cloud Ollama.
Probe summaries are in `F:/Temp/cx58-vision-cloud-20261118/`. Application code,
model settings, and deployment configuration were not changed.

The operator subsequently installed
`hf.co/Qwen/Qwen3-VL-8B-Instruct-GGUF:Q4_K_M` on Ollama 0.24.0. The agent
log at 14:25:46 UTC showed that Ollama returned HTTP 500 because that model's
runner stopped. The CX ID lookups surrounding the failure returned HTTP 200.
Direct Ollama requests with `num_ctx=4096` reproduced the runner failure
both with the original photo and with a short text-only prompt. The runner
exit cause remains unknown because Ollama container logs are unavailable
through the SSH gateway. The already installed `qwen3.5:4b` remains the
verified option for the affected photo.

## 2026-11-18: CX ID fallback in chat

Object and explicit photo-report lookup first matches display names. If there
is no name match, it looks up the same identifier as a CX ID in the accessible
tree. An explicitly named report scopes the response to that photo. Report
lookup uses the selected or named object when available and otherwise resolves
the parent from a unique accessible match. Existing date and period selection
still handles requests without explicit report identifiers.

The native agent build passed with 98 library tests passed and 12 external
S3 tests ignored. The signed chat smoke used isolated PostgreSQL and a stub
intent classifier. It verified name precedence on CX ID collisions, object
and report CX ID fallback, exact single- and two-report selection, description
of a report resolved by CX ID, date selection with an object CX ID, duplicate
IDs, and refusal of foreign or out-of-context nodes. The test script is
`F:/Temp/cx58-cxid-search-20261118/search-smoke.py`. Cloud classification
and deployment remain pending.

The operator requested an agent-only Docker build and publication after both
the CX ID search and empty-photo-description fixes passed local validation.
The build uses the captured source in
`F:/Temp/cx58-agent-cxid-search-image-20261118/`. The Linux/amd64 image passed
the disconnected check for the CX ID query, vision retry, migration, and
English and German prompts and messages. Published
`alsh/cx58a:rc1-cxid-search-20261118` to Docker Hub and verified its registry
index digest matches the checked local image:
`sha256:0375a49a6f986cde3aaaac763dd29645cf2c4a19677ca3da5331af7321b2811f`.
Cloud rollout and acceptance remain pending with the operator.

## 2026-11-18: Empty response from the photo description model

The operator deployed the CX ID fix and reported `No content provided` from
the vision request. Rig emits this error when the Ollama assistant message has
no content or tool calls. CX ID writes update only the CX ID, and the vision
request uses the report UUID, image bytes, and the original node name path.
The supplied cloud log does not expose Ollama's stop reason or token counts.

Added one retry for an empty vision response. The retry uses 2,048 generation
tokens and requests a JSON object with a description field. Both attempts
share the existing deadline. A second empty result returns a localized
photo-description error. The previous agent binary reproduced the exact Rig
error and generic client message against an empty Ollama fixture. Agent
library tests passed with 98 passed and 12 external S3 tests ignored. The
structured retry request also returned complete JSON for a local
Qwen3-VL-4B-Instruct sample.

The rebuilt native agent passed the signed `Show descriptions` smoke with
isolated PostgreSQL and Ollama/S3 fixtures. The vision request payloads were
identical before and after the CX ID write. Two empty responses emitted one
localized error and produced no cache row. An empty response followed by JSON
completed normally, stored the description, and produced a cache hit on the
next call. Reproduction is in
`F:/Temp/cx58-empty-description-20261118/show-descriptions-smoke.py`.
The isolated database and local Ollama process were stopped. Cloud validation
on the affected report remains pending. The later agent-only image includes
this fix and the CX ID lookup.

## 2026-11-18: CX ID display and Reports pagination

Moved object and photo-report CX ID editing to Reports. Objects, chat object
trees, and selected chat context display accessible IDs. Added a signed batch
lookup using the session identity and existing tree ownership checks. Reports
shows IDs in left-aligned, width-constrained controls. Moved the last-page
comparison out of the Leptos attribute expression. CX ID errors now distinguish
expired sessions, node access, and upstream authentication or missing routes.
The exact cause of the reported cloud access message is unverified.

The native agent and GMR SSR/WASM builds passed. GMR retains the active report
load and cancels it when a newer load starts. The CX ID lookup also discards
superseded requests. The isolated PostgreSQL smoke verified batch ownership,
inherited access, Unicode round trips, empty IDs, and the report date. Edge
verified the tree and selected-report labels, both Reports editors, ellipsis,
the two error paths, direct URLs, and both directions of 13-report pagination.
All 30 GMR library tests passed. Concurrent editor and batch reads shared one
OIDC refresh after expiry. Failed renewal returned 401 and an upstream HMAC
rejection returned 502.

Reproduction scripts are in
`F:/Temp/cx58-cxid-22a480e01bc84bfe91ea16c2d1c5f410/`
as `gmr_reports_smoke.py`, `agent_cx_id_batch_smoke.py`, and
`cx_id_session_smoke.py`. The isolated database was stopped after the check.
Cloud verification remains pending. Rollout requires both updated services.

### Local Docker build checkpoint

The operator requested local Linux/amd64 builds of both services and deferred
registry publication. Build from the source snapshots in
`F:/Temp/cx58-cxid-images-20261118/` under the `rc1-cxidfix-20261118` tag.
Both Linux/amd64 images are loaded locally. Disconnected container checks
verified the server binaries, CX ID batch routes, the agent migration, and
GMR JavaScript, WASM, and ellipsis styles. The image IDs are recorded in
`F:/Temp/cx58-cxid-images-20261118/image-manifest.json`.

Published both tags to Docker Hub and verified the `linux/amd64` entries.
The registry index digests match the locally checked images.

- `alsh/cx58:rc1-cxidfix-20261118` at `sha256:458e3da63dee22299098e550bd82cbd7d8f19db892afc07164aac32d826c003e`
- `alsh/cx58a:rc1-cxidfix-20261118` at `sha256:3f4988b16439cae382b169050bd8fbca28c2001f86db1fddd45a9b3b1e20d2ed`

Cloud rollout and acceptance are pending with the operator.

## 2026-11-18: Blackout recovery checkpoint

The operator confirmed both cloud services are running after the RC1
publication and database migration. Cloud acceptance is in progress.

Open issue: both PDF and JSON downloads show "This report is unavailable.
Check your session and report access." The UI maps HTTP 401, 403, and 404 to
that string. A schema-valid unsigned POST to the cloud GMR endpoint on 9344
returned 401, and an unsigned POST to `cx58-agent:3050/agent/report-export`
through `gate` also returned 401. Both routes exist. The 9454 GMR probe
returned 502. The Windows browser tool stopped because it could not verify the
active tab URL. The status of the authenticated failing request is still
unknown. Resume from the specific HTTP status of `POST /api/report-export`.
If it comes from the agent, inspect the HMAC outcome and the report ID, node
type, parent, and ownership lookup before changing access logic.

Verified local recovery bundle at
`F:/cargo-target/cx58-recovery/2026-11-18-before-1400/`. It contains the
workspace and Git metadata, the RC1 and prior local images, and the release,
migration, and test artifacts. Compared file hashes, Git HEAD and status for
all five repositories, and Docker archive contents. The saved PostgreSQL
backup was read in full. See `README.md` and `verification.txt` in the bundle.

### Export HTTP 401 follow-up

The operator captured HTTP 401 for the authenticated browser POST on 9344.
GMR returned 401 for expired ID tokens before trying the stored refresh token.
Chat used the same agent HMAC code but required only a cookie, so an open
chat could still produce a result after its session expired. This is a
locally reproducible code path. The source of the exact cloud 401 remains
unconfirmed. Updated GMR to refresh expired sessions with a usable refresh
token, serialize concurrent refreshes, distinguish session errors from report
access errors, and log an upstream HMAC rejection separately. The old local
binary reproduced 401 without attempting refresh. The updated GMR build passed
the stub OIDC regression: two concurrent expired-session exports shared one
refresh, invalid grant and changed subject returned 401, a revoked role
returned 403, and upstream 401 returned 502. CX ID browser smoke passed again.
The source overlay and test logs are in the recovery bundle under
`export-401-hotfix/`. All 29 GMR unit tests passed. Edge with the isolated
PostgreSQL and agent smoke test verified description and comparison downloads
as PDF, JSON, and Markdown. Built and checked
`cx58-v2-gmr:rc1-authfix-20261118` on Linux/amd64 and verified its saved Docker
archive. The hotfix stays local, with cloud verification pending.

### Photo description timeout checkpoint

`D:/Downloads/bug.txt` records an Ollama vision call for `qwen3-vl:8b`
exceeding 240 seconds on a report description. The agent converted its typed
deadline to a generic internal error. The agent SSE handler closes after its
terminal error, and GMR misclassified that ordinary EOF as an interrupted
stream. Updated the agent to preserve the vision error type and GMR to stop
after relaying the terminal error once. Kept the existing vision deadline.
The cloud Ollama `/api/ps` probe returned the vision model as loaded. The
reason the individual request exceeded the deadline is unverified.

The pinned Rig Ollama adapter sent `max_tokens` at the top level. The photo
description request now sends the intended 1,024-token cap in Ollama's
`options.num_predict`. A local request test exercises the serialized payload.
The actual cloud latency and output quality still require a photo report run.

The native GMR SSR build and all 30 library tests passed. A local HTTP smoke
test through authenticated GMR with a stub agent confirmed that a multiline
vision timeout appears once, abrupt EOF still reports an interruption, and
success still completes. Agent library tests reported 96 passed and 12
ignored S3 integration tests. The request regression test confirmed the cap
in the HTTP payload. The final native agent build passed. The next cloud
acceptance step is to rerun Show descriptions for the affected report and
record the vision duration and whether the response parses as complete JSON.

The operator asked to finish the Version 2 bug fixes and local verification
first. Build or publish Docker images only after a separate explicit command.

## 2026-11-18: Version 2 cloud release preparation

### Build inputs checkpoint

Reviewed the local Version 2 changes and the release order in the Stage 2
plan. The four deployment images are agent, admin backend, admin frontend,
and GMR. The agent applies the nullable CX ID migration on startup. Build the
four Linux/amd64 candidates from captured local sources under dedicated local
tags. Verify image contents and preserve the source manifests for traceability.
Cloud deployment and acceptance testing are planned for November 23–29.

Source snapshots and manifests are in `F:/Temp/cx58-v2-rc1-20261118/`. The
snapshots include the pending migration, SQLx metadata, PDF font and UI changes.
They contain the build inputs only, with local environment files excluded.
Next: run the four Docker builds and inspect the resulting images.

### Acceptance preparation checkpoint

Use the agent → admin backend/frontend → GMR order from the Stage 2 plan.
Before changing the stack, record its running image IDs and the database
backup reference. The agent health check and GMR `/api/ready` should pass on
the updated stack. Then exercise SSO by role, CX ID create/edit/clear on old
and new records, photo selection and streaming completion, PDF/JSON from the
same card, and EN/DE layouts on desktop and mobile. Validate outlet counts on
the deployment vision model using labeled real photos. Include an unfinished
outlet and a pair from the same viewpoint, and record counts and misses.

The local candidate build is running through `build-rc.ps1`. The separate
`verify-rc.ps1` checks platform, migration and prompt files, binaries and WASM
assets in disconnected containers, then writes the local image manifest.

### Agent image checkpoint

`cx58-v2-agent:rc1-20261118` built and loaded as Linux/amd64. The disconnected
container check passed for the server binary, the CX ID migration, and the
English and German outlet prompts. GMR is building next. Recorded local
baseline agent and GMR image IDs in `local-baseline-images.txt`. The cloud
stack's running digests still need to be captured at rollout.

### GMR image checkpoint

`cx58-v2-gmr:rc1-20261118` built and loaded as Linux/amd64. A disconnected
container verified the server, generated JavaScript/WASM, and export styles.
The admin backend is building next.

### Admin backend image checkpoint

`cx58-v2-admin-backend:rc1-20261118` built and loaded as Linux/amd64. The
disconnected container contained the executable. Docker build checks report
one pre-existing empty-continuation warning in the backend Dockerfile. The
normal backend build passed. The frontend image is building next.

### Verified release candidate checkpoint

All four Linux/amd64 images built and passed the disconnected container checks.
The admin frontend included the generated WASM and passed `nginx -t`. The
local image IDs and tags are recorded in
`F:/Temp/cx58-v2-rc1-20261118/image-manifest.json`, alongside the source
manifests, build logs, and verification script.

The images are stored locally under the `rc1-20261118` tags. Registry
publication, cloud rollout, and cloud acceptance testing are pending.

### Docker Hub publication checkpoint

Published these RC1 tags and verified their Linux/amd64 manifests in Docker Hub:

- `alsh/cx58:rc1-20261118` at `sha256:e621358b1c37aa08fa3f5b32764f45534a263c4ada0c12c6dafc4a4155ac321d`
- `alsh/cx58a:rc1-20261118` at `sha256:0b69d2a2ee6629bffa50658a6b6ff7534e77ac0864f00648cfb9fed0ed0160b4`

Admin images remain local. Cloud rollout and acceptance testing are pending.

### PostgreSQL CX ID migration checkpoint

Applied SQLx migration `20261117120000` to CX58 `ai_agent` on `db:5432` via
`gate`. The ten preceding migrations matched their source checksums. SQLx
recorded the new migration as successful with the expected checksum, and
`public.tree_nodes.cx_id` is nullable `text`. The pre-migration archive is
`F:/Temp/cx58-v2-db-20261118/ai_agent-before-cx-id.dump`. Verified that
`pg_restore` could read the full archive without connecting to a database.

## 2026-11-18: Version 2 fully installed wall outlets

### Requirements and design

Reviewed slide 6 of the Construct-X enhancement deck, including the finished
faceplate and exposed/loose installation examples. Count individual visible
wall receptacles only when the faceplate and socket are fully seated on the
wall. Open boxes, exposed mechanisms, loose assemblies, and uncertain fixtures
are excluded. The result describes visible installation only.

The vision response will classify candidates, and the agent will count the
confirmed ones. An unavailable assessment is distinct from a confirmed zero.
The comparison will use the counts from the two source photos in date order.
Advance both existing prompt cache versions and carry the structured counts
through cards and exports. Next: implement and test this contract, then check
available real-site image evidence locally.

### Implementation checkpoint

The count now flows from the vision parser through description and comparison
cards and Markdown, PDF, and versioned JSON. Comparisons report independent
source counts, including when views differ. Cache
prompt versions are now description 4 and comparison 3.

In the first local run with Qwen3-VL-4B, the three slide illustrations produced
one confirmed result for the finished outlet and zero for both unfinished
outlets. The real-site photos exposed format deviations and small missed
outlets. Tightened the candidate labels and scoped the generic attribute rule
to the other categories. Ambiguous or malformed candidates cannot increment
the count. Next: build, rerun the revised prompt on the labeled references,
and check UI and export propagation in Edge.

### Validation setup checkpoint

Reviewed six original indoor photos supplied by Britta at full resolution.
Recorded per-photo visible counts and reasons for exclusions in
`F:/Temp/cx58-outlets-20261118/manual-labels.csv`. The three slide images are
separate illustrations and are excluded from the real-site metrics. The
loopback inference script `vision.py` saves raw JSON, model tag, prompt and
input hashes, parsed counts and elapsed time. The separate
`run-outlet-smoke.ps1` uses deterministic card data for UI and export checks.

### Real-photo checkpoint at the existing 1200 px input size

Revised prompts produced usable candidate lists on all six real photos in
both languages. Against the local visual labels, exact confirmed counts were
5/6 in English and 4/6 in German. The three photos with zero confirmed outlets
had no positive counts in either language. Misses were on the wide views with
small outlets. The local sample is preliminary. Next: complete the 1800 px
comparison and finish the build/smoke checks.

### Image-size decision

The 1800 px run on the three positive photos improved some raw counts but
did not increase exact matches in either language and took longer. Kept the
existing 1200 px input path. Raw rows are in `vision-revised-1200.jsonl` and
`vision-revised-1800.jsonl`. Next: complete Edge and export propagation checks.

### Build checkpoint

The locked Windows agent build and the full GMR SSR/WASM build passed.
Next: run the isolated browser smoke and check generated PDF count text.

### Browser and PDF checkpoint

The first Edge smoke passed the description and comparison cards, structured
JSON, and Markdown propagation, including an explicit zero and an uncertainty
note. Text extraction showed that the PDF font lacked the comparison arrow.
Changed the PDF comparison to label each photo's count on a separate line.
Next: rebuild the agent and rerun the PDF verification.

### Unit test checkpoint

All seven targeted agent parser, cache, and comparison tests passed. All 29
GMR unit tests passed. The local 1200 px trials had median model times of 9.0 s
in English and 11.2 s in German across the six real-site samples. Accuracy
figures are based on one run per photo and language on the local 4B model.

### Render review checkpoint

The updated comparison PDF passed text verification for both photo labels and
counts. Visual review found a photo caption split from its image when the new
description count filled the first page. Increased the photo block's reserved
space so ordinary captions stay with their images. Next: rerun the smoke
and check the caption and JPEG together on the final description page.

### Verified local completion

Final agent build, Edge smoke, and PDF review passed. The description and
comparison cards carry the same confirmed counts and uncertainty as JSON,
Markdown, and PDF. The description PDF keeps its source photo caption and
JPEG on the same page. The isolated PostgreSQL cluster was stopped after the
smoke run. `verify-outlet-pdf.py` and the samples are in the validation folder.

Changes remain local. The benchmark used the installed Qwen3-VL-4B model and
six manually labeled real-site photos. Small or distant outlets were missed
in some views. Deployment-model accuracy and a broader same-view time series
remain unverified.

## 2026-11-18: Version 2 PDF and JSON export

### Requirements and design

Reviewed slides 4–5 of `T:/Construct-X/Scope - Enhancement- 2026-08-06.pptx`.
The export belongs to each description or comparison card and includes the exact
request, result, relevant photos and capture times, CX IDs, and tree path. PDF
adds a branded header, footer, and page numbers. Keep the legacy Markdown action.

Use one agent-side `cx-report-export/v1` model for both outputs. The card supplies
its original request and result, and the agent resolves the report IDs against
the current user's access and loads the object path, CX IDs, and photos from the
database and configured storage. JSON embeds the same bounded JPEGs as PDF with
stable report IDs, so both files work offline. Use a fixed A4 layout until a
project-specific template is provided. Next: implement the agent model and data
assembly, then connect the authenticated GMR route and card controls.

### Implementation checkpoint

Implemented the agent export route, authorized report lineage lookup, bounded
image reads, JSON serialization and localized A4 renderer. The renderer embeds
OFL-licensed Noto Sans. The GMR proxy signs requests using the email from the
operator session. Description and comparison cards preserve their originating
prompt and now offer PDF, JSON and Markdown, with download progress and errors.
Next: finish agent and GMR builds, then run the browser-to-agent export smoke
test and inspect generated PDF pages for both card types.

### Build and smoke-test checkpoint

The initial agent and GMR SSR/WASM builds passed. Added the draft 2020-12 JSON
Schema under `cx58-agent/docs`, fixed SSE decoding to preserve UTF-8 sequences
and CRLF separators split across chunks, and added EXIF orientation handling
for legacy photos. The isolated smoke harness is
`F:/Temp/cx58-report-export-20261118/export_smoke.py`. It uses the CX ID test
cluster on loopback port 55483, a local S3 stub, and deterministic chat events.
Next: rebuild the final sources, run the smoke harness, and verify both PDFs.

### End-to-end verification checkpoint

Agent and GMR SSR/WASM builds passed. All 29 GMR unit tests passed. The
loopback Edge smoke passed PDF, JSON, and Markdown downloads on both cards
after two distinct prompts. The actual export handler ran against the isolated
PostgreSQL cluster and S3 stub. Tests covered session-derived identity, guest
and unauthorized report rejection, cross-object comparison rejection, and a
missing photo. Both downloaded JSON files passed schema validation. PDF text
and photo bytes matched the JSON data. Poppler renders showed no clipping.
The description with extra result fields used an avoidable second page. Next:
fit photos into available space where readable, then rerun export and PDF QA.

### Verified local completion

The final renderer fits the description and its photo on one A4 page when
space permits. The long comparison fixture renders on four pages. Reran the
browser smoke and visually reviewed all final pages, including the German
description. The PDF text checker matched the request, result, tree path and
CX IDs, checked page numbers, and verified that embedded photo bytes exactly
match the JSON payloads. The earlier GMR CX ID smoke passed again as well.

Reproduction scripts and rendered samples are under
`F:/Temp/cx58-report-export-20261118/`. Run `run-smoke.ps1` for the loopback
export tests and `verify_pdf.py` with the bundled Python runtime for text and
JPEG parity checks. The isolated PostgreSQL cluster was stopped after testing.
The card supplies the request and result. Report metadata is resolved at the
time of each download. JSON preserves the original Unicode. PDF uses Noto Sans
and prints readable code-point escapes for glyphs the font does not cover.

The implementation remains local. The agent applies the CX ID migration on
startup. Use the updated agent with the updated GMR for rollout. Live SSO, S3,
and deployment were outside this local verification.

## 2026-11-18: CX58 Version 2 CX ID checkpoint

The CX ID migration and editor changes survived the blackout in the `gmr`,
`cx58-agent`, and `cx58-admin` worktrees. The agent migration adds nullable
`tree_nodes.cx_id` text. Objects and individual photo reports have independent
values. The admin and agent flows were verified with an isolated PostgreSQL 18
instance before interruption.

After restart, the GMR proxy and Edge smoke test passed for navigation from the
home page, Unicode updates, and clearing both kinds of IDs. Direct loads of
`/?panel=objects` and `/reports` reproduced a hydration panic. The captured HTML
shows the client-only tree loader interrupting the streamed sidebar markup.
`SideBody` now mounts both tree panels after hydration. A full `cargo leptos
build` and the Edge smoke test passed again after the recovery, with both
direct routes required to pass. The agent smoke test also passed again on the
isolated PostgreSQL 18 cluster, including access checks and Unicode round trips.
`cargo test --locked --features ssr --lib` passed all 28 GMR unit tests.
The smoke script is at
`F:/Temp/cx58-cxid-22a480e01bc84bfe91ea16c2d1c5f410/gmr_proxy_smoke.py`.
It takes the absolute GMR executable path and the `target/site` path, and starts
its own loopback GMR instance and OIDC/agent stub.
The sibling `gmr_cx_id_smoke.py` takes the absolute agent executable path. It
expects the isolated `data` cluster in that directory on `127.0.0.1:55483`. Start
that cluster with `pg_ctl -o "-p 55483 -h 127.0.0.1"` and stop it after the run.

The CX ID changes are local. For rollout, apply agent migration
`20261117120000_add_cx_id_to_tree_nodes.sql` with the agent update before enabling
the updated GMR or admin UI. PDF/JSON exports and the power-outlet focus are
pending Version 2 work.

During this Version 2 work, record the current changes, last completed check,
and next action in this worklog after each implementation or verification block
and before a long-running build or smoke test.

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
