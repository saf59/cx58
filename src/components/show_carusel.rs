#![allow(unused_variables)]
#![allow(dead_code)]
use leptos::*;
use leptos::logging::log;
use leptos::prelude::ElementChild;
use leptos::prelude::ClassAttribute;
use leptos::prelude::GlobalAttributes;
use leptos::prelude::IntoAny;
use uuid::Uuid;
use crate::components::tree::{NodeData, NodeType, NodeWithLeaf, Tree};

/// Carousel renderer for a single Branch node with ImageLeaf children
/// Displays node info and a 2-image-wide carousel with CSS popup
#[component]
pub fn CarouselRenderer(data: Vec<NodeWithLeaf>) -> impl IntoView {
    // Expecting exactly one Branch node
    log!("CarouselRenderer with {} nodes", &data.len());
    let branches = data.iter().find(|n| n.node_type == NodeType::Branch);
    match branches {
        Some(branch) => {
            let branch = branch.clone();
            let mut images:Vec<NodeWithLeaf> = data.into_iter().filter(|n| n.node_type == NodeType::ImageLeaf).collect();
            images.sort_by_key(|p| p.updated_at.clone());
            view! {
                <div class="carousel-container">
                    <div class="node-info">
                        <h2>{branch.name.clone()}</h2>
                        <div class="node-meta">
                            <span class="meta-item">
                                <i class="fas fa-images"></i>
                                " "
                                {images.len()}
                                " images"
                            </span>
                            <span class="meta-item">
                                <i class="fas fa-clock"></i>
                                " "
                                {branch.updated_at.clone()}
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
                                            log!("Data: {:?}", &img.data);
                                            let thumbnail = match &img.data {
                                                NodeData::Image(img_data) => {
                                                log!("ImageData: {:?}", &img_data);
                                                    img_data
                                                        .thumbnail_url
                                                        .clone()
                                                        .unwrap_or_else(|| img_data.url.clone().unwrap_or_default())
                                                }
                                                _ => String::new(),
                                            };
                                            let full_url = match &img.data {
                                                NodeData::Image(img_data) => {
                                                    img_data.url.clone().unwrap_or_default()
                                                }
                                                _ => String::new(),
                                            };
                                            let img_name = img
                                                .name.clone()
                                                .unwrap_or_else(|| format!("Image {}", idx + 1));
                                            let popup_id = format!("popup-{}", Uuid::now_v7());
                                            log!("thumbnail: {}, full_url: {}, img_name: {}, popup_id: {}", thumbnail, full_url, img_name, popup_id);
                                            view! {
                                                <div class="carousel-item">
                                                    <a href=format!("#{}", popup_id) class="thumbnail-link">
                                                        <img crossorigin="anonymous"
                                                            src=thumbnail
                                                            alt=img_name.clone()
                                                            class="thumbnail"
                                                            loading="lazy"
                                                        />
                                                        <div class="image-label">{img_name.clone()}</div>
                                                    </a>

                                                    // CSS Popup
                                                    <div id=popup_id class="popup">
                                                        <div class="popup-content">
                                                            <a href="#" class="popup-close">
                                                                "×"
                                                            </a>
                                                            <img crossorigin="anonymous" src=full_url alt=img_name class="popup-image" />
                                                        </div>
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
