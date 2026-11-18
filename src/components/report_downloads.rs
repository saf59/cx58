use crate::report_export::{ExportDownload, ExportFormat, ReportExport};
use leptos::prelude::*;
use leptos_fluent::move_tr;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, RequestInit, Response, Url, window};

#[component]
pub fn ReportDownloads(
    export: Option<ReportExport>,
    filename: String,
    markdown: String,
) -> impl IntoView {
    let has_export = export.is_some();
    let export = StoredValue::new(export);
    let pending = RwSignal::new(false);
    let error = RwSignal::new(None::<u16>);
    let download = move |format: ExportFormat| {
        if pending.get_untracked() {
            return;
        }
        let Some(report) = export.get_value() else {
            return;
        };
        pending.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            if let Err(status) = fetch_export(ExportDownload { report, format }).await {
                error.try_set(Some(status));
            }
            pending.try_set(false);
        });
    };
    let download_markdown = move |_| {
        error.set(None);
        if save_markdown(&filename, &markdown).is_err() {
            error.set(Some(0));
        }
    };

    view! {
        <div class="compact-download-row">
            {has_export.then(|| view! {
                <button type="button" class="compact-download-btn"
                    disabled=move || pending.get()
                    on:click=move |_| download(ExportFormat::Pdf)
                    aria-label=move || move_tr!("download-as-pdf").get()>"PDF"</button>
                <button type="button" class="compact-download-btn"
                    disabled=move || pending.get()
                    on:click=move |_| download(ExportFormat::Json)
                    aria-label=move || move_tr!("download-as-json").get()>"JSON"</button>
            })}
            <button type="button" class="compact-download-btn"
                disabled=move || pending.get()
                on:click=download_markdown
                aria-label=move || move_tr!("download-as-markdown").get()>"Markdown"</button>
        </div>
        <Show when=move || pending.get()>
            <div class="compact-download-status" role="status">{move_tr!("report-export-working")}</div>
        </Show>
        {move || error.get().map(|status| view! {
            <div class="compact-download-status" role="alert">
                {match status {
                    401 => move_tr!("report-export-session-error").get(),
                    403 | 404 => move_tr!("report-export-access-error").get(),
                    413 => move_tr!("report-export-size-error").get(),
                    429 => move_tr!("report-export-busy").get(),
                    503 => move_tr!("report-export-photo-error").get(),
                    _ => move_tr!("report-export-error").get(),
                }}
            </div>
        })}
    }
}

async fn fetch_export(download: ExportDownload) -> Result<(), u16> {
    const NETWORK_ERROR: u16 = 0;
    let window = window().ok_or(NETWORK_ERROR)?;
    let body = serde_json::to_string(&download).map_err(|_| NETWORK_ERROR)?;
    let headers = web_sys::Headers::new().map_err(|_| NETWORK_ERROR)?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|_| NETWORK_ERROR)?;
    let options = RequestInit::new();
    options.set_method("POST");
    options.set_headers(&headers);
    options.set_body(&JsValue::from_str(&body));
    let response: Response =
        JsFuture::from(window.fetch_with_str_and_init("/api/report-export", &options))
            .await
            .map_err(|_| NETWORK_ERROR)?
            .dyn_into()
            .map_err(|_| NETWORK_ERROR)?;
    if !response.ok() {
        return Err(response.status());
    }
    let blob: Blob = JsFuture::from(response.blob().map_err(|_| NETWORK_ERROR)?)
        .await
        .map_err(|_| NETWORK_ERROR)?
        .dyn_into()
        .map_err(|_| NETWORK_ERROR)?;
    save_blob(download.format.filename(), &blob).map_err(|_| NETWORK_ERROR)
}

fn save_markdown(filename: &str, markdown: &str) -> Result<(), JsValue> {
    let parts = js_sys::Array::new();
    parts.push(&JsValue::from_str(markdown));
    let options = BlobPropertyBag::new();
    options.set_type("text/markdown");
    let blob = Blob::new_with_str_sequence_and_options(&parts, &options)?;
    save_blob(filename, &blob)
}

fn save_blob(filename: &str, blob: &Blob) -> Result<(), JsValue> {
    let document = window()
        .and_then(|window| window.document())
        .ok_or(JsValue::NULL)?;
    let body = document.body().ok_or(JsValue::NULL)?;
    let link: HtmlAnchorElement = document.create_element("a")?.dyn_into()?;
    let url = Url::create_object_url_with_blob(blob)?;
    link.set_href(&url);
    link.set_download(filename);
    let result = body.append_child(&link).and_then(|_| {
        link.click();
        body.remove_child(&link)
    });
    Url::revoke_object_url(&url)?;
    result.map(|_| ())
}
