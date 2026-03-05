use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextRequest {
    pub prompt: String,
    pub suggestions: Vec<String>,
}

/// Client-side structure matching server's DescriptionData
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DescriptionData {
    pub object: String,
    #[serde(skip)]
    pub object_id: String, // We skip serialization but keep for internal use
    pub date: String,
    #[serde(skip)]
    pub date_id: String, // We skip serialization but keep for internal use
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doors: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radiators: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openings: Option<String>,
    pub model_name: String,
    pub confidence: Option<f32>,
    pub created_at: String,
}

impl DescriptionData {
    /// Renders the description data as a Markdown string
    pub fn to_markdown(&self) -> String {
        let (object_name, report_name) = extract_name_pair(&self.object);
        let mut md = format!(
            "# {}\n\n**Report:** {}\n\n{}\n",
            object_name, report_name, self.description
        );
        if let Some(v) = &self.windows {
            md.push_str(&format!("\n**Windows:** {}\n", v));
        }
        if let Some(v) = &self.doors {
            md.push_str(&format!("\n**Doors:** {}\n", v));
        }
        if let Some(v) = &self.radiators {
            md.push_str(&format!("\n**Radiators:** {}\n", v));
        }
        if let Some(v) = &self.openings {
            md.push_str(&format!("\n**Openings:** {}\n", v));
        }
        md
    }

    /// Builds the filename from the header text (spaces replaced with underscores)
    pub fn filename(&self) -> String {
        let (object_name, report_name) = extract_name_pair(&self.object);
        let object_name = object_name.replace(" - ", "_");
        let header = format!("Report_{}_{}", object_name, report_name);
        let sanitized = header.replace(' ', "_");
        format!("{}.md", sanitized)
    }
}

/// Client-side structure matching server's DescriptionData
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparisonData {
    pub object_name: String,
    pub prev_date: String,
    pub next_date: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doors: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radiators: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openings: Option<String>,
}

impl ComparisonData {
    /// Renders the comparison data as a Markdown string
    pub fn to_markdown(&self) -> String {
        let mut md = format!(
            "# {}\n\n**Changes from** {} **to** {}\n\n{}\n",
            self.object_name, self.prev_date, self.next_date, self.description
        );
        if let Some(v) = &self.windows {
            md.push_str(&format!("\n**Windows:** {}\n", v));
        }
        if let Some(v) = &self.doors {
            md.push_str(&format!("\n**Doors:** {}\n", v));
        }
        if let Some(v) = &self.radiators {
            md.push_str(&format!("\n**Radiators:** {}\n", v));
        }
        if let Some(v) = &self.openings {
            md.push_str(&format!("\n**Openings:** {}\n", v));
        }
        md
    }

    /// Builds the filename from the header text (spaces replaced with underscores)
    pub fn filename(&self) -> String {
        let object_name = self.object_name.replace(" - ", "_");
        let header = format!(
            "Changes_{}_from_{}_to_{}",
            &object_name, self.prev_date, self.next_date
        );
        let sanitized = header.replace(' ', "_");
        format!("{}.md", sanitized)
    }
}

pub fn extract_name_pair(full_name: &str) -> (String, String) {
    let full_name = full_name.replace("Root/", "");
    let parts: Vec<&str> = full_name.split('/').collect();

    let report_name = parts.last().unwrap_or(&"").to_string();
    let object_name = parts[..parts.len().saturating_sub(1)].join(" - ");

    (object_name, report_name)
}
