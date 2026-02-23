use leptos::prelude::{ElementChild, GlobalAttributes, OnAttribute};
use leptos::prelude::{ClassAttribute, For};
use leptos::*;
use serde::{Deserialize, Serialize};

/// Client-side structure matching server's DescriptionData
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DescriptionData {
    pub object: String,
    #[serde(skip)]
    pub object_id: String, // We skip serialization but keep for internal use
    pub date: String,
    #[serde(skip)]
    pub date_id: String, // We skip serialization but keep for internal use
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doors: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radiators: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openings: Option<String>,
    pub model_name: String,
    pub confidence: Option<f32>,
    pub created_at: String,
}

impl DescriptionData {
    /// Renders the description data as a Markdown string
    fn to_markdown(&self) -> String {
        let (object_name, report_name) = extract_name_pair(&self.object);
        let mut md = format!(
            "# {}\n\n**Report:** {}\n\n{}\n",
            object_name, report_name, self.description
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
        let (object_name, report_name) = extract_name_pair(&self.object);
        let object_name = object_name.replace(" - ", "_");
        let header = format!("Report_{}_{}", object_name, report_name);
        let sanitized = header.replace(' ', "_");
        format!("{}.md", sanitized)
    }
}

/// Triggers a browser download of `content` as a text file with the given `filename`
pub fn download_text_file(filename: &str, content: &str) {
    use wasm_bindgen::JsCast;
    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    // Build a Blob containing the markdown text
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&wasm_bindgen::JsValue::from_str(content));
    let blob_options = web_sys::BlobPropertyBag::new();
    blob_options.set_type("text/markdown");
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
fn DetailItem(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="detail-item">
            <strong>{label}</strong>
            {value}
        </div>
    }
}

#[component]
pub fn DescriptionRendererCompact(data: DescriptionData) -> impl IntoView {
    let (object_name, report_name) = extract_name_pair(data.object.as_str());

    // Prepare download payload before the view consumes `data`
    let markdown = data.to_markdown();
    let filename = data.filename();

    let on_download = move |_| {
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
                    <span class="right5">"Report "</span>
                    <i class="fas fa-image right5"></i>
                    {report_name}
                </span>
            </div>
            <p class="compact-description">{data.description}</p>

            <div class="compact-details">
                {data.windows.map(|v| view! { <DetailItem label="Windows: " value=v /> })}
                {data.doors.map(|v| view! { <DetailItem label="Doors: " value=v /> })}
                {data.radiators.map(|v| view! { <DetailItem label="Radiators: " value=v /> })}
                {data.openings.map(|v| view! { <DetailItem label="Openings: " value=v /> })}
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

fn extract_name_pair(full_name: &str) -> (String, String) {
    let full_name = full_name.replace("Root/", "");
    let parts: Vec<&str> = full_name.split('/').collect();

    let report_name = parts.last().unwrap_or(&"").to_string();
    let object_name = parts[..parts.len().saturating_sub(1)].join(" - ");

    (object_name, report_name)
}
