// src/components/chat_types.rs
// Types shared between SSR and client targets.
// No browser APIs, no WASM dependencies.

use crate::components::chat_data::{ComparisonData, ContextRequest, DescriptionData};
use crate::components::tree::{NodeWithLeaf, Tree};
use crate::report_export::ReportRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Llm,
    System,
    Error,
}

impl MessageRole {
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
    pub report_request: Option<ReportRequest>,
}

impl Message {
    pub fn new(role: MessageRole, content: MessageContent) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            role,
            content,
            report_request: None,
        }
    }

    pub fn new_text(role: MessageRole, text: String) -> Self {
        Self::new(role, MessageContent::Text(text))
    }

    #[cfg(not(feature = "ssr"))]
    pub fn from_report(content: MessageContent, request: ReportRequest) -> Self {
        let mut message = Self::new(MessageRole::Llm, content);
        message.report_request = Some(request);
        message
    }
}
