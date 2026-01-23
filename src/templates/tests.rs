//! Tests for the templates module

use super::{Template, TemplateCategory, TemplateGallery, TEMPLATES};
use crate::world::VirtualWorld;

// ============================================================================
// Template Definition Tests
// ============================================================================

#[test]
fn test_template_has_required_fields() {
    let template = Template {
        id: "test",
        name: "Test Template",
        description: "A test template",
        category: TemplateCategory::Marketing,
        preview_svg: None,
        source: "= Hello",
    };

    assert_eq!(template.id, "test");
    assert!(!template.name.is_empty());
    assert!(!template.description.is_empty());
    assert!(!template.source.is_empty());
}

#[test]
fn test_template_category_variants() {
    let categories = vec![
        TemplateCategory::Marketing,
        TemplateCategory::Business,
        TemplateCategory::Event,
        TemplateCategory::Data,
        TemplateCategory::Minimal,
    ];

    // Ensure all categories can be created
    assert_eq!(categories.len(), 5);
}

#[test]
fn test_template_category_display() {
    assert_eq!(TemplateCategory::Marketing.as_str(), "Marketing");
    assert_eq!(TemplateCategory::Business.as_str(), "Business");
    assert_eq!(TemplateCategory::Event.as_str(), "Event");
    assert_eq!(TemplateCategory::Data.as_str(), "Data");
    assert_eq!(TemplateCategory::Minimal.as_str(), "Minimal");
}

// ============================================================================
// Template Gallery Tests
// ============================================================================

#[test]
fn test_gallery_contains_all_templates() {
    let gallery = TemplateGallery::new();

    // Should have exactly 10 templates
    assert_eq!(gallery.templates().len(), 10);
}

#[test]
fn test_gallery_get_by_id() {
    let gallery = TemplateGallery::new();

    // Should find product-sheet template
    let template = gallery.get("product-sheet");
    assert!(template.is_some());
    assert_eq!(template.unwrap().id, "product-sheet");
}

#[test]
fn test_gallery_get_unknown_id_returns_none() {
    let gallery = TemplateGallery::new();

    let template = gallery.get("nonexistent");
    assert!(template.is_none());
}

#[test]
fn test_gallery_filter_by_category() {
    let gallery = TemplateGallery::new();

    let marketing = gallery.by_category(TemplateCategory::Marketing);
    assert!(!marketing.is_empty());

    for template in marketing {
        assert_eq!(template.category, TemplateCategory::Marketing);
    }
}

#[test]
fn test_gallery_all_ids_unique() {
    let gallery = TemplateGallery::new();

    let ids: Vec<&str> = gallery.templates().iter().map(|t| t.id).collect();
    let mut unique_ids = ids.clone();
    unique_ids.sort();
    unique_ids.dedup();

    assert_eq!(ids.len(), unique_ids.len(), "Template IDs must be unique");
}

// ============================================================================
// Template Compilation Tests
// ============================================================================

#[test]
fn test_all_templates_compile() {
    for template in TEMPLATES.iter() {
        let result = VirtualWorld::compile_to_svg(template.source);

        assert!(
            result.is_ok(),
            "Template '{}' failed to compile: {:?}",
            template.id,
            result.err()
        );
    }
}

#[test]
fn test_product_sheet_template_compiles() {
    let gallery = TemplateGallery::new();
    let template = gallery
        .get("product-sheet")
        .expect("product-sheet should exist");

    let result = VirtualWorld::compile_to_svg(template.source);
    assert!(result.is_ok());

    let svg = result.unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_event_flyer_template_compiles() {
    let gallery = TemplateGallery::new();
    let template = gallery
        .get("event-flyer")
        .expect("event-flyer should exist");

    let result = VirtualWorld::compile_to_svg(template.source);
    assert!(result.is_ok());
}

#[test]
fn test_one_pager_template_compiles() {
    let gallery = TemplateGallery::new();
    let template = gallery.get("one-pager").expect("one-pager should exist");

    let result = VirtualWorld::compile_to_svg(template.source);
    assert!(result.is_ok());
}

// ============================================================================
// Template Content Tests
// ============================================================================

#[test]
fn test_templates_have_page_setup() {
    for template in TEMPLATES.iter() {
        assert!(
            template.source.contains("#set page") || template.source.contains("set page"),
            "Template '{}' should have page setup",
            template.id
        );
    }
}

#[test]
fn test_templates_use_fonts() {
    // At least some templates should specify fonts
    let has_font_setup = TEMPLATES
        .iter()
        .any(|t| t.source.contains("#set text") || t.source.contains("font:"));

    assert!(has_font_setup, "At least one template should set fonts");
}
