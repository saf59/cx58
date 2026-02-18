use leptos::prelude::ElementChild;
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

/// Component to render a single DescriptionData item
#[component]
pub fn DescriptionRenderer(data: DescriptionData) -> impl IntoView {
    let formatted_date = data.created_at; //.format("%Y-%m-%d %H:%M:%S UTC").to_string();

    // Format confidence as percentage if present
    let confidence_display = data.confidence.map(|c| format!("{:.1}%", c * 100.0));

    view! {
        <div class="description-container">
            // Header section
            <div class="description-header">
                //<h3 class="description-title">{data.object.clone()}</h3>
                <div class="description-meta">
                    <i class="fas fa-building"></i>
                    <span class="description-date">{data.object.clone()}</span>
                    <i class="fas fa-image"></i>
                    <span class="description-model">"Model: " {data.date.clone()}</span>
                    {confidence_display
                        .map(|conf| {
                            view! {
                                <span class="description-confidence">"Confidence: " {conf}</span>
                            }
                        })}
                </div>
            </div>

            // Main description
            <div class="description-section">
                <h4 class="section-title">"Description"</h4>
                <p class="section-content">{data.description.clone()}</p>
            </div>

            // Optional sections
            {optional_section("Windows", data.windows.as_ref())}
            {optional_section("Doors", data.doors.as_ref())}
            {optional_section("Radiators", data.radiators.as_ref())}
            {optional_section("Openings", data.openings.as_ref())}
        </div>
    }
}

fn optional_section(title: &'static str, content: Option<&String>) -> impl IntoView {
    content.map(|text| {
        view! {
            <div class="description-section">
                <h4 class="section-title">{title}</h4>
                <p class="section-content">{text.clone()}</p>
            </div>
        }
    })
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
                    view! { <DescriptionRenderer data=item /> }
                }
            />
        </div>
    }
}

/// Compact version without optional sections
#[component]
pub fn DescriptionRendererCompact(data: DescriptionData) -> impl IntoView {
    view! {
        <div class="description-compact">
            <div class="compact-header">
                <strong>{data.object.clone()}</strong>
                <span class="compact-date">{data.date.clone()}</span>
            </div>
            <p class="compact-description">{data.description.clone()}</p>

            <div class="compact-details">
                {data
                    .windows
                    .as_ref()
                    .map(|w| {
                        view! {
                            <div class="detail-item">
                                <strong>"Windows: "</strong>
                                {w.clone()}
                            </div>
                        }
                    })}
                {data
                    .doors
                    .as_ref()
                    .map(|d| {
                        view! {
                            <div class="detail-item">
                                <strong>"Doors: "</strong>
                                {d.clone()}
                            </div>
                        }
                    })}
                {data
                    .radiators
                    .as_ref()
                    .map(|r| {
                        view! {
                            <div class="detail-item">
                                <strong>"Radiators: "</strong>
                                {r.clone()}
                            </div>
                        }
                    })}
                {data
                    .openings
                    .as_ref()
                    .map(|o| {
                        view! {
                            <div class="detail-item">
                                <strong>"Openings: "</strong>
                                {o.clone()}
                            </div>
                        }
                    })}
            </div>
        </div>
    }
}
