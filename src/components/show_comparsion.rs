use leptos::prelude::ElementChild;
use leptos::prelude::ClassAttribute;
use leptos::*;
use serde::{Deserialize, Serialize};

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

#[component]
pub fn ComparisonRenderer(data: ComparisonData) -> impl IntoView {
    view! {
        <div class="description-compact">
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
        </div>
    }
}
