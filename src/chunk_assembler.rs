use serde_json::Value;

#[derive(Debug, Clone)]
pub enum UiChunk {
    Text(String),
    Markdown(String),
    Json(Value),
}

pub struct ChunkAssembler {
    response_buffer: String,
    text_buffer: String,
}

impl Default for ChunkAssembler {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkAssembler {
    pub fn new() -> Self {
        Self {
            response_buffer: String::new(),
            text_buffer: String::new(),
        }
    }

    /// Extracts text from the buffer
    fn extract_text_from_buffer(&self) -> Option<String> {
        // Find the start of "text":"
        let text_start_pattern = r#""text""#;
        let text_start = self.response_buffer.find(text_start_pattern)?;

        // Find the start of the value (after :)
        let after_key = &self.response_buffer[text_start + text_start_pattern.len()..];
        let colon_pos = after_key.find(':')?;
        let after_colon = &after_key[colon_pos + 1..].trim_start();

        // Check if the value starts with a quote
        if !after_colon.starts_with('"') {
            return None;
        }

        // Extract text until the closing quote (or until the end if there is no closing quote)
        let mut result = String::new();
        let chars = after_colon[1..].chars();
        let mut escaped = false;

        //while let Some(ch) = chars.next() {
        for ch in chars {
            if escaped {
                // Match escape sequences
                match ch {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    _ => {
                        result.push('\\');
                        result.push(ch);
                    }
                }
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                // End of text
                break;
            } else {
                result.push(ch);
            }
        }

        Some(result)
    }

    ///   Add JSON string from SSE and return UiChunk if text is ready
    pub fn push_sse_line(&mut self, line: &str) -> Vec<UiChunk> {
        let mut output = Vec::new();
        let line = line.trim();
        if line.is_empty() {
            return output;
        }

        // Parse JSON string from SSE line
        match serde_json::from_str::<Value>(line) {
            Ok(val) => {
                // Collect response for final JSON
                if let Some(resp) = val.get("response").and_then(|v| v.as_str()) {
                    self.response_buffer.push_str(resp);

                    // Extract text from buffer
                    if let Some(current_text) = self.extract_text_from_buffer() {
                        // Send chunk if new text is available
                        if current_text.len() > self.text_buffer.len() {
                            let new_part = &current_text[self.text_buffer.len()..];
                            if !new_part.is_empty() {
                                //tracing::info!("Streaming text chunk: {:?}", new_part);
                                output.push(UiChunk::Text(new_part.to_string()));
                            }
                            self.text_buffer = current_text;
                        }
                    }
                }

                // Send final chunk if done=true
                if val.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                    //tracing::info!("Done=true, final buffer length: {}",self.response_buffer.len());
                    if !self.response_buffer.is_empty() {
                        match serde_json::from_str::<Value>(&self.response_buffer) {
                            Ok(final_json) => {
                                //tracing::info!("Pushing final JSON");
                                output.push(UiChunk::Json(final_json));
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to parse accumulated response as JSON: {:?}",
                                    e
                                );
                            }
                        }
                    }
                    self.response_buffer.clear();
                    self.text_buffer.clear();
                }
            }
            Err(e) => {
                tracing::warn!("Failed to parse SSE line: {:?}", e);
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_text_from_incomplete_json() {
        let mut a = ChunkAssembler::new();
        a.response_buffer = r#"{ "text": "Hello"#.to_string();
        assert_eq!(a.extract_text_from_buffer(), Some("Hello".to_string()));
    }

    #[test]
    fn streams_text_incrementally() {
        let mut a = ChunkAssembler::new();

        let chunks1 = a.push_sse_line(r#"{"response":"{ \"text\": \"H","done":false}"#);
        assert_eq!(chunks1.len(), 1);
        assert!(matches!(&chunks1[0], UiChunk::Text(t) if t == "H"));

        let chunks2 = a.push_sse_line(r#"{"response":"ello","done":false}"#);
        assert_eq!(chunks2.len(), 1);
        assert!(matches!(&chunks2[0], UiChunk::Text(t) if t == "ello"));
    }

    #[test]
    fn handles_escape_sequences() {
        let mut a = ChunkAssembler::new();
        a.response_buffer = r#"{ "text": "line1\nline2"#.to_string();
        let text = a.extract_text_from_buffer().unwrap();
        assert_eq!(text, "line1\nline2");
    }
}
