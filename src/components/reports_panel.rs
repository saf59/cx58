use crate::auth::Auth;
use crate::components::chat_context::ChatContext;
use crate::components::show_tree::DetailsTreeRendererWithContext;
use crate::components::tree::{NodeData, NodeInfo, NodeType, NodeWithLeaf, TreeViewerResource};
use js_sys::{Date, Reflect};
use leptos::prelude::*;
use leptos::wasm_bindgen::{JsCast, JsValue};
use leptos::{IntoView, component, view};
use leptos_fluent::move_tr;
use uuid::Uuid;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FormData, HtmlInputElement, Request, RequestInit, RequestMode, Response, window};

const REPORTS_PAGE_SIZE: usize = 10;

#[component]
pub fn ReportsObjectPicker() -> impl IntoView {
    let auth_signal = use_context::<RwSignal<Auth>>().expect("Auth must be provided");
    let ctx = use_context::<ChatContext>().expect("ChatContext must be provided");

    let email = auth_signal
        .get_untracked()
        .email()
        .unwrap_or("mock".to_string());
    let selected_node = RwSignal::new(None::<NodeInfo>);

    view! {
        <div class="reports-picker">
            <TreeViewerResource
                user_id=email
                with_leafs=false
                renderer=move |tree| {
                    view! {
                        <DetailsTreeRendererWithContext
                            tree=tree
                            on_node_click=move |node_info| {
                                ctx.set_parent(node_info.clone());
                                selected_node.set(Some(node_info));
                            }
                        />
                    }
                }
            />

        </div>
    }
}

