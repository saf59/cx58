use crate::components::tree::{NodeInfo, NodeWithLeaf};
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
        let id = node_info.id;
        if let Some(parent) = self.parent.get()
            && parent.id == id
        {
            self.parent.set(None);
            self.prev_leaf.set(None);
            self.next_leaf.set(None);
        }
        if let Some(next) = self.next_leaf.get()  && next.id == id {
            self.next_leaf.set(None);
        } else if let Some(prev) = self.prev_leaf.get() && prev.id == id {
            let new_prev = self.next_leaf.read().clone();
            self.next_leaf.set(None);
            self.prev_leaf.set(new_prev);
        }
    }

    pub fn set_parent(&self, node_info: NodeInfo) {
        if let Some(parent) = &self.parent.get()
            && parent.id == node_info.id {
                return;
            }
        self.parent.set(Some(node_info));
        self.prev_leaf.set(None);
        self.next_leaf.set(None);
    }

    pub fn set_leaf(&self, node_info: &NodeWithLeaf, parent_node: &NodeWithLeaf) {
        if let Some(parent) = &self.parent.get()
            && parent.id != parent_node.id
        {
            self.parent.set(Some(parent_node.clone().into()));
        }
        if self.parent.get().is_none() {
            self.parent.set(Some(parent_node.clone().into()));
        }
        self.set_one_leaf(node_info.clone().into())
    }
    #[allow(dead_code)]
    pub fn set_one_leaf(&self, new_node: NodeInfo) {
        if self.prev_leaf.get().is_none() {
            self.prev_leaf.set(Some(new_node));
        } else {
            let prev_cloned = self.prev_leaf.read().clone().expect("prev leaf");
            match prev_cloned.date_time.cmp(&new_node.date_time) {
                std::cmp::Ordering::Less => {
                    // prev < new (next is None)
                    self.prev_leaf.set(Some(new_node.clone()));
                    self.next_leaf.set(Some(prev_cloned));
                }
                std::cmp::Ordering::Equal => {} // do nothing
                std::cmp::Ordering::Greater => {
                    // prev > next
                    self.next_leaf.set(Some(new_node.clone()));
                }
            }
        }
    }
}
