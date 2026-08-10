use crate::components::tree::NodeInfo;
use crate::components::tree::NodeWithLeaf;
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct ChatContext {
    pub clear_history: RwSignal<bool>,
    pub insert_text: RwSignal<Option<String>>,
    pub insert_and_enter: RwSignal<Option<String>>,
    pub parent: RwSignal<Option<NodeInfo>>,
    pub prev_leaf: RwSignal<Option<NodeInfo>>,
    pub next_leaf: RwSignal<Option<NodeInfo>>,
    report_context_submitted: RwSignal<bool>,
}

impl ChatContext {
    pub fn new() -> Self {
        Self {
            clear_history: RwSignal::new(false),
            insert_text: RwSignal::new(None),
            insert_and_enter: RwSignal::new(None),
            parent: RwSignal::new(None),
            prev_leaf: RwSignal::new(None),
            next_leaf: RwSignal::new(None),
            report_context_submitted: RwSignal::new(false),
        }
    }
    pub fn clear(&self) {
        self.parent.set(None);
        self.prev_leaf.set(None);
        self.next_leaf.set(None);
        self.report_context_submitted.set(false);
    }
    pub fn delete_node_info(&self, node_info: NodeInfo) {
        let id = node_info.id;
        if let Some(parent) = self.parent.get()
            && parent.id == id
        {
            self.parent.set(None);
            self.prev_leaf.set(None);
            self.next_leaf.set(None);
            self.report_context_submitted.set(false);
        }
        if let Some(next) = self.next_leaf.get()
            && next.id == id
        {
            self.next_leaf.set(None);
            self.report_context_submitted.set(false);
        } else if let Some(prev) = self.prev_leaf.get()
            && prev.id == id
        {
            let new_prev = self.next_leaf.read().clone();
            self.next_leaf.set(None);
            self.prev_leaf.set(new_prev);
            self.report_context_submitted.set(false);
        }
    }

    pub fn set_parent(&self, node_info: NodeInfo) {
        if let Some(parent) = &self.parent.get()
            && parent.id == node_info.id
        {
            return;
        }
        self.parent.set(Some(node_info));
        self.prev_leaf.set(None);
        self.next_leaf.set(None);
        self.report_context_submitted.set(false);
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

    pub fn set_one_leaf(&self, new_node: NodeInfo) {
        if self.report_context_submitted.get_untracked() {
            self.prev_leaf.set(None);
            self.next_leaf.set(None);
            self.report_context_submitted.set(false);
        }

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

    #[cfg(any(not(feature = "ssr"), test))]
    pub fn mark_report_context_submitted(&self) {
        if self.prev_leaf.get_untracked().is_some() || self.next_leaf.get_untracked().is_some() {
            self.report_context_submitted.set(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::tree::NodeType;
    use uuid::Uuid;

    fn report(name: &str, date_time: i64) -> NodeInfo {
        NodeInfo {
            id: Uuid::now_v7(),
            parent_id: None,
            name: Some(name.to_string()),
            node_type: NodeType::ImageLeaf,
            date_time,
        }
    }

    #[test]
    fn two_clicks_before_request_build_comparison_pair() {
        Owner::new().with(|| {
            let context = ChatContext::new();
            let older = report("22.05.2026 19:30:00", 1);
            let newer = report("22.05.2026 20:00:00", 2);

            context.set_one_leaf(older.clone());
            context.set_one_leaf(newer.clone());

            assert_eq!(context.prev_leaf.get_untracked().unwrap().id, newer.id);
            assert_eq!(context.next_leaf.get_untracked().unwrap().id, older.id);
        });
    }

    #[test]
    fn first_click_after_request_replaces_submitted_report_context() {
        Owner::new().with(|| {
            let context = ChatContext::new();
            let old_report = report("22.05.2026 19:30:00", 1);
            let old_current_report = report("30.08.2026 17:00:00", 3);
            let new_report = report("22.05.2026 20:00:00", 2);

            context.set_one_leaf(old_report);
            context.set_one_leaf(old_current_report);
            context.mark_report_context_submitted();
            context.set_one_leaf(new_report.clone());

            assert_eq!(context.prev_leaf.get_untracked().unwrap().id, new_report.id);
            assert!(context.next_leaf.get_untracked().is_none());
        });
    }
}
