#![allow(unused_variables)]
#![allow(dead_code)]
use crate::components::chat_context::ChatContext;
use crate::components::tree::{NodeData, NodeType, NodeWithLeaf};
use leptos::context::use_context;
use leptos::logging::log;
use leptos::prelude::ClassAttribute;
use leptos::prelude::ElementChild;
use leptos::prelude::IntoAny;
use leptos::prelude::OnAttribute;
use leptos::prelude::GlobalAttributes;
use leptos::*;
use uuid::Uuid;

/// Carousel renderer for a single Branch node with ImageLeaf children
/// Displays node info and a 2-image-wide carousel with CSS popup
#[component]
pub fn CarouselRenderer(data: Vec<NodeWithLeaf>) -> impl IntoView {
    use web_sys::window;
    // Expecting exactly one Branch node
    log!("CarouselRenderer with {} nodes", &data.len());

    let media_proxy = window()
        .and_then(|w| {
            js_sys::Reflect::get(&w, &wasm_bindgen::JsValue::from_str("MEDIA_PROXY"))
                .ok()
                .and_then(|v| v.as_string())
        })
        .unwrap_or_default();

    log!("Media proxy: {}", media_proxy);
    let ctx = use_context::<ChatContext>().expect("Context lost");
    let branches = data.iter().find(|n| n.node_type == NodeType::Branch);
    match branches {
        Some(branch) => {
            let branch = branch.clone();
            let mut images: Vec<NodeWithLeaf> = data.into_iter().filter(|n| n.node_type == NodeType::ImageLeaf).collect();
            images.sort_by_key(|p| std::cmp::Reverse( p.updated_at.clone()));
            view! {
                <div class="carousel-container">
                    <div class="node-info">
                        <h3>{branch.name.clone()}</h3>
                        <div class="node-meta">
                            <span class="meta-item">
                                <i class="fas fa-images"></i>
                                " "
                                {images.len()}
                                " "
                                "reports"
                            </span>
                        </div>
                    </div>

                    {if images.is_empty() {
                        view! {
                            <div class="no-images">
                                <p>"No images available"</p>
                            </div>
                        }
                            .into_any()
                    } else {
                        view! {
                            <div class="carousel-wrapper">
                                <div class="carousel">
                                    {images
                                        .into_iter()
                                        .enumerate()
                                        .map(|(idx, img)| {
                                            let thumbnail = match &img.data {
                                                NodeData::Image(img_data) => {
                                                    log!("ImageData: {:?}", &img_data.thumbnail_url);
                                                    let img = img_data
                                                        .thumbnail_url
                                                        .clone()
                                                        .unwrap_or_else(|| {
                                                            img_data.url.clone().unwrap_or_default()
                                                        });
                                                    proxy_media(&media_proxy, &img)
                                                }
                                                _ => String::new(),
                                            };
                                            let full_url = match &img.data {
                                                NodeData::Image(img_data) => {
                                                    img_data.url.clone().unwrap_or_default()
                                                }
                                                _ => String::new(),
                                            };
                                            let full_url = proxy_media(&media_proxy, &full_url);
                                            let img_name = img
                                                .name
                                                .clone()
                                                .unwrap_or_else(|| format!("Image {}", idx + 1));
                                            let popup_id = format!("popup-{}", Uuid::now_v7());
                                            let popup_id_for_click = popup_id.clone();
                                            log!(
                                                "thumbnail: {}, full_url: {}, img_name: {}, popup_id: {}", thumbnail, full_url, img_name, popup_id
                                            );

                                            let img_clone_for_label = img.clone();
                                            let branch_clone_for_label = branch.clone();
                                            //let ctx_clone = ctx.clone();

                                        /*                                            // Clone необходимые данные для обработчиков событий

                                            // Store node data in Store for event handlers
                                            let img_stored = leptos::prelude::StoredValue::new(img.clone());
                                            let ctx_stored = leptos::prelude::StoredValue::new(ctx.clone());
*/
                                            // Store node data in Store for event handlers
                                            //let img_stored = leptos::prelude::StoredValue::new(img.clone());
                                            //let branch_stored = leptos::prelude::StoredValue::new(branch.clone());
                                            //let ctx_stored = leptos::prelude::StoredValue::new(ctx.clone());

                                            view! {
                                                <div class="carousel-item">
                                                    <button
                                                        popovertarget=popup_id.clone()
                                                        class="thumbnail-link"
                                                    >
                                                        <img
                                                            crossorigin="anonymous"
                                                            src=thumbnail
                                                            alt=img_name.clone()
                                                            class="thumbnail"
                                                            loading="lazy"
                                                        />
                                                    </button>
                                                    <div
                                                        class="image-label"
                                                        on:click=move |ev| {
                                                            log!("Label clicked!");
                                                            //let img_val = img_clone_for_label.get_value();
                                                            //let branch_val = branch_clone_for_label.get_value();
/*                                                            let ctx_val = ctx.get_value();
                                                            log!("Calling set_leaf with img: {:?}, branch: {:?}",
                                                                img_val.name,
                                                                branch_val.name);
*/                                                            ctx.set_leaf(&img_clone_for_label, &branch_clone_for_label);
                                                        }

/*                                                        on:click=move |_| {
                                                            ctx_clone.set_leaf(&img_clone_for_label, &branch_clone_for_label);
                                                        }
*/
/*                                            on:click=move |ev| {
                                                            log!("Label clicked!");
                                                            let img_val = img_stored.get_value();
                                                            let ctx_val = ctx_stored.get_value();
                                                            let name = img_val.name.clone().unwrap_or("(unnamed)".to_string());
                                                            log!("Setting insert_text to: {}", name);
                                                            ctx_val.insert_text.set(Some(name));
                                                        }
*/
                                            >
                                                        {img_name.clone()}
                                                    </div>
                                                </div>

                                                <div id=popup_id popover class="popup">
                                                    <div class="popup-content">
                                                        <button popovertarget=popup_id.clone() class="popup-close">
                                                            "×"
                                                        </button>
                                                        <img
                                                            crossorigin="anonymous"
                                                            src=full_url
                                                            alt=img_name
                                                            class="popup-image"
                                                        />
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </div>
                            </div>
                        }
                            .into_any()
                    }}
                </div>
            }.into_any()
        }
        None => {
            view! {
                <div class="carousel-container">
                    <p>"No data available"</p>
                </div>
            }.into_any()
        }
    }
}
fn proxy_media(rule: &str, value: &str) -> String {
    if rule.is_empty() {
        return value.to_string();
    }
    let parts: Vec<&str> = rule.split(',').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return value.to_string();
    }
    let old_value = parts[0];
    let new_value = parts[1];
    value.replace(old_value, new_value)
}