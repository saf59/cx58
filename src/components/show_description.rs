#![cfg(not(feature = "ssr"))]
use leptos::prelude::*;
use leptos::*;
use leptos_fluent::{move_tr, I18n};
use crate::components::chat_data::{extract_name_pair, DescriptionData};

/// Triggers a browser download of `content` as a text file with the given `filename`
#[cfg(not(feature = "ssr"))]
pub fn download_text_file(filename: &str, content: &str) {
    use wasm_bindgen::JsCast;
    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    // Build a Blob containing the markdown text
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&wasm_bindgen::JsValue::from_str(content));
    let blob_options = web_sys::BlobPropertyBag::new();
    blob_options.set_type("text/markdown");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &blob_options)
        .expect("failed to create blob");

    // Create a temporary object URL and click a hidden <a> element to trigger download
    let url =
        web_sys::Url::create_object_url_with_blob(&blob).expect("failed to create object URL");

    let a: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .expect("failed to create <a>")
        .dyn_into()
        .expect("not an anchor");

    a.set_href(&url);
    a.set_download(filename);
    a.set_attribute("style", "display:none").ok();
    document.body().expect("no body").append_child(&a).ok();
    a.click();
    document.body().expect("no body").remove_child(&a).ok();
    web_sys::Url::revoke_object_url(&url).ok();
}

/// Component to render multiple descriptions
#[component]
pub fn DescriptionListRenderer(
    /// List of description data to display
    data: Vec<DescriptionData>,
) -> impl IntoView {
    view! {
        <div class="description-list">
            <For
                each=move || data.clone()
                key=|item| format!("{}-{}", item.object, item.date_id)
                children=move |item| {
                    view! { <DescriptionRendererCompact data=item /> }
                }
            />
        </div>
    }
}

/// Compact version without optional sections
#[component]
fn DetailItem(label: String, value: String) -> impl IntoView {
    view! {
        <div class="detail-item">
            <strong>{label}</strong>
            {value}
        </div>
    }
}

#[component]
pub fn DescriptionRendererCompact(data: DescriptionData) -> impl IntoView {
    let i18n = expect_context::<I18n>();
    let (object_name, report_name) = extract_name_pair(data.object.as_str());

    // Prepare download payload before the view consumes `data`
    let markdown = data.to_markdown();
    let filename = data.filename();

    let on_download = move |_| {
        #[cfg(not(feature = "ssr"))]
        download_text_file(&filename, &markdown);
    };

    view! {
        <div class="description-compact border-left-green">
            <div class="compact-header">
                <span>
                    <i class="fas fa-building right5"></i>
                    <strong>{object_name}</strong>
                </span>
                <span class="compact-date">
                    <span class="right5">{move_tr!("description-report-label")}" "</span>
                    <i class="fas fa-image right5"></i>
                    {report_name}
                </span>
            </div>
            <p class="compact-description">{data.description}</p>

            <div class="compact-details">
                {data.windows.map(|v| view! { <DetailItem label=i18n.tr("detail-label-windows") value=v /> })}
                {data.doors.map(|v| view! { <DetailItem label=i18n.tr("detail-label-doors") value=v /> })}
                {data.radiators.map(|v| view! { <DetailItem label=i18n.tr("detail-label-radiators") value=v /> })}
                {data.openings.map(|v| view! { <DetailItem label=i18n.tr("detail-label-openings") value=v /> })}
            </div>

            // Download button anchored to the bottom-right of the card
            <div class="compact-download-row">
                <button
                    class="compact-download-btn"
                    title=move_tr!("download-as-markdown")
                    on:click=on_download
                >
                    <i class="fas fa-arrow-down"></i>
                </button>
            </div>
        </div>
    }
}

