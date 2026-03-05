pub(crate) mod chat_context;
pub(crate) mod home_page;
pub(crate) mod lang;
pub(crate) mod media_proxy_script;
pub(crate) mod node_info_display;
pub(crate) mod show_carusel;
pub(crate) mod show_description;
pub(crate) mod show_tree;
pub(crate) mod side_body;
pub(crate) mod side_top;
pub(crate) mod sidebar;
pub(crate) mod tree;
pub(crate) mod user_info;

pub(crate) mod chat;
pub(crate) mod chat_client;
pub(crate) mod chat_data;
pub(crate) mod chat_types;
pub(crate) mod message_renderer;
pub(crate) mod show_comparison;
pub(crate) mod show_context_request;

// Builds a FluentValue args map from key => value pairs.
// Usage: args!["error" => some_string, "status" => code]
//#[cfg(not(feature = "ssr"))]
macro_rules! args {
    ( $( $k:literal => $v:expr ),* $(,)? ) => {{
        use std::collections::HashMap;
        use std::borrow::Cow;
        use fluent_templates::fluent_bundle::FluentValue;
        HashMap::from([
            $( (Cow::Borrowed($k), FluentValue::from($v)), )*
        ])
    }};
}
//#[cfg(not(feature = "ssr"))]
pub(crate) use args;
