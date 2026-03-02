// src/components/chat_types.rs
// Types shared between SSR and client targets.
// No browser APIs, no WASM dependencies.

use crate::components::chat_data::{ComparisonData, ContextRequest, DescriptionData};
use crate::components::tree::{NodeWithLeaf, Tree};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Llm,
    System,
    Error,
}

impl MessageRole {
    #[cfg(not(feature = "ssr"))]
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::User => "message user",
            Self::Llm => "message bot balance",
            Self::System => "message system",
            Self::Error => "message error",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    ObjectTree(Vec<Tree>),
    DocumentTree(Vec<NodeWithLeaf>),
    Description(Box<Vec<DescriptionData>>),
    Comparison(ComparisonData),
    ContextRequest(ContextRequest),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: MessageContent,
}

impl Message {
    pub fn new(role: MessageRole, content: MessageContent) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            role,
            content,
        }
    }

    pub fn new_text(role: MessageRole, text: String) -> Self {
        Self::new(role, MessageContent::Text(text))
    }
}
