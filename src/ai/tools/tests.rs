//! Integration tests for AI tools

use super::*;
use crate::data::{ContactInfo, SlickSheetData, Stat, StyleHints};

#[test]
fn test_round_trip_json() {
    let original = SlickSheetData::new("Test Product")
        .with_subtitle("Amazing Product")
        .with_body("Product description")
        .with_feature("Fast")
        .with_feature("Reliable")
        .with_stat(Stat::new("99%", "Uptime"))
        .with_contact(ContactInfo::with_email("test@example.com"))
        .with_style(StyleHints {
            primary_color: Some("#e94560".to_string()),
            ..Default::default()
        });

    // Read to JSON
    let result = ReadJsonTool::execute(&original);
    assert!(result.is_success());
    let json = result.message();

    // Write back from JSON
    let write_result = WriteJsonTool::execute_without_compile(json);
    assert!(write_result.is_ok());

    let restored = write_result.unwrap();
    assert_eq!(restored, original);
}

#[test]
fn test_template_with_data() {
    let template = r#"# {{title}}
{{#if subtitle}}## {{subtitle}}{{/if}}
{{body}}
{{#each features}}- {{this}}
{{/each}}"#;

    let data = SlickSheetData::new("Product")
        .with_subtitle("Tagline")
        .with_body("Description")
        .with_feature("A")
        .with_feature("B");

    // Read template
    let read_result = ReadTemplateTool::execute(template);
    assert!(read_result.is_success());

    // Write template (validation only)
    let write_result = WriteTemplateTool::execute_without_compile(template, &data);
    assert!(write_result.is_ok());
}

#[test]
fn test_tool_names() {
    let read_json = ReadJsonTool;
    let write_json = WriteJsonTool;
    let read_template = ReadTemplateTool;
    let write_template = WriteTemplateTool;

    assert_eq!(read_json.name(), "read_json");
    assert_eq!(write_json.name(), "write_json");
    assert_eq!(read_template.name(), "read_template");
    assert_eq!(write_template.name(), "write_template");
}

#[test]
fn test_tool_descriptions() {
    let read_json = ReadJsonTool;
    let write_json = WriteJsonTool;
    let read_template = ReadTemplateTool;
    let write_template = WriteTemplateTool;

    // All tools should have non-empty descriptions
    assert!(!read_json.description().is_empty());
    assert!(!write_json.description().is_empty());
    assert!(!read_template.description().is_empty());
    assert!(!write_template.description().is_empty());

    // write_json description should mention required fields
    assert!(write_json.description().contains("title"));

    // write_template description should mention placeholders
    assert!(write_template.description().contains("{{"));
}

#[test]
fn test_tool_result() {
    let success = ToolResult::Success("OK".to_string());
    let error = ToolResult::Error("Failed".to_string());

    assert!(success.is_success());
    assert!(!error.is_success());

    assert_eq!(success.message(), "OK");
    assert_eq!(error.message(), "Failed");
}

#[test]
fn test_write_json_validation_errors() {
    // Empty title
    let result = WriteJsonTool::execute_without_compile(r#"{"title": ""}"#);
    assert!(result.is_err());

    // Invalid JSON
    let result = WriteJsonTool::execute_without_compile(r#"not json"#);
    assert!(result.is_err());

    // Invalid color
    let result = WriteJsonTool::execute_without_compile(
        r#"{"title": "Test", "style": {"primaryColor": "bad"}}"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_write_template_validation_errors() {
    let data = SlickSheetData::new("Test");

    // Unclosed tag
    let result = WriteTemplateTool::execute_without_compile("{{#if x}}open", &data);
    assert!(result.is_err());

    // Empty template
    let result = WriteTemplateTool::execute_without_compile("", &data);
    assert!(result.is_err());
}
