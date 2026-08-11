use crate::components::tree::{NodeInfo, NodeType};
use leptos::prelude::*;
use leptos_fluent::move_tr;

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

                    let display_name = node
                        .name
                        .clone()
                        .unwrap_or_else(|| move_tr!("selected-context-unnamed").get());

                    if node.node_type == NodeType::ImageLeaf {
                        let popup_id = format!("selected-report-popup-{}", node.id);
                        let popup_target = popup_id.clone();
                        let popup_close = popup_id.clone();
                        let thumbnail_url = node.thumbnail_url.clone().or_else(|| node.full_url.clone());
                        let full_url = node.full_url.clone().or_else(|| thumbnail_url.clone());
                        let open_label = move_tr!("selected-report-open").get();
                        let remove_label = move_tr!("selected-report-remove").get();
                        let open_accessible_label = format!("{}: {}", open_label, display_name);

                        view! {
                            <div class="node-info-item selected-report-item">
                                <button
                                    type="button"
                                    class="selected-report-open"
                                    popovertarget=popup_target
                                    title=open_accessible_label.clone()
                                    aria-label=open_accessible_label
                                    disabled=full_url.is_none()
                                >
                                    {thumbnail_url
                                        .map(|url| view! {
                                            <img
                                                crossorigin="anonymous"
                                                src=url
                                                alt=display_name.clone()
                                                loading="lazy"
                                            />
                                        })}
                                    <span class="selected-report-name">{display_name.clone()}</span>
                                </button>
                                <button
                                    type="button"
                                    class="delete-btn selected-report-remove"
                                    on:click=move |_| on_node_click.run(node_clone.clone())
                                    title=remove_label.clone()
                                    aria-label=format!("{}: {}", remove_label, display_name)
                                >
                                    <i class="fas fa-times"></i>
                                </button>
                            </div>
                            {full_url.map(|url| view! {
                                <div id=popup_id popover class="popup">
                                    <div class="popup-content">
                                        <button
                                            type="button"
                                            popovertarget=popup_close
                                            class="popup-close"
                                            aria-label=move_tr!("selected-report-close")
                                        >
                                            "×"
                                        </button>
                                        <img
                                            crossorigin="anonymous"
                                            src=url
                                            alt=display_name
                                            class="popup-image"
                                        />
                                    </div>
                                </div>
                            })}
                        }.into_any()
                    } else {
                        view! {
                            <div class="node-info-item">
                            <button
                                type="button"
                                class="delete-btn"
                                on:click=move |_| {
                                    on_node_click.run(node_clone.clone());
                                }
                                title=move_tr!("selected-context-remove")
                                aria-label=move_tr!("selected-context-remove")
                            >
                                <i class="fas fa-times"></i>
                            </button>
                            <i class=icon_class></i>
                            <span class="node-name">{display_name}</span>
                        </div>
                        }.into_any()
                    }
                })
        }}
    }
}