#[component]
pub fn ReportsPage() -> impl IntoView {
    let ctx = use_context::<ChatContext>().expect("ChatContext must be provided");

    view! {
        <div class="reports-page">
            {move || {
                ctx.parent
                    .get()
                    .map(|node| view! { <SelectedReports node=node /> }.into_any())
                    .unwrap_or_else(|| {
                        view! {
                            <div class="reports-page-empty">
                                <i class="fas fa-images"></i>
                                <span>{move_tr!("reports-select-object")}</span>
                            </div>
                        }.into_any()
                    })
            }}
        </div>
    }
}
#[component]
fn SelectedReports(node: NodeInfo) -> impl IntoView {
    let reports = RwSignal::new(Vec::<NodeWithLeaf>::new());
    let loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let file_input_ref = NodeRef::<leptos::html::Input>::new();
    let datetime_input_ref = NodeRef::<leptos::html::Input>::new();
    let parent_id = node.id;
    let has_selected_file = RwSignal::new(false);
    let current_page = RwSignal::new(0usize);
    let media_proxy = media_proxy_rule();

    let reload = Action::new_unsync(move |_: &()| async move {
        loading.set(true);
        error.set(None);
        match fetch_reports(parent_id).await {
            Ok(mut data) => {
                data.retain(|item| item.node_type == NodeType::ImageLeaf);
                data.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                let max_page = max_page_index(data.len());
                current_page.update(|page| {
                    if *page > max_page {
                        *page = max_page;
                    }
                });
                reports.set(data);
            }
            Err(e) => error.set(Some(e)),
        }
        loading.set(false);
    });

    Effect::new(move |_| {
        reload.dispatch(());
    });

    let on_upload = move |_| {
        let Some(input) = file_input_ref.get() else {
            return;
        };
        let input: HtmlInputElement = input.unchecked_into();
        let Some(file_list) = input.files() else {
            return;
        };
        let Some(file) = file_list.item(0) else {
            return;
        };
        let datetime = datetime_input_ref
            .get()
            .map(|input| {
                let input: HtmlInputElement = input.unchecked_into();
                input.value()
            })
            .filter(|value| !value.is_empty())
            .unwrap_or_else(current_datetime_berlin);

        leptos::task::spawn_local(async move {
            match upload_report(parent_id, &file, &datetime).await {
                Ok(()) => {
                    if let Some(input) = file_input_ref.get() {
                        let input: HtmlInputElement = input.unchecked_into();
                        input.set_value("");
                    }
                    has_selected_file.set(false);
                    reload.dispatch(());
                }
                Err(e) => error.set(Some(e)),
            }
        });
    };

    view! {
        <div class="reports-selected">
            <div class="reports-upload">
                <input
                    id="reports-upload-datetime"
                    name="berlin_datetime"
                    type="text"
                    node_ref=datetime_input_ref
                    prop:value=current_datetime_agent()
                    aria-label=move || move_tr!("reports-datetime").get()
                />
                <input
                    id="reports-upload-file"
                    name="image"
                    type="file"
                    accept="image/*"
                    node_ref=file_input_ref
                    aria-label=move || move_tr!("reports-file").get()
                    on:change=move |ev| {
                        let input: HtmlInputElement = event_target(&ev);
                        has_selected_file.set(input.files().is_some_and(|files| files.length() > 0));
                    }
                />
                <button
                    type="button"
                    on:click=on_upload
                    disabled=move || !has_selected_file.get()
                    title=move || move_tr!("reports-upload").get()
                >
                    <i class="fas fa-upload"></i>
                </button>
            </div>

            {move || if loading.get() {
                view! { <div class="reports-status">{move_tr!("reports-loading")}</div> }.into_any()
            } else if let Some(e) = error.get() {
                view! { <div class="reports-status reports-error">{e}</div> }.into_any()
            } else if reports.get().is_empty() {
                view! { <div class="reports-status">{move_tr!("reports-empty")}</div> }.into_any()
            } else {
                let media_proxy = media_proxy.clone();
                view! {
                    <ReportsPaginator reports=reports current_page=current_page />
                    <div class="reports-list">
                        <For
                            each=move || paged_reports(reports.get(), current_page.get())
                            key=|report| report.id
                            children={
                                move |report| {
                                    let media_proxy = media_proxy.clone();
                                    view! {
                                        <ReportItem report=report reload=reload media_proxy=media_proxy />
                                    }
                                }
                            }
                        />
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn ReportsPaginator(
    reports: RwSignal<Vec<NodeWithLeaf>>,
    current_page: RwSignal<usize>,
) -> impl IntoView {
    let total = move || reports.get().len();
    let page_count = move || total().div_ceil(REPORTS_PAGE_SIZE).max(1);
    let from = move || {
        if total() == 0 {
            0
        } else {
            current_page.get() * REPORTS_PAGE_SIZE + 1
        }
    };
    let to = move || ((current_page.get() + 1) * REPORTS_PAGE_SIZE).min(total());

    view! {
        <Show when=move || { total() > REPORTS_PAGE_SIZE }>
            <div class="reports-pagination">
                <button
                    type="button"
                    class="reports-page-button"
                    disabled=move || current_page.get() == 0
                    title=move || move_tr!("reports-prev-page").get()
                    aria-label=move || move_tr!("reports-prev-page").get()
                    on:click=move |_| {
                        current_page.update(|page| *page = page.saturating_sub(1));
                    }
                >
                    <i class="fas fa-chevron-left"></i>
                </button>
                <span class="reports-page-count">
                    {move || format!("{}-{} / {}", from(), to(), total())}
                </span>
                <span class="reports-page-index">
                    {move || format!("{}/{}", current_page.get() + 1, page_count())}
                </span>
                <button
                    type="button"
                    class="reports-page-button"
                    disabled=move || current_page.get() + 1 >= page_count()
                    title=move || move_tr!("reports-next-page").get()
                    aria-label=move || move_tr!("reports-next-page").get()
                    on:click=move |_| {
                        current_page.update(|page| {
                            let max_page = page_count().saturating_sub(1);
                            if *page < max_page {
                                *page += 1;
                            }
                        });
                    }
                >
                    <i class="fas fa-chevron-right"></i>
                </button>
            </div>
        </Show>
    }
}

fn paged_reports(reports: Vec<NodeWithLeaf>, current_page: usize) -> Vec<NodeWithLeaf> {
    reports
        .into_iter()
        .skip(current_page * REPORTS_PAGE_SIZE)
        .take(REPORTS_PAGE_SIZE)
        .collect()
}

fn max_page_index(total: usize) -> usize {
    total.saturating_sub(1) / REPORTS_PAGE_SIZE
}

#[component]
fn ReportItem(report: NodeWithLeaf, reload: Action<(), ()>, media_proxy: String) -> impl IntoView {
    let report_id = report.id;
    let name = report
        .name
        .clone()
        .unwrap_or_else(|| move_tr!("reports-image").get());
    let thumbnail = image_url(&report, true, &media_proxy);
    let full_url = image_url(&report, false, &media_proxy);
    let popup_id = format!("report-popup-{}", Uuid::now_v7());
    let popup_target = popup_id.clone();
    let popup_close = popup_id.clone();
    let date_input_id = format!("report-date-{report_id}");
    let edit_value = RwSignal::new(datetime_edit_value(&report));
    let update_error = RwSignal::new(None::<String>);

    view! {
        <div class="reports-item">
            <button type="button" popovertarget=popup_target class="reports-thumb">
                <img crossorigin="anonymous" src=thumbnail alt=name.clone() loading="lazy" />
            </button>
            <div id=popup_id popover class="popup">
                <div class="popup-content">
                    <button popovertarget=popup_close class="popup-close">"×"</button>
                    <img crossorigin="anonymous" src=full_url alt=name.clone() class="popup-image" />
                </div>
            </div>
            <input
                id=date_input_id
                name="berlin_datetime"
                class="reports-date-edit"
                type="text"
                prop:value=move || edit_value.get()
                on:input=move |ev| edit_value.set(event_target_value(&ev))
            />
            <button
                type="button"
                class="reports-save-date"
                title=move || move_tr!("reports-save-date").get()
                on:click=move |_| {
                    let datetime = edit_value.get();
                    leptos::task::spawn_local(async move {
                        match update_report_date(report_id, &datetime).await {
                            Ok(()) => {
                                update_error.set(None);
                                reload.dispatch(());
                            }
                            Err(e) => update_error.set(Some(e)),
                        }
                    });
                }
            >
                <i class="fas fa-save"></i>
            </button>
            <button
                type="button"
                class="reports-delete"
                title=move || move_tr!("reports-delete").get()
                on:click=move |_| {
                    leptos::task::spawn_local(async move {
                        match delete_report(report_id).await {
                            Ok(()) => {
                                update_error.set(None);
                                reload.dispatch(());
                            }
                            Err(e) => update_error.set(Some(e)),
                        }
                    });
                }
            >
                <i class="fas fa-trash"></i>
            </button>
            <div class="reports-item-error" class:none=move || update_error.get().is_none()>
                {move || update_error.get().unwrap_or_default()}
            </div>
        </div>
    }
}
fn image_url(report: &NodeWithLeaf, thumbnail: bool, media_proxy: &str) -> String {
    match &report.data {
        NodeData::Image(data) if thumbnail => proxy_media(
            media_proxy,
            &data
                .thumbnail_url
                .clone()
                .or_else(|| data.url.clone())
                .unwrap_or_default(),
        ),
        NodeData::Image(data) => proxy_media(media_proxy, &data.url.clone().unwrap_or_default()),
        _ => String::new(),
    }
}

fn media_proxy_rule() -> String {
    window()
        .and_then(|w| {
            Reflect::get(&w, &JsValue::from_str("MEDIA_PROXY"))
                .ok()
                .and_then(|v| v.as_string())
        })
        .unwrap_or_default()
}

fn proxy_media(rule: &str, value: &str) -> String {
    if rule.is_empty() {
        return value.to_string();
    }
    let parts: Vec<&str> = rule.split(',').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return value.to_string();
    }
    value.replace(parts[0], parts[1])
}

async fn fetch_reports(node_id: Uuid) -> Result<Vec<NodeWithLeaf>, String> {
    let url = format!("/api/proxy/reports/{}", node_id);
    let resp = send_request("GET", &url, None).await?;
    let json = JsFuture::from(
        resp.json()
            .map_err(|e| format!("Failed to get JSON: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("Failed to parse JSON: {:?}", e))?;

    serde_wasm_bindgen::from_value(json).map_err(|e| format!("Failed to deserialize: {:?}", e))
}

async fn upload_report(
    node_id: Uuid,
    file: &web_sys::File,
    datetime_local: &str,
) -> Result<(), String> {
    let form = FormData::new().map_err(|e| format!("{e:?}"))?;
    let datetime = datetime_for_agent(datetime_local);
    validate_agent_datetime(&datetime)?;

    form.append_with_blob("image", file)
        .map_err(|e| format!("{e:?}"))?;
    form.append_with_str("berlin_datetime", &datetime)
        .map_err(|e| format!("{e:?}"))?;

    let url = format!("/api/proxy/images/upload/{}", node_id);
    let body = JsValue::from(form);
    let resp = send_request("POST", &url, Some(body)).await?;
    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Upload failed: HTTP {}", resp.status()))
    }
}

async fn update_report_date(node_id: Uuid, datetime_local: &str) -> Result<(), String> {
    let datetime = datetime_for_agent(datetime_local);
    validate_agent_datetime(&datetime)?;
    let body = serde_json::json!({
        "berlin_datetime": datetime,
    })
    .to_string();
    let resp = send_json_request("PUT", &format!("/api/proxy/reports/{}", node_id), &body).await?;
    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Update failed: HTTP {}", resp.status()))
    }
}

async fn send_json_request(method: &str, url: &str, body: &str) -> Result<Response, String> {
    let window = window().ok_or_else(|| "No window available".to_string())?;
    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::Cors);
    opts.set_body(&JsValue::from_str(body));
    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;
    request
        .headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Failed to set header: {:?}", e))?;
    request
        .headers()
        .set("Accept", "application/json")
        .map_err(|e| format!("Failed to set header: {:?}", e))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;
    resp_value
        .dyn_into()
        .map_err(|_| "Failed to convert to Response".to_string())
}

fn datetime_edit_value(report: &NodeWithLeaf) -> String {
    datetime_for_agent(&datetime_local_from_report(&report.updated_at))
}

fn datetime_local_from_report(value: &str) -> String {
    value.chars().take(16).collect::<String>().replace(' ', "T")
}

fn normalize_agent_datetime(value: &str) -> Option<String> {
    let mut parts = value.split(['.', ' ', ':']);
    let day = parts.next()?.trim();
    let month = parts.next()?.trim();
    let year = parts.next()?.trim();
    let hour = parts.next()?.trim();
    let minute = parts.next()?.trim();
    let second = parts.next().unwrap_or("00").trim();

    if day.len() != 2 || month.len() != 2 || year.len() != 4 || hour.len() != 2 || minute.len() != 2
    {
        return None;
    }

    Some(format!(
        "{}.{}.{} {}:{}:{}",
        day, month, year, hour, minute, second
    ))
}
async fn delete_report(node_id: Uuid) -> Result<(), String> {
    let url = format!("/api/proxy/images/{}", node_id);
    let resp = send_request("DELETE", &url, None).await?;
    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Delete failed: HTTP {}", resp.status()))
    }
}

async fn send_request(method: &str, url: &str, body: Option<JsValue>) -> Result<Response, String> {
    let window = window().ok_or_else(|| "No window available".to_string())?;
    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::Cors);
    if let Some(body) = body {
        opts.set_body(&body);
    }
    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;
    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Failed to convert to Response".to_string())?;
    if !resp.ok() && method == "GET" {
        return Err(format!("Request failed with status: {}", resp.status()));
    }
    Ok(resp)
}

fn current_datetime_berlin() -> String {
    let date = Date::new_0();
    let offset_hours = berlin_utc_offset_hours(
        date.get_utc_full_year(),
        date.get_utc_month() + 1,
        date.get_utc_date(),
        date.get_utc_hours(),
    );
    let berlin_date = Date::new(&JsValue::from_f64(
        date.get_time() + f64::from(offset_hours) * 60.0 * 60.0 * 1000.0,
    ));
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}",
        berlin_date.get_utc_full_year(),
        berlin_date.get_utc_month() + 1,
        berlin_date.get_utc_date(),
        berlin_date.get_utc_hours(),
        berlin_date.get_utc_minutes()
    )
}

