//! Integration tests for the template module

use super::engine::TemplateEngine;
use super::parser::parse_template;
use super::validation::validate_template;
use crate::data::{ContactInfo, Section, SlickSheetData, Stat, StyleHints};

#[test]
fn test_full_product_sheet_template() {
    let template = r#"#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "{{style.fontFamily | default: 'Inter'}}", size: 11pt)

#align(center)[
  #text(size: 24pt, weight: "bold", fill: rgb("{{style.primaryColor | default: '#000000'}}"))[{{title}}]
  {{#if subtitle}}
  #v(0.5em)
  #text(size: 14pt, fill: gray)[{{subtitle}}]
  {{/if}}
]

#v(1em)

{{body}}

{{#if features}}
== Features
{{#each features}}
- {{this}}
{{/each}}
{{/if}}

{{#if contact}}
#v(1em)
#align(center)[
  {{#if contact.email}}Email: {{contact.email}}{{/if}}
]
{{/if}}"#;

    let data = SlickSheetData::new("Amazing Product")
        .with_subtitle("The Best Solution")
        .with_body("This product will change your life.")
        .with_feature("Fast performance")
        .with_feature("Easy to use")
        .with_contact(ContactInfo::with_email("sales@example.com"))
        .with_style(StyleHints {
            primary_color: Some("#e94560".to_string()),
            font_family: Some("Inter".to_string()),
            ..Default::default()
        });

    // Validate the template
    let validation = validate_template(template);
    assert!(validation.is_ok(), "Template should be valid");

    // Render the template
    let result = TemplateEngine::render(template, &data);
    assert!(result.is_ok(), "Template should render successfully");

    let rendered = result.unwrap();
    assert!(rendered.contains("Amazing Product"));
    assert!(rendered.contains("The Best Solution"));
    assert!(rendered.contains("Fast performance"));
    assert!(rendered.contains("Easy to use"));
    assert!(rendered.contains("sales\\@example.com")); // @ is escaped for Typst
    assert!(rendered.contains("\\#e94560")); // # is escaped for Typst
}

#[test]
fn test_conditional_rendering() {
    let template = "{{#if subtitle}}[{{subtitle}}]{{else}}[no subtitle]{{/if}}";

    // With subtitle
    let data_with = SlickSheetData::new("Test").with_subtitle("My Subtitle");
    let result = TemplateEngine::render(template, &data_with).unwrap();
    assert_eq!(result, "[My Subtitle]");

    // Without subtitle
    let data_without = SlickSheetData::new("Test");
    let result = TemplateEngine::render(template, &data_without).unwrap();
    assert_eq!(result, "[no subtitle]");
}

#[test]
fn test_loop_rendering() {
    let template = "Features: {{#each features}}{{this}}; {{/each}}";

    let data = SlickSheetData::default()
        .with_feature("A")
        .with_feature("B")
        .with_feature("C");

    let result = TemplateEngine::render(template, &data).unwrap();
    assert_eq!(result, "Features: A; B; C; ");
}

#[test]
fn test_nested_conditionals() {
    let template = "{{#if contact}}{{#if contact.email}}{{contact.email}}{{/if}}{{/if}}";

    let data = SlickSheetData::default().with_contact(ContactInfo::with_email("test@test.com"));

    let result = TemplateEngine::render(template, &data).unwrap();
    assert_eq!(result, "test\\@test.com"); // @ is escaped for Typst
}

#[test]
fn test_default_values() {
    let template =
        "Color: {{style.primaryColor | default: '#ffffff'}}, Font: {{style.fontFamily | default: 'Arial'}}";

    // With no style - # is escaped for Typst
    let data = SlickSheetData::new("Test");
    let result = TemplateEngine::render(template, &data).unwrap();
    assert_eq!(result, "Color: \\#ffffff, Font: Arial");

    // With style - # is escaped for Typst
    let data_styled = SlickSheetData::new("Test").with_style(StyleHints {
        primary_color: Some("#000000".to_string()),
        ..Default::default()
    });
    let result = TemplateEngine::render(template, &data_styled).unwrap();
    assert_eq!(result, "Color: \\#000000, Font: Arial");
}

#[test]
fn test_stats_rendering() {
    let template = "Stats count: {{stats.length}}";

    let data = SlickSheetData::default()
        .with_stat(Stat::new("50%", "Growth"))
        .with_stat(Stat::new("100", "Users"))
        .with_stat(Stat::new("$1M", "Revenue"));

    let result = TemplateEngine::render(template, &data).unwrap();
    assert_eq!(result, "Stats count: 3");
}

#[test]
fn test_empty_arrays() {
    let template = "{{#if features}}Has features{{else}}No features{{/if}}";

    // Empty features
    let data = SlickSheetData::new("Test");
    let result = TemplateEngine::render(template, &data).unwrap();
    assert_eq!(result, "No features");

    // With features
    let data_with = SlickSheetData::new("Test").with_feature("One");
    let result = TemplateEngine::render(template, &data_with).unwrap();
    assert_eq!(result, "Has features");
}

#[test]
fn test_parser_preserves_typst_syntax() {
    let template = r#"#set page(margin: 1in)
#text(fill: rgb("{{style.primaryColor | default: '#000'}}"))[{{title}}]"#;

    let nodes = parse_template(template).unwrap();

    // Check that Typst syntax is preserved in text nodes
    let has_typst_set = nodes
        .iter()
        .any(|n| matches!(n, super::parser::TemplateNode::Text(t) if t.contains("#set page")));
    assert!(has_typst_set, "Should preserve Typst #set commands");
}

#[test]
fn test_complex_sections_template() {
    let template = r#"{{#if sections}}
{{#each sections}}
== Section
{{/each}}
{{/if}}"#;

    let data = SlickSheetData::new("Test")
        .with_section(Section::text("Overview", "Content here"))
        .with_section(Section::list(
            "Features",
            vec!["A".to_string(), "B".to_string()],
        ));

    let result = TemplateEngine::render(template, &data).unwrap();
    assert!(result.contains("== Section"));
}

#[test]
fn test_whitespace_preservation() {
    let template = "Line 1\n{{title}}\nLine 3";
    let data = SlickSheetData::new("Middle");

    let result = TemplateEngine::render(template, &data).unwrap();
    assert_eq!(result, "Line 1\nMiddle\nLine 3");
}
