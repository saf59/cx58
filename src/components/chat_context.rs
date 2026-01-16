use leptos::prelude::*;
use crate::components::tree::{NodeInfo};

#[derive(Clone, Copy)]
pub struct ChatContext {
    pub clear_history: RwSignal<bool>,
    pub insert_text: RwSignal<Option<String>>,
    pub parent:RwSignal<Option<NodeInfo>>,
    pub prev_leaf:RwSignal<Option<NodeInfo>>,
    pub next_leaf:RwSignal<Option<NodeInfo>>,
}

impl ChatContext {
    pub fn new() -> Self {
        Self {
            clear_history: RwSignal::new(false),
            insert_text: RwSignal::new(None),
            parent: RwSignal::new(None),
            prev_leaf: RwSignal::new(None),
            next_leaf: RwSignal::new(None),
        }
    }

}
