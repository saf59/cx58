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

# --- Shared download button tooltip ---
# Used in: show_comparison and show_description.rs
download-as-markdown = Download as Markdown

# --- Tree component (show_tree.rs) ---
# Fallback label for nodes where name is None
tree-node-unnamed = (unnamed)
