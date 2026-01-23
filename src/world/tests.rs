//! Tests for VirtualWorld

use super::*;

#[test]
fn test_virtual_world_new() {
    let world = VirtualWorld::new("Hello World");
    assert_eq!(world.main.text(), "Hello World");
}

#[test]
fn test_virtual_world_set_source() {
    let mut world = VirtualWorld::new("Initial");
    world.set_source("Updated");
    assert_eq!(world.main.text(), "Updated");
}

#[test]
fn test_virtual_world_add_file() {
    let mut world = VirtualWorld::new("main");
    world.add_file("helper.typ", "#let helper = 1".as_bytes().to_vec());

    let id = FileId::new(None, VirtualPath::new("helper.typ"));
    let source = world.source(id).unwrap();
    assert_eq!(source.text(), "#let helper = 1");
}

#[test]
fn test_compile_hello_world() {
    let source = r#"Hello World"#;
    let result = VirtualWorld::compile_to_svg(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    let svg = result.unwrap();
    assert!(svg.contains("<svg"), "Output should be SVG");
    assert!(svg.contains("</svg>"), "Output should be valid SVG");
}

#[test]
fn test_compile_with_formatting() {
    let source = r#"#set text(size: 14pt)
= Heading
Some *bold* text."#;
    let result = VirtualWorld::compile_to_svg(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
}

#[test]
fn test_compile_error() {
    let source = r#"#let x = "#; // Unterminated string
    let result = VirtualWorld::compile_to_svg(source);
    assert!(result.is_err(), "Should fail to compile invalid Typst");
}

#[test]
fn test_fonts_loaded() {
    let world = VirtualWorld::new("test");
    let fonts = world.fonts();
    assert!(fonts.len() >= 2, "Should have at least 2 fonts loaded");
}

#[test]
fn test_font_book() {
    let world = VirtualWorld::new("test");
    let fonts = world.fonts();
    assert!(!fonts.is_empty(), "Should have fonts loaded");
}

#[test]
fn test_world_trait_main() {
    let world = VirtualWorld::new("test");
    let main_id = world.main();
    assert_eq!(
        main_id.vpath().as_rootless_path().to_str(),
        Some("main.typ")
    );
}

#[test]
fn test_world_trait_source() {
    let world = VirtualWorld::new("Hello");
    let main_id = world.main();
    let source = world.source(main_id).unwrap();
    assert_eq!(source.text(), "Hello");
}

#[test]
fn test_world_trait_file_not_found() {
    let world = VirtualWorld::new("test");
    let id = FileId::new(None, VirtualPath::new("nonexistent.typ"));
    let result = world.file(id);
    assert!(result.is_err());
}

#[test]
fn test_world_trait_font() {
    let world = VirtualWorld::new("test");
    let font = world.font(0);
    assert!(font.is_some(), "Should have at least one font");
}

#[test]
fn test_world_trait_today() {
    let world = VirtualWorld::new("test");
    let today = world.today(None);
    assert!(today.is_some(), "Should return current date");
}

#[test]
fn test_compile_with_link_extracts_cmd_url() {
    // Test that cmd:// links are extracted and added to SVG
    let source = r#"#link("cmd://edit/title")[Click to Edit Title]"#;
    let result = VirtualWorld::compile_to_svg(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    let svg = result.unwrap();

    // The SVG should contain an <a> element with the cmd:// URL
    assert!(
        svg.contains(r#"href="cmd://edit/title""#),
        "SVG should contain cmd://edit/title link. SVG: {}",
        svg
    );
}

#[test]
fn test_compile_with_multiple_links() {
    // Test multiple links in a document
    let source = r#"
#link("cmd://edit/title")[Title]

#link("cmd://edit/subtitle")[Subtitle]

#link("cmd://edit/body")[Body content]
"#;
    let result = VirtualWorld::compile_to_svg(source);
    assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    let svg = result.unwrap();

    assert!(
        svg.contains(r#"href="cmd://edit/title""#),
        "SVG should contain title link"
    );
    assert!(
        svg.contains(r#"href="cmd://edit/subtitle""#),
        "SVG should contain subtitle link"
    );
    assert!(
        svg.contains(r#"href="cmd://edit/body""#),
        "SVG should contain body link"
    );
}
