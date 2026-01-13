use leptos::*;
use leptos::ev::MouseEvent;
use leptos::prelude::{ClassAttribute, IntoAny, OnAttribute};
use crate::components::tree::{NodeType, Tree};
use leptos::prelude::ElementChild;
/// Details-based tree renderer with Font Awesome icons
#[component]
pub fn DetailsTreeRenderer(tree: Vec<Tree>) -> impl IntoView {
    view! {
        <div class="details-tree">
            {tree.into_iter().map(|node| {
                view! { <DetailsTreeNode node=node /> }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn DetailsTreeNode(node: Tree) -> impl IntoView {
    let has_children = !node.children.is_empty();
    let is_leaf = node.node_type == NodeType::ImageLeaf;
    let is_own = node.own;
    let node_name = node.name.clone().unwrap_or_else(|| "(unnamed)".to_string());
    let children = node.children.clone();

    // Determine icon based on node type
    let icon_class = match node.node_type {
        NodeType::Root => "fas fa-building",
        NodeType::Branch => "fas fa-building",
        NodeType::ImageLeaf => "fas fa-image",
    };

    if is_leaf {
        // Leaf node - render as div
        view! {
            <div class="leaf">
                <span 
                    class="node-content"
                    class:clickable=is_own
                    on:click=move |e: MouseEvent| {
                        if is_own {
                            logging::log!("Clicked on {}", node_name);
                            // You can add your context logic here
                            // ctx.insert_text.set(Some(node_name.clone()));
                            e.stop_propagation();
                            e.prevent_default();
                        }
                    }
                >
                    <i class={icon_class}></i>
                    {node_name.clone()}
                </span>
            </div>
        }.into_any()
    } else if has_children {
        // Branch node with children - render as details
        view! {
            <details>
                <summary>
                    <span 
                        class="node-content"
                        class:clickable=is_own
                        on:click=move |e: MouseEvent| {
                            if is_own {
                                logging::log!("Clicked on {}", node_name);
                                // You can add your context logic here
                                // ctx.insert_text.set(Some(node_name.clone()));
                                e.stop_propagation();
                                e.prevent_default();
                            }
                        }
                    >
                        <i class={icon_class}></i>
                        {node_name.clone()}
                    </span>
                </summary>
                {children.into_iter().map(|child| {
                    view! { <DetailsTreeNode node=child /> }
                }).collect::<Vec<_>>()}
            </details>
        }.into_any()
    } else {
        // Branch node without children
        view! {
            <div class="leaf">
                <span 
                    class="node-content"
                    class:clickable=is_own
                    on:click=move |e: MouseEvent| {
                        if is_own {
                            logging::log!("Clicked on {}", node_name);
                            // You can add your context logic here
                            // ctx.insert_text.set(Some(node_name.clone()));
                            e.stop_propagation();
                            e.prevent_default();
                        }
                    }
                >
                    <i class={icon_class}></i>
                    {node_name.clone()}
                </span>
            </div>
        }.into_any()
    }
}

/// Version with context parameter for insert_text
#[component]
pub fn DetailsTreeRendererWithContext(
    tree: Vec<Tree>,
    on_node_click: impl Fn(String) + 'static + Clone,
) -> impl IntoView {
    view! {
        <div class="obj-area tree">
            {tree.into_iter().map(|node| {
                view! { <DetailsTreeNodeWithContext node=node on_node_click=on_node_click.clone() /> }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn DetailsTreeNodeWithContext(
    node: Tree,
    on_node_click: impl Fn(String) + 'static + Clone,
) -> impl IntoView {
    let has_children = !node.children.is_empty();
    let is_leaf = node.node_type == NodeType::ImageLeaf;
    let is_own = node.own;
    let node_name = node.name.clone().unwrap_or_else(|| "(unnamed)".to_string());
    let children = node.children.clone();

    let icon_class = match node.node_type {
        NodeType::Root => "fas fa-building",
        NodeType::Branch => "fas fa-building",
        NodeType::ImageLeaf => "fas fa-image",
    };

    if is_leaf {
        view! {
            <div class="leaf">
                <span 
                    class="node-content"
                    class:clickable=is_own
                    on:click=move |e: MouseEvent| {
                        if is_own {
                            logging::log!("Clicked on {}", node_name);
                            on_node_click(node_name.clone());
                            e.stop_propagation();
                            e.prevent_default();
                        }
                    }
                >
                    <i class={icon_class}></i>
                    {node_name.clone()}
                </span>
            </div>
        }.into_any()
    } else if has_children {
        view! {
            <details>
                <summary>
                    <span 
                        class="node-content"
                        class:clickable=is_own
                        on:click= {
                         let value = on_node_click.clone();
                            move |e: MouseEvent| {
                            if is_own {
                                logging::log!("Clicked on {}", node_name);
                                value(node_name.clone());
                                e.stop_propagation();
                                e.prevent_default();
                            }
                        }
                        }
                    >
                        <i class={icon_class}></i>
                        {node_name.clone()}
                    </span>
                </summary>
                {children.into_iter().map(|child| {
                    view! { <DetailsTreeNodeWithContext node=child on_node_click=on_node_click.clone() /> }
                }).collect::<Vec<_>>()}
            </details>
        }.into_any()
    } else {
        view! {
            <div class="leaf">
                <span 
                    class="node-content"
                    class:clickable=is_own
                    on:click=move |e: MouseEvent| {
                        if is_own {
                            logging::log!("Clicked on {}", node_name);
                            on_node_click(node_name.clone());
                            e.stop_propagation();
                            e.prevent_default();
                        }
                    }
                >
                    <i class={icon_class}></i>
                    {node_name.clone()}
                </span>
            </div>
        }.into_any()
    }
}
