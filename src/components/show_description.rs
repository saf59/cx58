use leptos::*;
use leptos::prelude::ClassAttribute;
use leptos::prelude::ElementChild;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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
    pub created_at: DateTime<Utc>,
}

/// Component to render a single DescriptionData item
#[component]
pub fn DescriptionRenderer(
    /// The description data to display
    data: DescriptionData,
) -> impl IntoView {
    // Format the timestamp
    let formatted_date = data.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string();

    // Format confidence as percentage if present
    let confidence_display = data.confidence.map(|c| format!("{:.1}%", c * 100.0));

    view! {
        <div class="description-container">
            // Header section
            <div class="description-header">
                <h3 class="description-title">{data.object.clone()}</h3>
                <div class="description-meta">
                    <span class="description-date">{data.date.clone()}</span>
                    <span class="description-model">"Model: " {data.model_name.clone()}</span>
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
            {data
                .windows
                .as_ref()
                .map(|windows| {
                    view! {
                        <div class="description-section">
                            <h4 class="section-title">"Windows"</h4>
                            <p class="section-content">{windows.clone()}</p>
                        </div>
                    }
                })}

            {data
                .doors
                .as_ref()
                .map(|doors| {
                    view! {
                        <div class="description-section">
                            <h4 class="section-title">"Doors"</h4>
                            <p class="section-content">{doors.clone()}</p>
                        </div>
                    }
                })}

            {data
                .radiators
                .as_ref()
                .map(|radiators| {
                    view! {
                        <div class="description-section">
                            <h4 class="section-title">"Radiators"</h4>
                            <p class="section-content">{radiators.clone()}</p>
                        </div>
                    }
                })}

            {data
                .openings
                .as_ref()
                .map(|openings| {
                    view! {
                        <div class="description-section">
                            <h4 class="section-title">"Openings"</h4>
                            <p class="section-content">{openings.clone()}</p>
                        </div>
                    }
                })}

            // Footer with timestamp
            <div class="description-footer">
                <span class="description-timestamp">"Generated: " {formatted_date}</span>
            </div>
        </div>
    }
}

/// Component to render multiple descriptions
/*#[component]
pub fn DescriptionListRenderer(
    /// List of description data to display
    data: Vec<DescriptionData>,
) -> impl IntoView {
    view! {
        <div class="description-list">
            <For
                each=move || data.clone()
                key=|item| format!("{}-{}", item.object, item.created_at.timestamp())
                children=move |item| {
                    view! {
                        <DescriptionRenderer data=item />
                    }
                }
            />
        </div>
    }
}
*/
/// Compact version without optional sections
#[component]
pub fn DescriptionRendererCompact(
    data: DescriptionData,
) -> impl IntoView {
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
