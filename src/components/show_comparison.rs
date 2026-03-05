use crate::components::chat_data::ComparisonData;
use crate::components::show_description::download_text_file;
use leptos::prelude::{expect_context, ClassAttribute};
use leptos::prelude::{ElementChild, GlobalAttributes, OnAttribute};
use leptos::*;
use leptos_fluent::{move_tr, I18n};

/// Compact version without optional sections
#[component]
fn ComparisonDetailItem(label: String, value: String) -> impl IntoView {
    view! {
        <div class="detail-item">
            <strong>{label}" "</strong>
            {value}
        </div>
    }
}

#[component]
pub fn ComparisonRenderer(data: ComparisonData) -> impl IntoView {
    let i18n = expect_context::<I18n>();
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
                    <span class="right5">{move_tr!("comparison-changes-from")}" "</span>
                    <i class="fas fa-image right5"></i>
                    {data.prev_date}
                    <span class="left5 right5">" "{move_tr!("comparison-changes-to")}" "</span>
                    <i class="fas fa-image right5"></i>
                    {data.next_date}
                </span>
            </div>
            <p class="compact-description">{data.description}</p>

            <div class="compact-details">
                {data.windows.map(|v| view! {
                    <ComparisonDetailItem label=i18n.tr("detail-label-windows") value=v />})}
                {data.doors.map(|v| view! {
                    <ComparisonDetailItem label=i18n.tr("detail-label-doors") value=v />})}
                {data.radiators.map(|v| view! {
                    <ComparisonDetailItem label=i18n.tr("detail-label-radiators") value=v />})}
                {data.openings.map(|v| view! {
                    <ComparisonDetailItem label=i18n.tr("detail-label-openings") value=v />})}
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