fn current_datetime_agent() -> String {
    datetime_for_agent(&current_datetime_berlin())
}

fn berlin_utc_offset_hours(year: u32, month: u32, day: u32, hour_utc: u32) -> i32 {
    let in_dst = match month {
        4..=9 => true,
        1 | 2 | 11 | 12 => false,
        3 => {
            let start_day = last_sunday_day(year as i32, 3);
            day > start_day || (day == start_day && hour_utc >= 1)
        }
        10 => {
            let end_day = last_sunday_day(year as i32, 10);
            day < end_day || (day == end_day && hour_utc < 1)
        }
        _ => false,
    };
    if in_dst { 2 } else { 1 }
}

fn last_sunday_day(year: i32, month: u32) -> u32 {
    let mut day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 30,
    };
    while day_of_week(year, month, day) != 0 {
        day -= 1;
    }
    day
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn day_of_week(year: i32, month: u32, day: u32) -> u32 {
    let (month, year) = if month < 3 {
        (month as i32 + 12, year - 1)
    } else {
        (month as i32, year)
    };
    let k = year % 100;
    let j = year / 100;
    let h = (day as i32 + (13 * (month + 1)) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    ((h + 6) % 7) as u32
}

fn validate_agent_datetime(datetime: &str) -> Result<(), String> {
    let mut parts = datetime.split(['.', ' ', ':']);
    let _day = parts.next().ok_or_else(|| "Invalid datetime".to_string())?;
    let _month = parts.next().ok_or_else(|| "Invalid datetime".to_string())?;
    let year = parts
        .next()
        .ok_or_else(|| "Invalid datetime".to_string())?
        .parse::<i32>()
        .map_err(|_| "Invalid datetime year".to_string())?;
    if !(2000..=2100).contains(&year) {
        return Err("Invalid datetime year".to_string());
    }
    Ok(())
}
fn datetime_for_agent(datetime_value: &str) -> String {
    if let Some(datetime) = normalize_agent_datetime(datetime_value) {
        return datetime;
    }

    let mut parts = datetime_value.split(['-', 'T', ':']);
    let year = parts.next().unwrap_or("1970");
    let month = parts.next().unwrap_or("01");
    let day = parts.next().unwrap_or("01");
    let hour = parts.next().unwrap_or("00");
    let minute = parts.next().unwrap_or("00");
    format!("{}.{}.{} {}:{}:00", day, month, year, hour, minute)
}
