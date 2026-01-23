//! Read JSON content tool

#![allow(dead_code)]

use super::{AiTool, ToolResult};
use crate::data::SlickSheetData;

/// Tool for reading the current JSON content data
pub struct ReadJsonTool;

impl ReadJsonTool {
    /// Execute the read_json tool
    pub fn execute(data: &SlickSheetData) -> ToolResult {
        match serde_json::to_string_pretty(data) {
            Ok(json) => ToolResult::Success(json),
            Err(e) => ToolResult::Error(format!("Failed to serialize JSON: {}", e)),
        }
    }
}

impl AiTool for ReadJsonTool {
    fn name(&self) -> &'static str {
        "read_json"
    }

    fn description(&self) -> &'static str {
        "Read the current content data as JSON. Returns the full JSON object with all fields including title, subtitle, body, sections, features, stats, contact, and style."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{ContactInfo, Stat};

    #[test]
    fn test_read_json_minimal() {
        let data = SlickSheetData::new("Test Title");
        let result = ReadJsonTool::execute(&data);

        assert!(result.is_success());
        let json = result.message();
        assert!(json.contains("Test Title"));
    }

    #[test]
    fn test_read_json_full() {
        let data = SlickSheetData::new("Product")
            .with_subtitle("Amazing")
            .with_body("Description")
            .with_feature("Fast")
            .with_stat(Stat::new("100%", "Uptime"))
            .with_contact(ContactInfo::with_email("test@test.com"));

        let result = ReadJsonTool::execute(&data);

        assert!(result.is_success());
        let json = result.message();
        assert!(json.contains("Product"));
        assert!(json.contains("Amazing"));
        assert!(json.contains("Fast"));
        assert!(json.contains("100%"));
        assert!(json.contains("test@test.com"));
    }

    #[test]
    fn test_read_json_is_valid_json() {
        let data = SlickSheetData::new("Test");
        let result = ReadJsonTool::execute(&data);

        let json = result.message();
        let parsed: Result<SlickSheetData, _> = serde_json::from_str(json);
        assert!(parsed.is_ok());
    }
}
