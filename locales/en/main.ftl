# locales/en/main.ftl

logo = CX-5.8
title = CX-5.8 AI agent
welcome-cx58 = Welcome  to CX-5.8!
you-are-unauthenticated = You are unauthenticated!
welcome-authenticated = Welcome! You are authenticated!
log-in = Log In
p404 = 404 - Page Not Found
page-not-found = The requested page was not found.
this-is-the-public = This is the public
home-page = home page.
no-access = Unfortunately, you do not have access to any objects.
no-access-message = Please contact the administrator to gain access!
ask-me-anything = Ask me anything...
new-chat = New chat
faq = FAQ
objects = Objects
home = Home
play = Under construction
profile = Profile
users = Users
start = Start
stop = Stop
models = Models
models-vision = Vision
models-text = Text
models-chat = Chat
models-link-title = Use vision model where possible
models-save = Save
models-saving = Saving...
models-saved = Saved
models-loading = Loading models...
models-refresh = Refresh models

# FAQ
q-1 = Show my objects
q-2 = Show last reports
q-3 = Show all reports
q-4 = Show descriptions
q-5 = Show last description
q-6 = Show prev description
q-7 = Show changes

# Under construction
select-a-language = Select a language:
language-selected-is = The selected language is { $lang }.

# --- Chat UI (bottom footer) ---
chat-footer = CX-5.8 is AI and can make mistakes.

# --- Client-side error messages ---
# Used in: handle_stream, process_stream (chat.rs)
# Must be captured via move_tr!().get() BEFORE async boundary, passed as ChatI18n
chat-error-connection = Connection error: { $error }
chat-error-request-failed = Request failed: { $error }
chat-error-invalid-reader = Invalid stream reader
chat-error-read-error = Read error: { $error }
chat-error-no-chunk-value = No value in stream chunk
chat-error-server = { $error }

# --- Stop reason labels ---
# Mapped from SSE on_stop event data field:
#   "by_user"         → chat-stop-by-user
#   "timeout"         → chat-stop-timeout
#   "max_tokens"      → chat-stop-max-tokens
#   "transport_error" → chat-stop-transport-error
chat-stop-by-user = Stopped by user
chat-stop-timeout = Response timed out
chat-stop-max-tokens = Token limit reached
chat-stop-transport-error = Connection lost
chat-stop-stream-ended = The agent response ended before completion. Please try again.

# --- Server SSE error payloads (if server migrates to key-based errors) ---
# Used when server sends event: error / data: llm-error or transport-error
chat-llm-error = LLM error [{ $status }]: { $detail }
chat-transport-error = Transport error: { $error }
# --- Carousel component (show_carusel.rs) ---
carousel-reports-label = reports
carousel-no-reports = No reports available
carousel-image-fallback = Image { $index }
carousel-no-data = No data available

# --- Comparison component (show_comparison) ---
comparison-changes-from = Changes from
comparison-changes-to = to

# --- Description component (show_description.rs) ---
description-report-label = Report

# --- Shared detail labels ---
# Used in: show_comparison (ComparisonDetailItem) and show_description.rs (DetailItem)
# Both files render Windows / Doors / Radiators / Openings — same domain, shared keys
detail-label-windows = Windows:
detail-label-doors = Doors:
detail-label-radiators = Radiators:
detail-label-openings = Openings:
detail-label-power-outlets = Fully installed wall outlets:
detail-label-power-outlets-comparison = Fully installed wall outlets (earlier → later):
detail-outlet-uncertain = unclear candidates excluded
detail-outlet-unavailable = not assessed

# --- Shared download button tooltip ---
# Used in: show_comparison and show_description.rs
download-as-markdown = Download as Markdown
download-as-pdf = Download as PDF
download-as-json = Download as JSON
report-export-working = Preparing report…
report-export-session-error = Your session has expired or is unavailable. Sign in again to download the report.
report-export-access-error = This report is unavailable. Check your report access.
report-export-size-error = The report is too large to export.
report-export-busy = Exports are busy. Try again in a moment.
report-export-photo-error = The report or its photos are temporarily unavailable. Try again.
report-export-error = The download failed. Try again.

# --- Tree component (show_tree.rs) ---
# Fallback label for nodes where name is None
tree-node-unnamed = (unnamed)

reports = Reports
reports-select-object = Select an object to show reports
reports-loading = Loading reports...
reports-empty = No reports yet
reports-upload = Upload report
reports-delete = Delete report
reports-image = Report
reports-use-context = Use this report in chat context
reports-datetime = Report date and time
reports-file = Report image
reports-save-date = Save report date
reports-prev-page = Previous reports page
reports-next-page = Next reports page
selected-context-unnamed = Unnamed
selected-context-remove = Remove selected object
selected-report-open = Open selected report image
selected-report-remove = Remove selected report
selected-report-close = Close report image

cx-id-edit = Edit CX ID
cx-id-object = Object CX ID
cx-id-session-error = Your session has expired. Sign in again to edit the CX ID.
cx-id-optional = CX ID (optional)
cx-id-save = Save
cx-id-clear = Clear
cx-id-close = Close
cx-id-saved = CX ID saved.
cx-id-working = Please wait…
cx-id-access-error = You do not have access to the CX ID for this item.
cx-id-invalid = The CX ID contains an unsupported character.
cx-id-request-error = Could not load or save the CX ID. Try again.
