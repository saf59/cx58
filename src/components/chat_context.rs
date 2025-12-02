use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct ChatContext {
    pub clear_history: RwSignal<bool>,
    pub insert_text: RwSignal<Option<String>>,
    pub locale: RwSignal<String>,
}

impl ChatContext {
    pub fn new() -> Self {
        Self {
            clear_history: RwSignal::new(false),
            insert_text: RwSignal::new(None),
            locale: RwSignal::new("en".to_string()),
        }
    }
}
