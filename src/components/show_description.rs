use crate::components::chat_context::ChatContext;
use crate::components::chat_data::{DescriptionData, extract_name_pair};
use crate::components::report_downloads::ReportDownloads;
use crate::components::report_preview::ReportPreview;
use crate::report_export::{
    ExportResult, ExportSources, PowerOutletResult, ReportExport, ReportRequest,
};
use leptos::prelude::*;
use leptos_fluent::{I18n, move_tr};

/// Component to render multiple descriptions
#[component]
pub fn DescriptionListRenderer(
    /// List of description data to display
    data: Vec<DescriptionData>,
    request: Option<ReportRequest>,
) -> impl IntoView {
    view! {
        <div class="description-list">
            <For
                each=move || data.clone()
                key=|item| format!("{}-{}", item.object, item.date_id)
                children=move |item| {
                    view! { <DescriptionRendererCompact data=item request=request.clone() /> }
                }
            />
        </div>
    }
}

/// Compact version without optional sections
#[component]
fn DetailItem(#[prop(into)] label_key: String, value: String) -> impl IntoView {
    let i18n = expect_context::<I18n>();

    view! {
        <div class="detail-item">
            <strong>{move || i18n.tr(&label_key)}" "</strong>
            {value}
        </div>
    }
}

#[component]
pub fn DescriptionRendererCompact(
    data: DescriptionData,
    request: Option<ReportRequest>,
) -> impl IntoView {
    let ctx = expect_context::<ChatContext>();
    let i18n = expect_context::<I18n>();
    let (object_name, report_name) = extract_name_pair(data.object.as_str());
    let report = ctx.report_by_id(&data.date_id);

    let downloads = view! {
        <ReportDownloads
            export=request.and_then(|request| Some(ReportExport {
                request,
                sources: ExportSources::Description { report_id: data.date_id.parse().ok()? },
                result: ExportResult {
                    description: data.description.clone(),
                    windows: data.windows.clone(),
                    doors: data.doors.clone(),
                    radiators: data.radiators.clone(),
                    openings: data.openings.clone(),
                    power_outlets: data.power_outlets.map(PowerOutletResult::Photo),
                    model_name: Some(data.model_name.clone()),
                },
            }))
            markdown=data.to_markdown()
            filename=data.filename()
        />
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
                    <ReportPreview label=report_name report=report />
                </span>
            </div>
            <p class="compact-description">{data.description}</p>

            <div class="compact-details">
                {data.windows.map(|v| view! { <DetailItem label_key="detail-label-windows" value=v /> })}
                {data.doors.map(|v| view! { <DetailItem label_key="detail-label-doors" value=v /> })}
                {data.radiators.map(|v| view! { <DetailItem label_key="detail-label-radiators" value=v /> })}
                {data.openings.map(|v| view! { <DetailItem label_key="detail-label-openings" value=v /> })}
                {data.power_outlets.map(|count| view! {
                    <div class="detail-item">
                        <strong>{move_tr!("detail-label-power-outlets")}" "</strong>
                        {move || count.display(&i18n.tr("detail-outlet-uncertain"))}
                    </div>
                })}
            </div>

            {downloads}
        </div>
    }
}
