use leptos::prelude::{ElementChild, GlobalAttributes, OnAttribute};
use leptos::prelude::ClassAttribute;
use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::show_description::download_text_file;

/// Client-side structure matching server's DescriptionData
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparisonData {
    pub object_name: String,
    pub prev_date: String,
    pub next_date: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doors: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radiators: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openings: Option<String>,
}

impl ComparisonData {
    /// Renders the comparison data as a Markdown string
    fn to_markdown(&self) -> String {
        let mut md = format!(
            "# {}\n\n**Changes from** {} **to** {}\n\n{}\n",
            self.object_name, self.prev_date, self.next_date, self.description
        );
        if let Some(v) = &self.windows {
            md.push_str(&format!("\n**Windows:** {}\n", v));
        }
        if let Some(v) = &self.doors {
            md.push_str(&format!("\n**Doors:** {}\n", v));
        }
        if let Some(v) = &self.radiators {
            md.push_str(&format!("\n**Radiators:** {}\n", v));
        }
        if let Some(v) = &self.openings {
            md.push_str(&format!("\n**Openings:** {}\n", v));
        }
        md
    }

    /// Builds the filename from the header text (spaces replaced with underscores)
    fn filename(&self) -> String {
        let object_name = self.object_name.replace(" - ", "_");
        let header = format!(
            "Changes_{}_from_{}_to_{}",
            self.object_name, self.prev_date, self.next_date
        );
        let sanitized = header.replace(' ', "_");
        format!("{}.md", sanitized)
    }
}

/// Compact version without optional sections
#[component]
fn ComparisonDetailItem(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="detail-item">
            <strong>{label}</strong>
            {value}
        </div>
    }
}

/// Triggers a browser download of `content` as a text file with the given `filename`
fn download_text_file_(filename: &str, content: &str) {
    use wasm_bindgen::JsCast;
    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    // Build a Blob containing the markdown text
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&wasm_bindgen::JsValue::from_str(content));
    let mut blob_options = web_sys::BlobPropertyBag::new();
    blob_options.type_("text/markdown");
    let blob =
        web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &blob_options)
            .expect("failed to create blob");

    // Create a temporary object URL and click a hidden <a> element to trigger download
    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .expect("failed to create object URL");

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

#[component]
pub fn ComparisonRenderer(data: ComparisonData) -> impl IntoView {
    // Clone what we need for the download closure
    let markdown = data.to_markdown();
    let filename = data.filename();

    let on_download = move |_| {
        download_text_file(&filename, &markdown);
    };

    view! {
        <div class="description-compact border-left-cx58">
            <div class="compact-header">
                <span>
                    <i class="fas fa-building right5"></i>
                    <strong>{data.object_name}</strong>
                </span>
                <span class="compact-date">
                    <span class="right5">"Changes from "</span>
                    <i class="fas fa-image right5"></i>{data.prev_date}
                    <span class="left5 right5">" to "</span>
                    <i class="fas fa-image right5"></i>{data.next_date}
                </span>
            </div>
            <p class="compact-description">{data.description}</p>

            <div class="compact-details">
                {data.windows.map(|v| view! { <ComparisonDetailItem label="Windows: " value=v /> })}
                {data.doors.map(|v| view! { <ComparisonDetailItem label="Doors: " value=v /> })}
                {data.radiators.map(|v| view! { <ComparisonDetailItem label="Radiators: " value=v /> })}
                {data.openings.map(|v| view! { <ComparisonDetailItem label="Openings: " value=v /> })}
            </div>

            // Download button anchored to the bottom-right of the card
            <div class="compact-download-row">
                <button
                    class="compact-download-btn"
                    title="Download as Markdown"
                    on:click=on_download
                >
                    <i class="fas fa-arrow-down"></i>
                </button>
            </div>
        </div>
    }
}
