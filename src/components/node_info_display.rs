use leptos::*;
use leptos::prelude::*;
use crate::components::tree::{NodeInfo, NodeType};

#[component]
pub fn NodeInfoDisplay(
    node_signal: RwSignal<Option<NodeInfo>>,
    #[prop(into)] on_node_click: Callback<NodeInfo>,
) -> impl IntoView {
    view! {
        {move || {
            node_signal
                .get()
                .map(|node| {
                    let node_clone = node.clone();
                    let icon_class = match node.node_type {
                        NodeType::Root => "fas fa-building",
                        NodeType::Branch => "fas fa-building",
                        NodeType::ImageLeaf => "fas fa-image",
                    };

                    view! {
                        <div class="node-info-item">
                            <button
                                class="delete-btn"
                                on:click=move |_| {
                                    on_node_click.run(node_clone.clone());
                                }
                                title="Delete"
                            >
                                <i class="fas fa-times"></i>
                            </button>
                            <i class=icon_class></i>
                            <span class="node-name">
                                {node.name.unwrap_or_else(|| "Unnamed".to_string())}
                            </span>
                        </div>
                    }
                })
        }}
    }
}