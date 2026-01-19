use crate::components::tree::NodeInfo;
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct ChatContext {
    pub clear_history: RwSignal<bool>,
    pub insert_text: RwSignal<Option<String>>,
    pub parent: RwSignal<Option<NodeInfo>>,
    pub prev_leaf: RwSignal<Option<NodeInfo>>,
    pub next_leaf: RwSignal<Option<NodeInfo>>,
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
    pub fn delete_node_info(&self, node_info: NodeInfo) {
        if let Some(parent) = self.parent.get()
            && parent.id == node_info.id {
                self.parent.set(None);
                self.prev_leaf.set(None);
                self.next_leaf.set(None);
        }
        if let Some(_next) = self.next_leaf.get() {
            self.next_leaf.set(None);
        }
        if let Some(_prev) = self.prev_leaf.get() {
            self.prev_leaf.set(self.next_leaf.read().clone());
            self.next_leaf.set(None);
        }
    }
    pub fn set_parent(&self, node_info: NodeInfo) {
        self.parent.set(Some(node_info));
    }

    #[allow(dead_code)]
    pub fn set_leaf(&self, node_info: NodeInfo) {
        if self.prev_leaf.get().is_none() {
            self.prev_leaf.set(Some(node_info));
        } else if self.next_leaf.get().is_none() {
            let new_cloned = node_info.clone();
            if let (Some(prev_node), new_node) = (self.prev_leaf.read().as_ref(), &node_info) {
                let prev_name = prev_node.name.as_deref().unwrap_or("");
                let new_name = new_node.name.as_deref().unwrap_or("");
                match prev_name.cmp(new_name) {
                    std::cmp::Ordering::Less => {
                        // prev < new
                        self.next_leaf.set(Some(new_cloned));
                    }
                    std::cmp::Ordering::Equal => {} // do nothing
                    std::cmp::Ordering::Greater => {
                        // prev > next
                        self.next_leaf
                            .set(Some(self.prev_leaf.read().clone().expect("prev leaf")));
                        self.prev_leaf.set(Some(new_cloned));
                    }
                }
            } else if let (Some(prev_node), new_node) = (self.prev_leaf.read().as_ref(), &node_info)
            {
                let prev_name = prev_node.name.as_deref().unwrap_or("");
                let new_name = new_node.name.as_deref().unwrap_or("");
                match prev_name.cmp(new_name) {
                    std::cmp::Ordering::Greater => {
                        // prev < new
                        self.prev_leaf.set(Some(new_cloned));
                    }
                    _ => {
                        self.next_leaf.set(Some(new_cloned));
                    }
                }
            }
        }
    }
}
