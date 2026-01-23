//! Tests for the editor module

use super::content::Content;
use super::links::{parse_cmd_url, EditCommand};
use std::collections::HashMap;

// ============================================================================
// Content Model Tests
// ============================================================================

#[test]
fn test_content_new_creates_default() {
    let content = Content::new();
    assert_eq!(content.title, "Hello World");
    assert!(content.subtitle.is_none());
    assert!(!content.body.is_empty());
    assert!(content.image_url.is_none());
    assert!(content.metadata.is_empty());
}

#[test]
fn test_content_serializes_to_json() {
    let content = Content {
        title: "Test Title".to_string(),
        subtitle: Some("Test Subtitle".to_string()),
        body: "Test body text".to_string(),
        image_url: Some("https://example.com/image.png".to_string()),
        metadata: HashMap::from([("key".to_string(), "value".to_string())]),
    };

    let json = serde_json::to_string(&content).expect("should serialize");
    assert!(json.contains("Test Title"));
    assert!(json.contains("Test Subtitle"));
    assert!(json.contains("Test body text"));
}

#[test]
fn test_content_deserializes_from_json() {
    let json = r#"{
        "title": "Deserialized Title",
        "subtitle": "Deserialized Subtitle",
        "body": "Deserialized body",
        "image_url": null,
        "metadata": {}
    }"#;

    let content: Content = serde_json::from_str(json).expect("should deserialize");
    assert_eq!(content.title, "Deserialized Title");
    assert_eq!(content.subtitle, Some("Deserialized Subtitle".to_string()));
    assert_eq!(content.body, "Deserialized body");
    assert!(content.image_url.is_none());
}

#[test]
fn test_content_roundtrip() {
    let original = Content {
        title: "Round Trip Test".to_string(),
        subtitle: Some("With Subtitle".to_string()),
        body: "Body content here".to_string(),
        image_url: Some("https://example.com/img.jpg".to_string()),
        metadata: HashMap::from([
            ("author".to_string(), "Test Author".to_string()),
            ("date".to_string(), "2024-01-15".to_string()),
        ]),
    };

    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: Content = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(original, deserialized);
}

#[test]
fn test_content_to_typst_generates_valid_markup() {
    let content = Content::new();
    let typst = content.to_typst();

    assert!(typst.contains("#set page"));
    assert!(typst.contains("#set text"));
    assert!(typst.contains("cmd://edit/title"));
    assert!(typst.contains("cmd://edit/body"));
}

#[test]
fn test_content_to_typst_includes_subtitle() {
    let content = Content {
        subtitle: Some("Test Subtitle".to_string()),
        ..Default::default()
    };
    let typst = content.to_typst();

    assert!(typst.contains("cmd://edit/subtitle"));
}

#[test]
fn test_content_to_typst_includes_metadata() {
    let content = Content {
        metadata: HashMap::from([("author".to_string(), "John Doe".to_string())]),
        ..Default::default()
    };
    let typst = content.to_typst();

    assert!(typst.contains("cmd://edit/meta/author"));
    assert!(typst.contains("John Doe"));
}

// ============================================================================
// Link Parsing Tests
// ============================================================================

#[test]
fn test_parse_cmd_url_title() {
    let result = parse_cmd_url("cmd://edit/title");
    assert_eq!(result, Some(EditCommand::Title));
}

#[test]
fn test_parse_cmd_url_subtitle() {
    let result = parse_cmd_url("cmd://edit/subtitle");
    assert_eq!(result, Some(EditCommand::Subtitle));
}

#[test]
fn test_parse_cmd_url_body() {
    let result = parse_cmd_url("cmd://edit/body");
    assert_eq!(result, Some(EditCommand::Body));
}

#[test]
fn test_parse_cmd_url_image() {
    let result = parse_cmd_url("cmd://edit/image");
    assert_eq!(result, Some(EditCommand::Image));
}

#[test]
fn test_parse_cmd_url_metadata() {
    let result = parse_cmd_url("cmd://edit/meta/author");
    assert_eq!(result, Some(EditCommand::Metadata("author".to_string())));
}

#[test]
fn test_parse_cmd_url_https_returns_none() {
    let result = parse_cmd_url("https://example.com");
    assert_eq!(result, None);
}

#[test]
fn test_parse_cmd_url_invalid_cmd_returns_none() {
    let result = parse_cmd_url("cmd://invalid/path");
    assert_eq!(result, None);
}

#[test]
fn test_parse_cmd_url_empty_returns_none() {
    let result = parse_cmd_url("");
    assert_eq!(result, None);
}

// ============================================================================
// EditCommand Tests
// ============================================================================

#[test]
fn test_edit_command_covers_all_fields() {
    // This test ensures the EditCommand enum has all expected variants
    let commands = vec![
        EditCommand::Title,
        EditCommand::Subtitle,
        EditCommand::Body,
        EditCommand::Image,
        EditCommand::Metadata("test".to_string()),
    ];

    assert_eq!(commands.len(), 5);
}

#[test]
fn test_edit_command_clone() {
    let cmd = EditCommand::Metadata("key".to_string());
    let cloned = cmd.clone();
    assert_eq!(cmd, cloned);
}

#[test]
fn test_edit_command_debug() {
    let cmd = EditCommand::Title;
    let debug = format!("{:?}", cmd);
    assert!(debug.contains("Title"));
}
