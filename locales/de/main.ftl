# locales/de/main.ftl

logo = CX-5.8
welcome-cx58 = Willkommen bei CX-5.8!
you-are-unauthenticated = Sie sind nicht authentifiziert!
welcome-authenticated = Willkommen! Sie sind authentifiziert!
log-in = Anmelden
p404 = 404 - Seite nicht gefunden
page-not-found = Die angeforderte Seite wurde nicht gefunden.
this-is-the-public = Dies ist die öffentliche
home-page = Startseite.
no-access = Leider haben Sie keinen Zugriff auf Objekte.
no-access-message = Bitte kontaktieren Sie den Administrator, um Zugriff zu erhalten!
ask-me-anything = Frag mich alles...
new-chat = Neuer Chat
faq = FAQ
objects = Objekte
home = Startseite
play = In Arbeit
profile = Profil
users = Benutzer
start = Start
stop = Stopp

# FAQ
q-1 = Meine Objekte anzeigen
q-2 = Letzte Berichte anzeigen
q-3 = Alle Berichte anzeigen
q-4 = Beschreibungen anzeigen
q-5 = Letzte Beschreibung anzeigen
q-6 = Vorherige Beschreibung anzeigen
q-7 = Änderungen anzeigen

# Under construction
select-a-language = Wählen Sie eine Sprache:
language-selected-is = Die ausgewählte Sprache ist { $lang }.

# --- Chat UI (bottom footer) ---
chat-footer = CX-58 ist eine KI und kann Fehler machen.

# --- Client-side error messages ---
# Used in: handle_stream, process_stream (chat.rs)
# Must be captured via move_tr!().get() BEFORE async boundary, passed as ChatI18n
chat-error-connection = ⚠ Verbindungsfehler: { $error }
chat-error-request-failed = Anfrage fehlgeschlagen: { $error }
chat-error-invalid-reader = Ungültiger Stream-Reader
chat-error-read-error = Lesefehler: { $error }
chat-error-no-chunk-value = Kein Wert im Stream-Chunk
chat-error-server = ⚠ { $error }

# --- Stop reason labels ---
# Mapped from SSE on_stop event data field:
#   "by_user"         → chat-stop-by-user
#   "timeout"         → chat-stop-timeout
#   "max_tokens"      → chat-stop-max-tokens
#   "transport_error" → chat-stop-transport-error
chat-stop-by-user = Vom Benutzer gestoppt
chat-stop-timeout = Zeitüberschreitung der Antwort
chat-stop-max-tokens = Token-Limit erreicht
chat-stop-transport-error = Verbindung unterbrochen

# --- Server SSE error payloads (if server migrates to key-based errors) ---
# Used when server sends event: error / data: llm-error or transport-error
chat-llm-error = LLM-Fehler [{ $status }]: { $detail }
chat-transport-error = Übertragungsfehler: { $error }

# --- Carousel component (show_carusel.rs) ---
carousel-reports-label = Berichte
carousel-no-reports = Keine Berichte verfügbar
carousel-image-fallback = Bild { $index }
carousel-no-data = Keine Daten verfügbar

# --- Comparison component (show_comparsion.rs) ---
comparison-changes-from = Änderungen von
comparison-changes-to = bis

# --- Description component (show_description.rs) ---
description-report-label = Bericht

# --- Shared detail labels ---
# Used in: show_comparsion.rs (ComparisonDetailItem) and show_description.rs (DetailItem)
detail-label-windows = Fenster:
detail-label-doors = Türen:
detail-label-radiators = Heizkörper:
detail-label-openings = Öffnungen:

# --- Shared download button tooltip ---
# Used in: show_comparsion.rs and show_description.rs
download-as-markdown = Als Markdown herunterladen

# --- Tree component (show_tree.rs) ---
# Fallback label for nodes where name is None
tree-node-unnamed = (unbenannt)
