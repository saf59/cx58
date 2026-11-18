use crate::components::tree::{NodeInfo, NodeType, Tree};
use crate::cx_id::{name_with_cx_id, use_cx_ids};
use leptos::ev::MouseEvent;
use leptos::prelude::{
    Callable, Callback, ClassAttribute, ElementChild, IntoAny, OnAttribute, expect_context,
};
use leptos::*;
use leptos_fluent::I18n;
use uuid::Uuid;

#[component]
pub fn DetailsTreeRendererWithContext(
    tree: Vec<Tree>,
    on_node_click: impl Fn(NodeInfo) + 'static + Clone,
) -> impl IntoView {
    fn collect_owned_ids(nodes: &[Tree], ids: &mut Vec<Uuid>) {
        for node in nodes {
            if node.own {
                ids.push(node.id);
            }
            collect_owned_ids(&node.children, ids);
        }
    }
    let mut ids = Vec::new();
    collect_owned_ids(&tree, &mut ids);
    let cx_ids = use_cx_ids(move || ids.clone());
    view! {
        <div class="obj-area tree">
            {tree
                .into_iter()
                .map(|node| {
                    view! {
                        <DetailsTreeNodeWithContext node=node on_node_click=on_node_click.clone() cx_ids=cx_ids />
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
    cx_ids: Callback<Uuid, Option<String>>,
) -> impl IntoView {
    let i18n = expect_context::<I18n>();
    let has_children = node.node_type != NodeType::ImageLeaf && !node.children.is_empty();
    let is_own = node.own;
    let node_id = node.id;
    let node_name = node
        .name
        .clone()
        .unwrap_or_else(|| i18n.tr("tree-node-unnamed"));
    let display_name = move || name_with_cx_id(&node_name, cx_ids.run(node_id).as_deref());
    let children = node.children.clone();

    let icon_class = match node.node_type {
        NodeType::Root => "fas fa-building",
        NodeType::Branch => "fas fa-building",
        NodeType::ImageLeaf => "fas fa-image",
    };

    let select_node = on_node_click.clone();
    let content = view! {
        <span
            class="node-content"
            class:clickable=is_own
            on:click=move |e: MouseEvent| {
                if is_own {
                    select_node(node.node_info());
                    e.stop_propagation();
                    e.prevent_default();
                }
            }
        >
            <i class=icon_class></i>
            {display_name}
        </span>
    };

    if has_children {
        view! {
            <details>
                <summary>
                    {content}
                </summary>
                {children
                    .into_iter()
                    .map(|child| {
                        view! {
                            <DetailsTreeNodeWithContext
                                node=child
                                on_node_click=on_node_click.clone()
                                cx_ids=cx_ids
                            />
                        }
                    })
                    .collect::<Vec<_>>()}
            </details>
        }
        .into_any()
    } else {
        view! {
            <div class="leaf">{content}</div>
        }
        .into_any()
    }
}
