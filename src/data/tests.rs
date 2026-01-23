//! Tests for the data module

use super::schema::*;
use super::validation::*;

#[test]
fn test_slick_sheet_data_serialization() {
    let data = SlickSheetData::new("Test Title")
        .with_subtitle("Test Subtitle")
        .with_body("Test body content");

    let json = serde_json::to_string(&data).expect("Should serialize");
    let deserialized: SlickSheetData = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(data, deserialized);
}

#[test]
fn test_slick_sheet_data_full_example() {
    let data = SlickSheetData::new("Product Launch")
        .with_subtitle("Revolutionary New Product")
        .with_body("Introducing our latest innovation...")
        .with_section(Section::list(
            "Features",
            vec!["Fast".to_string(), "Reliable".to_string()],
        ))
        .with_section(Section::table(
            "Specs",
            vec![vec!["Size".to_string(), "Large".to_string()]],
            2,
        ))
        .with_stat(Stat::new("99%", "Uptime"))
        .with_contact(ContactInfo::with_email("sales@example.com"))
        .with_style(StyleHints {
            primary_color: Some("#ff0000".to_string()),
            ..Default::default()
        });

    // Serialize and deserialize
    let json = serde_json::to_string_pretty(&data).expect("Should serialize");
    let deserialized: SlickSheetData = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(data, deserialized);
    assert!(validate_schema(&deserialized).is_ok());
}

#[test]
fn test_section_types() {
    // Text section
    let text = Section::text("Heading", "Content");
    assert_eq!(text.section_type, SectionType::Text);
    assert_eq!(text.content, "Content");

    // List section
    let list = Section::list("Heading", vec!["Item 1".to_string()]);
    assert_eq!(list.section_type, SectionType::List);
    assert!(list.items.is_some());

    // Table section
    let table = Section::table("Heading", vec![vec!["Cell".to_string()]], 1);
    assert_eq!(table.section_type, SectionType::Table);
    assert!(table.rows.is_some());
    assert_eq!(table.columns, Some(1));

    // Quote section
    let quote = Section::quote("Heading", "Quote text");
    assert_eq!(quote.section_type, SectionType::Quote);
}

#[test]
fn test_stat_builder() {
    let stat = Stat::new("100%", "Success Rate").with_color("#00ff00");

    assert_eq!(stat.value, "100%");
    assert_eq!(stat.label, "Success Rate");
    assert_eq!(stat.color, Some("#00ff00".to_string()));
}

#[test]
fn test_style_hints_defaults() {
    let style = StyleHints::default();

    assert_eq!(style.primary_color_or_default(), "#e94560");
    assert_eq!(style.accent_color_or_default(), "#4ecca3");
    assert_eq!(style.font_family_or_default(), "Inter");
}

#[test]
fn test_style_hints_custom() {
    let style = StyleHints {
        primary_color: Some("#ff0000".to_string()),
        accent_color: Some("#00ff00".to_string()),
        font_family: Some("Roboto".to_string()),
    };

    assert_eq!(style.primary_color_or_default(), "#ff0000");
    assert_eq!(style.accent_color_or_default(), "#00ff00");
    assert_eq!(style.font_family_or_default(), "Roboto");
}

#[test]
fn test_json_with_optional_fields_omitted() {
    // Create minimal data
    let data = SlickSheetData::new("Title Only");

    // Serialize
    let json = serde_json::to_string(&data).expect("Should serialize");

    // Check that optional None fields are not in the JSON
    assert!(!json.contains("subtitle"));
    assert!(!json.contains("contact"));
    assert!(!json.contains("style"));
}

#[test]
fn test_json_deserialization_with_missing_optional_fields() {
    let json = r#"{"title": "Just Title"}"#;

    let data: SlickSheetData = serde_json::from_str(json).expect("Should deserialize");

    assert_eq!(data.title, "Just Title");
    assert!(data.subtitle.is_none());
    assert!(data.body.is_empty());
    assert!(data.sections.is_empty());
    assert!(data.features.is_empty());
    assert!(data.stats.is_empty());
    assert!(data.contact.is_none());
    assert!(data.style.is_none());
}

#[test]
fn test_section_type_serialization() {
    let list = Section::list("Test", vec!["Item".to_string()]);
    let json = serde_json::to_string(&list).expect("Should serialize");

    // Check that type is serialized as lowercase
    assert!(json.contains(r#""type":"list""#));
}

#[test]
fn test_validation_with_valid_data() {
    let data = SlickSheetData::new("Valid Title")
        .with_section(Section::list(
            "Features",
            vec!["Feature 1".to_string(), "Feature 2".to_string()],
        ))
        .with_section(Section::table("Specs", vec![vec!["A".to_string()]], 1))
        .with_stat(Stat::new("50%", "Improvement"));

    assert!(validate_schema(&data).is_ok());
}

#[test]
fn test_validation_multiple_errors() {
    let data = SlickSheetData {
        title: "".to_string(), // Error: empty title
        sections: vec![
            Section {
                heading: "".to_string(), // Error: empty heading
                section_type: SectionType::List,
                items: Some(vec![]), // Error: empty list
                ..Default::default()
            },
            Section {
                heading: "Table".to_string(),
                section_type: SectionType::Table,
                rows: Some(vec![]), // Error: empty table
                columns: None,      // Error: missing columns
                ..Default::default()
            },
        ],
        stats: vec![Stat {
            value: "".to_string(),              // Error: empty value
            label: "".to_string(),              // Error: empty label
            color: Some("invalid".to_string()), // Error: invalid color
        }],
        ..Default::default()
    };

    let result = validate_schema(&data);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    // Should have multiple errors
    assert!(errors.len() >= 5);
}
