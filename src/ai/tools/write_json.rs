//! Write JSON content tool

#![allow(dead_code)]

use super::{AiTool, ToolResult};
use crate::data::{validate_schema, SlickSheetData};
use crate::template::TemplateEngine;

/// Tool for writing new JSON content data
pub struct WriteJsonTool;

impl WriteJsonTool {
    /// Execute the write_json tool
    ///
    /// Validates the JSON before accepting:
    /// 1. Parse JSON syntax
    /// 2. Validate schema (required fields, data types)
    /// 3. Test compilation with current template
    pub fn execute(
        new_json: &str,
        current_template: &str,
        compile_fn: impl FnOnce(&str) -> Result<String, Vec<String>>,
    ) -> Result<SlickSheetData, ToolResult> {
        // Step 1: Parse JSON
        let data: SlickSheetData = match serde_json::from_str(new_json) {
            Ok(d) => d,
            Err(e) => {
                return Err(ToolResult::Error(format!("JSON parse error: {}", e)));
            }
        };

        // Step 2: Validate schema
        if let Err(errors) = validate_schema(&data) {
            let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            return Err(ToolResult::Error(format!(
                "Validation errors:\n{}",
                error_messages.join("\n")
            )));
        }

        // Step 3: Test compilation with current template
        let rendered = match TemplateEngine::render(current_template, &data) {
            Ok(r) => r,
            Err(errors) => {
                return Err(ToolResult::Error(format!(
                    "Template rendering failed:\n{}",
                    errors.join("\n")
                )));
            }
        };

        // Step 4: Test Typst compilation
        if let Err(errors) = compile_fn(&rendered) {
            return Err(ToolResult::Error(format!(
                "Typst compilation failed:\n{}",
                errors.join("\n")
            )));
        }

        // Success - return the validated data
        Ok(data)
    }

    /// Execute without compilation test (for simpler validation)
    pub fn execute_without_compile(new_json: &str) -> Result<SlickSheetData, ToolResult> {
        // Step 1: Parse JSON
        let data: SlickSheetData = match serde_json::from_str(new_json) {
            Ok(d) => d,
            Err(e) => {
                return Err(ToolResult::Error(format!("JSON parse error: {}", e)));
            }
        };

        // Step 2: Validate schema
        if let Err(errors) = validate_schema(&data) {
            let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            return Err(ToolResult::Error(format!(
                "Validation errors:\n{}",
                error_messages.join("\n")
            )));
        }

        Ok(data)
    }
}

impl AiTool for WriteJsonTool {
    fn name(&self) -> &'static str {
        "write_json"
    }

    fn description(&self) -> &'static str {
        r#"Update the content data with new JSON.

IMPORTANT: Always write the COMPLETE JSON object, not partial updates.
The system validates your JSON before accepting it.

Required fields:
- title: string (cannot be empty)

Optional fields:
- subtitle: string
- body: string
- sections: array of section objects
- features: array of strings
- stats: array of {value, label, color?} objects
- contact: {email?, phone?, website?, address?}
- style: {primaryColor?, accentColor?, fontFamily?}

Section object:
- heading: string (required)
- type: "text" | "list" | "table" | "quote"
- content: string (for text/quote)
- items: string[] (for list)
- rows: string[][] (for table)
- columns: number (for table)"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_json_valid() {
        let json = r#"{"title": "Test Title", "body": "Test body"}"#;
        let result = WriteJsonTool::execute_without_compile(json);

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.title, "Test Title");
        assert_eq!(data.body, "Test body");
    }

    #[test]
    fn test_write_json_invalid_syntax() {
        let json = r#"{"title": "Test", invalid}"#;
        let result = WriteJsonTool::execute_without_compile(json);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message().contains("JSON parse error"));
    }

    #[test]
    fn test_write_json_empty_title() {
        let json = r#"{"title": ""}"#;
        let result = WriteJsonTool::execute_without_compile(json);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message().contains("Title is required"));
    }

    #[test]
    fn test_write_json_invalid_color() {
        let json = r#"{"title": "Test", "style": {"primaryColor": "invalid"}}"#;
        let result = WriteJsonTool::execute_without_compile(json);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message().contains("Invalid color format"));
    }

    #[test]
    fn test_write_json_with_sections() {
        let json = r#"{
            "title": "Product",
            "sections": [
                {"heading": "Features", "type": "list", "items": ["Fast", "Reliable"]},
                {"heading": "Specs", "type": "table", "rows": [["Size", "Large"]], "columns": 2}
            ]
        }"#;
        let result = WriteJsonTool::execute_without_compile(json);

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.sections.len(), 2);
    }

    #[test]
    fn test_write_json_with_stats() {
        let json = r##"{
            "title": "Metrics",
            "stats": [
                {"value": "99%", "label": "Uptime"},
                {"value": "1M", "label": "Users", "color": "#ff0000"}
            ]
        }"##;
        let result = WriteJsonTool::execute_without_compile(json);

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.stats.len(), 2);
        assert_eq!(data.stats[1].color, Some("#ff0000".to_string()));
    }

    #[test]
    fn test_write_json_full_example() {
        let json = r##"{
            "title": "Amazing Product",
            "subtitle": "The Best Solution",
            "body": "Product description here.",
            "features": ["Fast", "Reliable", "Secure"],
            "stats": [{"value": "99%", "label": "Uptime"}],
            "contact": {"email": "sales@example.com"},
            "style": {"primaryColor": "#e94560"}
        }"##;
        let result = WriteJsonTool::execute_without_compile(json);

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.title, "Amazing Product");
        assert_eq!(data.subtitle, Some("The Best Solution".to_string()));
        assert_eq!(data.features.len(), 3);
    }
}
