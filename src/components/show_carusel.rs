#![allow(unused_variables)]
#![allow(dead_code)]
use leptos::*;
use leptos::prelude::ElementChild;
use leptos::prelude::ClassAttribute;
use leptos::prelude::GlobalAttributes;
use leptos::prelude::IntoAny;
use crate::components::tree::{NodeData, NodeType, Tree};

/// Carousel renderer for a single Branch node with ImageLeaf children
/// Displays node info and a 2-image-wide carousel with CSS popup
#[component]
pub fn CarouselRenderer(tree: Vec<Tree>) -> impl IntoView {
    // Expecting exactly one Branch node
    let branch = tree.into_iter().next();
    
    match branch {
        Some(node) => {
            let node_name = node.name.clone().unwrap_or_else(|| "Unnamed".to_string());
            let images: Vec<_> = node.children.into_iter()
                .filter(|child| child.node_type == NodeType::ImageLeaf)
                .collect();
            
            view! {
                <div class="carousel-container">
                    <div class="node-info">
                        <h2>{node_name}</h2>
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
                                {node.updated_at}
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
                                                .name
                                                .unwrap_or_else(|| format!("Image {}", idx + 1));
                                            let popup_id = format!("popup-{}", idx);

                                            view! {
                                                <div class="carousel-item">
                                                    <a href=format!("#{}", popup_id) class="thumbnail-link">
                                                        <img
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
                                                            <img src=full_url alt=img_name class="popup-image" />
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

                <style>
                    {r#"
                    .carousel-container {
                    width: 100%;
                    max-width: 1200px;
                    margin: 0 auto;
                    padding: 1rem;
                    }
                    
                    .node-info {
                    margin-bottom: 1.5rem;
                    padding: 1rem;
                    background: #f8f9fa;
                    border-radius: 8px;
                    }
                    
                    .node-info h2 {
                    margin: 0 0 0.5rem 0;
                    color: #333;
                    }
                    
                    .node-meta {
                    display: flex;
                    gap: 1.5rem;
                    color: #666;
                    font-size: 0.9rem;
                    }
                    
                    .meta-item {
                    display: flex;
                    align-items: center;
                    gap: 0.3rem;
                    }
                    
                    .no-images {
                    text-align: center;
                    padding: 3rem;
                    color: #999;
                    }
                    
                    /* Carousel with Scroll Snap */
                    .carousel-wrapper {
                    position: relative;
                    overflow: hidden;
                    border-radius: 8px;
                    }
                    
                    .carousel {
                    display: grid;
                    grid-auto-flow: column;
                    grid-auto-columns: 50%; /* 2 images visible */
                    gap: 1rem;
                    overflow-x: auto;
                    scroll-snap-type: x mandatory;
                    scroll-behavior: smooth;
                    padding: 1rem;
                    scrollbar-width: thin;
                    scrollbar-color: #888 #f1f1f1;
                    }
                    
                    .carousel::-webkit-scrollbar {
                    height: 8px;
                    }
                    
                    .carousel::-webkit-scrollbar-track {
                    background: #f1f1f1;
                    border-radius: 4px;
                    }
                    
                    .carousel::-webkit-scrollbar-thumb {
                    background: #888;
                    border-radius: 4px;
                    }
                    
                    .carousel::-webkit-scrollbar-thumb:hover {
                    background: #555;
                    }
                    
                    .carousel-item {
                    scroll-snap-align: start;
                    position: relative;
                    }
                    
                    .thumbnail-link {
                    display: block;
                    position: relative;
                    overflow: hidden;
                    border-radius: 8px;
                    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
                    transition: transform 0.2s, box-shadow 0.2s;
                    text-decoration: none;
                    }
                    
                    .thumbnail-link:hover {
                    transform: translateY(-4px);
                    box-shadow: 0 4px 16px rgba(0,0,0,0.2);
                    }
                    
                    .thumbnail {
                    width: 100%;
                    height: 300px;
                    object-fit: cover;
                    display: block;
                    }
                    
                    .image-label {
                    position: absolute;
                    bottom: 0;
                    left: 0;
                    right: 0;
                    background: linear-gradient(to top, rgba(0,0,0,0.7), transparent);
                    color: white;
                    padding: 1rem 0.75rem 0.75rem;
                    font-size: 0.9rem;
                    }
                    
                    /* CSS Popup */
                    .popup {
                    display: none;
                    position: fixed;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100%;
                    background: rgba(0, 0, 0, 0.9);
                    z-index: 9999;
                    align-items: center;
                    justify-content: center;
                    animation: fadeIn 0.3s;
                    }
                    
                    .popup:target {
                    display: flex;
                    }
                    
                    @keyframes fadeIn {
                    from { opacity: 0; }
                    to { opacity: 1; }
                    }
                    
                    .popup-content {
                    position: relative;
                    max-width: 90%;
                    max-height: 90%;
                    animation: zoomIn 0.3s;
                    }
                    
                    @keyframes zoomIn {
                    from { transform: scale(0.8); }
                    to { transform: scale(1); }
                    }
                    
                    .popup-close {
                    position: absolute;
                    top: -40px;
                    right: 0;
                    color: white;
                    font-size: 40px;
                    font-weight: bold;
                    text-decoration: none;
                    line-height: 1;
                    transition: color 0.2s;
                    }
                    
                    .popup-close:hover {
                    color: #ff4444;
                    }
                    
                    .popup-image {
                    max-width: 100%;
                    max-height: 90vh;
                    object-fit: contain;
                    border-radius: 4px;
                    }
                    
                    /* Responsive adjustments */
                    @media (max-width: 768px) {
                    .carousel {
                        grid-auto-columns: 100%; /* 1 image on mobile */
                    }
                    
                    .thumbnail {
                        height: 250px;
                    }
                    }
                    "#}
                </style>
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

// Usage example:
// <TreeViewerResource
//     user_id="user@example.com".to_string()
//     with_leafs=true
//     renderer=|tree| view! { <CarouselRenderer tree=tree /> }
// />