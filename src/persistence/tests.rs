//! Tests for the persistence module

use super::export::pdf_bytes_from_source;
use super::project::{Project, ProjectMetadata};

// ============================================================================
// Project Serialization Tests
// ============================================================================

#[test]
fn test_project_new_has_defaults() {
    let project = Project::new();

    assert!(!project.metadata.name.is_empty());
    assert!(!project.source.is_empty());
    assert!(project.metadata.created_at.is_some());
}

#[test]
fn test_project_with_name() {
    let project = Project::with_name("My Project".to_string());

    assert_eq!(project.metadata.name, "My Project");
}

#[test]
fn test_project_serializes_to_json() {
    let project = Project::with_name("Test Project".to_string());

    let json = serde_json::to_string(&project).expect("should serialize");

    assert!(json.contains("Test Project"));
    assert!(json.contains("source"));
    assert!(json.contains("metadata"));
}

#[test]
fn test_project_deserializes_from_json() {
    let json = r#"{
        "metadata": {
            "name": "Deserialized Project",
            "description": "A test project",
            "version": "1.0.0",
            "created_at": "2024-01-15T12:00:00Z",
            "modified_at": null
        },
        "source": "= Hello World"
    }"#;

    let project: Project = serde_json::from_str(json).expect("should deserialize");

    assert_eq!(project.metadata.name, "Deserialized Project");
    assert_eq!(project.source, "= Hello World");
}

#[test]
fn test_project_roundtrip() {
    let original = Project {
        metadata: ProjectMetadata {
            name: "Round Trip Test".to_string(),
            description: Some("Test description".to_string()),
            version: "1.0.0".to_string(),
            created_at: Some("2024-01-15T12:00:00Z".to_string()),
            modified_at: Some("2024-01-15T13:00:00Z".to_string()),
        },
        source: "= Test\n\nContent here".to_string(),
    };

    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: Project = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(original.metadata.name, deserialized.metadata.name);
    assert_eq!(original.source, deserialized.source);
}

#[test]
fn test_project_to_json_pretty() {
    let project = Project::with_name("Pretty JSON Test".to_string());

    let json = project.to_json_pretty();

    assert!(json.is_ok());
    let json_str = json.unwrap();
    assert!(json_str.contains('\n')); // Pretty formatted has newlines
    assert!(json_str.contains("Pretty JSON Test"));
}

#[test]
fn test_project_from_json() {
    let json = r#"{"metadata":{"name":"From JSON","description":null,"version":"1.0.0","created_at":null,"modified_at":null},"source":"= Title"}"#;

    let project = Project::from_json(json);

    assert!(project.is_ok());
    assert_eq!(project.unwrap().metadata.name, "From JSON");
}

#[test]
fn test_project_from_invalid_json() {
    let json = "not valid json";

    let result = Project::from_json(json);

    assert!(result.is_err());
}

// ============================================================================
// Project Metadata Tests
// ============================================================================

#[test]
fn test_metadata_default() {
    let metadata = ProjectMetadata::default();

    assert_eq!(metadata.name, "Untitled Project");
    assert_eq!(metadata.version, "1.0.0");
    assert!(metadata.description.is_none());
}

#[test]
fn test_metadata_custom_name() {
    let metadata = ProjectMetadata {
        name: "Custom Name".to_string(),
        ..Default::default()
    };

    assert_eq!(metadata.name, "Custom Name");
}

// ============================================================================
// PDF Export Tests
// ============================================================================

#[test]
fn test_pdf_export_from_source() {
    let source = r#"#set page(width: 8.5in, height: 11in, margin: 0.75in)
= Hello World

This is a test document."#;

    let result = pdf_bytes_from_source(source);

    assert!(result.is_ok(), "PDF export failed: {:?}", result.err());

    let pdf_bytes = result.unwrap();
    assert!(!pdf_bytes.is_empty());

    // PDF files start with %PDF-
    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "Output doesn't look like a PDF"
    );
}

#[test]
fn test_pdf_export_with_invalid_source() {
    let source = "= Title\n\n#invalid_function()";

    let result = pdf_bytes_from_source(source);

    // Should return an error for invalid Typst
    assert!(result.is_err());
}

#[test]
fn test_pdf_export_with_complex_document() {
    let source = r##"#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "Inter", size: 11pt)

= Product Sheet

== Overview
Lorem ipsum dolor sit amet.

#table(
  columns: 2,
  [*Feature*], [*Value*],
  [Item 1], [Value 1],
  [Item 2], [Value 2],
)
"##;

    let result = pdf_bytes_from_source(source);

    assert!(result.is_ok());
    let pdf_bytes = result.unwrap();
    assert!(pdf_bytes.len() > 1000); // Complex doc should produce larger PDF
}
