use crate::components::chat_context::ChatContext;
use crate::components::chat_types::{Message, MessageContent};
use crate::components::show_carusel::CarouselRenderer;
use crate::components::show_comparison::ComparisonRenderer;
use crate::components::show_context_request::ContextRequestRenderer;
use crate::components::show_description::DescriptionListRenderer;
use crate::components::show_tree::DetailsTreeRendererWithContext;
use leptos::context::use_context;
use leptos::html::InnerHtmlAttribute;
use leptos::prelude::{ClassAttribute, ElementChild, IntoAny};
use leptos::IntoView;
use leptos_macro::{component, view};

#[component]
pub fn MessageRenderer(message: Message) -> impl IntoView {
    let css_class = message.role.css_class();
    let ctx = use_context::<ChatContext>().expect("Context lost");

    match message.content {
        MessageContent::Text(text) => view! { <div class=css_class inner_html=text /> }.into_any(),
        MessageContent::ObjectTree(tree) => view! {
            <div class=css_class>
                <DetailsTreeRendererWithContext
                    tree=tree
                    on_node_click=move |node_info| {
                        tracing::info!("Node clicked: {:?}", node_info.name);
                        ctx.set_parent(node_info.clone())
                    }
                />
            </div>
        }
        .into_any(),
        MessageContent::DocumentTree(data) => view! {
            <div class=css_class>
                <CarouselRenderer data=data />
            </div>
        }
        .into_any(),
        MessageContent::Description(data) => view! {
            <div class=css_class>
                <DescriptionListRenderer data=*data />
            </div>
        }
        .into_any(),
        MessageContent::Comparison(data) => view! {
            <div class=css_class>
                <ComparisonRenderer data=data />
            </div>
        }
        .into_any(),
        MessageContent::ContextRequest(data) => view! {
            <div class=css_class>
                <ContextRequestRenderer data=data.clone() />
            </div>
        }
        .into_any(),
    }
}
