use crate::components::chat_context::ChatContext;
use crate::components::chat_data::ComparisonData;
use crate::components::report_downloads::ReportDownloads;
use crate::components::report_preview::ReportPreview;
use crate::report_export::{
    ExportResult, ExportSources, PowerOutletResult, ReportExport, ReportRequest,
};
use leptos::prelude::ElementChild;
use leptos::prelude::{ClassAttribute, expect_context};
use leptos::*;
use leptos_fluent::{I18n, move_tr};

/// Compact version without optional sections
#[component]
fn ComparisonDetailItem(#[prop(into)] label_key: String, value: String) -> impl IntoView {
    let i18n = expect_context::<I18n>();

    view! {
        <div class="detail-item">
            <strong>{move || i18n.tr(&label_key)}" "</strong>
            {value}
        </div>
    }
}

#[component]
pub fn ComparisonRenderer(data: ComparisonData, request: Option<ReportRequest>) -> impl IntoView {
    let ctx = expect_context::<ChatContext>();
    let i18n = expect_context::<I18n>();
    let previous_report = ctx.report_by_id(&data.prev_id);
    let current_report = ctx.report_by_id(&data.next_id);
    let downloads = view! {
        <ReportDownloads
            export=request.and_then(|request| Some(ReportExport {
                request,
                sources: ExportSources::Comparison {
                    previous_id: data.prev_id.parse().ok()?,
                    current_id: data.next_id.parse().ok()?,
                },
                result: ExportResult {
                    description: data.description.clone(),
                    windows: data.windows.clone(),
                    doors: data.doors.clone(),
                    radiators: data.radiators.clone(),
                    openings: data.openings.clone(),
                    power_outlets: data.power_outlets.map(PowerOutletResult::Comparison),
                    model_name: None,
                },
            }))
            markdown=data.to_markdown()
            filename=data.filename()
        />
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
                    <ReportPreview label=data.prev_date report=previous_report />
                    <span class="left5 right5">" "{move_tr!("comparison-changes-to")}" "</span>
                    <i class="fas fa-image right5"></i>
                    <ReportPreview label=data.next_date report=current_report />
                </span>
            </div>
            <p class="compact-description">{data.description}</p>

            <div class="compact-details">
                {data.windows.map(|v| view! {
                    <ComparisonDetailItem label_key="detail-label-windows" value=v />})}
                {data.doors.map(|v| view! {
                    <ComparisonDetailItem label_key="detail-label-doors" value=v />})}
                {data.radiators.map(|v| view! {
                    <ComparisonDetailItem label_key="detail-label-radiators" value=v />})}
                {data.openings.map(|v| view! {
                    <ComparisonDetailItem label_key="detail-label-openings" value=v />})}
                {data.power_outlets.map(|counts| view! {
                    <div class="detail-item">
                        <strong>{move_tr!("detail-label-power-outlets-comparison")}" "</strong>
                        {move || counts.display(&i18n.tr("detail-outlet-uncertain"), &i18n.tr("detail-outlet-unavailable"))}
                    </div>
                })}
            </div>

            {downloads}
        </div>
    }
}
