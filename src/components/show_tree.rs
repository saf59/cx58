use crate::components::tree::{NodeInfo, NodeType, Tree};
use leptos::ev::MouseEvent;
use leptos::prelude::{expect_context, ElementChild};
use leptos::prelude::{ClassAttribute, IntoAny, OnAttribute};
use leptos::*;
use leptos_fluent::I18n;

#[component]
pub fn DetailsTreeRendererWithContext(
    tree: Vec<Tree>,
    on_node_click: impl Fn(NodeInfo) + 'static + Clone,
) -> impl IntoView {
    view! {
        <div class="obj-area tree">
            {tree
                .into_iter()
                .map(|node| {
                    view! {
                        <DetailsTreeNodeWithContext node=node on_node_click=on_node_click.clone() />
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn DetailsTreeNodeWithContext(
    node: Tree,
    on_node_click: impl Fn(NodeInfo) + 'static + Clone,
) -> impl IntoView {
    let i18n = expect_context::<I18n>();
    let has_children = !node.children.is_empty();
    let is_leaf = node.node_type == NodeType::ImageLeaf;
    let is_own = node.own;
    let node_name = node
        .name
        .clone()
        .unwrap_or_else(|| i18n.tr("tree-node-unnamed"));
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
                            on_node_click(node.node_info());
                            e.stop_propagation();
                            e.prevent_default();
                        }
                    }
                >
                    <i class=icon_class></i>
                    {node_name.clone()}
                </span>
            </div>
        }
        .into_any()
    } else if has_children {
        view! {
            <details>
                <summary>
                    <span
                        class="node-content"
                        class:clickable=is_own
                        on:click={
                            let value = on_node_click.clone();
                            move |e: MouseEvent| {
                                if is_own {
                                    value(node.node_info());
                                    e.stop_propagation();
                                    e.prevent_default();
                                }
                            }
                        }
                    >
                        <i class=icon_class></i>
                        {node_name.clone()}
                    </span>
                </summary>
                {children
                    .into_iter()
                    .map(|child| {
                        view! {
                            <DetailsTreeNodeWithContext
                                node=child
                                on_node_click=on_node_click.clone()
                            />
                        }
                    })
                    .collect::<Vec<_>>()}
            </details>
        }
        .into_any()
    } else {
        view! {
            <div class="leaf">
                <span
                    class="node-content"
                    class:clickable=is_own
                    on:click=move |e: MouseEvent| {
                        if is_own {
                            on_node_click(node.node_info());
                            e.stop_propagation();
                            e.prevent_default();
                        }
                    }
                >
                    <i class=icon_class></i>
                    {node_name.clone()}
                </span>
            </div>
        }
        .into_any()
    }
}
