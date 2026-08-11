use crate::components::tree::NodeInfo;
use leptos::prelude::*;
use leptos_fluent::move_tr;

#[component]
pub fn ReportPreview(label: String, report: Option<NodeInfo>) -> impl IntoView {
    let image_url = report.and_then(|report| {
        report
            .thumbnail_url
            .clone()
            .or(report.full_url.clone())
            .map(|url| (url, report.name.unwrap_or_else(|| label.clone())))
    });

    match image_url {
        Some((url, alt)) => {
            let accessible_label = format!("{}: {}", move_tr!("selected-report-open").get(), label);
            view! {
                <span
                    class="compact-report-preview"
                    tabindex="0"
                    aria-label=accessible_label
                >
                    <span class="compact-report-label">{label}</span>
                    <span class="compact-report-popup" role="tooltip">
                        <img
                            crossorigin="anonymous"
                            src=url
                            alt=alt
                            class="thumbnail"
                            loading="lazy"
                        />
                    </span>
                </span>
            }
            .into_any()
        }
        None => view! { <span class="compact-report-label">{label}</span> }.into_any(),
    }
}
