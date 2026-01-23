//! Template rendering engine

use super::parser::{parse_template, TemplateNode};
use crate::data::{Section, SectionType, SlickSheetData};

/// Template rendering engine
pub struct TemplateEngine;

impl TemplateEngine {
    /// Render a template with data
    pub fn render(template: &str, data: &SlickSheetData) -> Result<String, Vec<String>> {
        let nodes = parse_template(template).map_err(|e| vec![e.to_string()])?;
        let mut output = String::new();
        let mut errors = Vec::new();

        Self::render_nodes(&nodes, data, &mut output, &mut errors, None);

        if errors.is_empty() {
            Ok(output)
        } else {
            Err(errors)
        }
    }

    fn render_nodes(
        nodes: &[TemplateNode],
        data: &SlickSheetData,
        output: &mut String,
        errors: &mut Vec<String>,
        loop_context: Option<&LoopContext>,
    ) {
        for node in nodes {
            match node {
                TemplateNode::Text(text) => {
                    output.push_str(text);
                }
                TemplateNode::Variable { path, default } => {
                    let value = Self::resolve_path(path, data, loop_context);
                    let rendered = value.or_else(|| default.clone()).unwrap_or_default();
                    // Image IDs are safe system-generated identifiers, don't escape them
                    // They only contain: img_ prefix + hex characters
                    let is_image_ref = path.first().map(|s| s == "images").unwrap_or(false);
                    if is_image_ref {
                        output.push_str(&rendered);
                    } else {
                        // Escape Typst special characters in user data
                        output.push_str(&Self::escape_typst(&rendered));
                    }
                }
                TemplateNode::Conditional {
                    path,
                    then_branch,
                    else_branch,
                } => {
                    let is_truthy = Self::is_path_truthy(path, data, loop_context);
                    if is_truthy {
                        Self::render_nodes(then_branch, data, output, errors, loop_context);
                    } else {
                        Self::render_nodes(else_branch, data, output, errors, loop_context);
                    }
                }
                TemplateNode::Loop { path, body } => {
                    Self::render_loop(path, body, data, output, errors, loop_context);
                }
            }
        }
    }

    fn render_loop(
        path: &[String],
        body: &[TemplateNode],
        data: &SlickSheetData,
        output: &mut String,
        errors: &mut Vec<String>,
        parent_context: Option<&LoopContext>,
    ) {
        let items = Self::resolve_array(path, data, parent_context);

        for (index, item) in items.iter().enumerate() {
            let context = LoopContext {
                item: item.clone(),
                index,
                parent: parent_context,
            };
            Self::render_nodes(body, data, output, errors, Some(&context));
        }
    }

    fn resolve_path(
        path: &[String],
        data: &SlickSheetData,
        loop_context: Option<&LoopContext>,
    ) -> Option<String> {
        if path.is_empty() {
            return None;
        }

        let first = &path[0];

        // Check for special loop variables
        if first == "this" {
            return loop_context.map(|ctx| ctx.item.clone());
        }

        if first == "@index" {
            return loop_context.map(|ctx| ctx.index.to_string());
        }

        // Handle nested paths
        if path.len() == 1 {
            Self::resolve_simple_path(first, data)
        } else {
            Self::resolve_nested_path(path, data)
        }
    }

    fn resolve_simple_path(key: &str, data: &SlickSheetData) -> Option<String> {
        match key {
            "title" => Some(data.title.clone()),
            "subtitle" => data.subtitle.clone(),
            "body" => Some(data.body.clone()),
            _ => {
                // Check metadata
                data.metadata.get(key).cloned()
            }
        }
    }

    fn resolve_nested_path(path: &[String], data: &SlickSheetData) -> Option<String> {
        if path.len() < 2 {
            return None;
        }

        let first = &path[0];
        let second = &path[1];

        match first.as_str() {
            "style" => {
                let style = data.style.as_ref()?;
                match second.as_str() {
                    "primaryColor" | "primary_color" => style.primary_color.clone(),
                    "accentColor" | "accent_color" => style.accent_color.clone(),
                    "fontFamily" | "font_family" => style.font_family.clone(),
                    _ => None,
                }
            }
            "contact" => {
                let contact = data.contact.as_ref()?;
                match second.as_str() {
                    "email" => contact.email.clone(),
                    "phone" => contact.phone.clone(),
                    "website" => contact.website.clone(),
                    "address" => contact.address.clone(),
                    _ => None,
                }
            }
            "stats" => {
                // Handle stats.length
                if second == "length" {
                    return Some(data.stats.len().to_string());
                }
                None
            }
            "sections" => {
                // Handle sections.length
                if second == "length" {
                    return Some(data.sections.len().to_string());
                }
                None
            }
            "features" => {
                // Handle features.length
                if second == "length" {
                    return Some(data.features.len().to_string());
                }
                None
            }
            "images" => {
                // Handle images.X - returns the image path for use in #image() calls
                // The template should use: #image("{{images.logo}}", ...)
                // JSON data should include full path with extension: "logo": "img_abc123.png"
                // This returns the path, so it becomes: #image("img_abc123.png", ...)
                data.images.get(second).cloned()
            }
            _ => None,
        }
    }

    fn is_path_truthy(
        path: &[String],
        data: &SlickSheetData,
        loop_context: Option<&LoopContext>,
    ) -> bool {
        if path.is_empty() {
            return false;
        }

        let first = &path[0];

        // Check for arrays - truthy if non-empty
        match first.as_str() {
            "sections" => !data.sections.is_empty(),
            "features" => !data.features.is_empty(),
            "stats" => !data.stats.is_empty(),
            "subtitle" => data.subtitle.as_ref().is_some_and(|s| !s.is_empty()),
            "contact" => data.contact.is_some(),
            "style" => data.style.is_some(),
            "images" => {
                // If path is just "images", check if any images exist
                // If path is "images.X", check if that specific image exists
                if path.len() == 1 {
                    !data.images.is_empty()
                } else {
                    data.images.contains_key(&path[1])
                }
            }
            _ => {
                // Try to resolve as a value and check if non-empty
                Self::resolve_path(path, data, loop_context)
                    .map(|v| !v.is_empty())
                    .unwrap_or(false)
            }
        }
    }

    fn resolve_array(
        path: &[String],
        data: &SlickSheetData,
        _loop_context: Option<&LoopContext>,
    ) -> Vec<String> {
        if path.is_empty() {
            return Vec::new();
        }

        let first = &path[0];

        match first.as_str() {
            "features" => data.features.clone(),
            "sections" => data.sections.iter().map(Self::section_to_string).collect(),
            "stats" => data
                .stats
                .iter()
                .map(|s| format!("{}: {}", s.value, s.label))
                .collect(),
            _ => Vec::new(),
        }
    }

    fn section_to_string(section: &Section) -> String {
        match section.section_type {
            SectionType::Text => format!("{}: {}", section.heading, section.content),
            SectionType::List => {
                let items = section
                    .items
                    .as_ref()
                    .map(|i| i.join(", "))
                    .unwrap_or_default();
                format!("{}: [{}]", section.heading, items)
            }
            SectionType::Table => {
                format!("{}: <table>", section.heading)
            }
            SectionType::Quote => {
                format!("{}: \"{}\"", section.heading, section.content)
            }
        }
    }

    /// Escape Typst special characters in user-provided content
    ///
    /// This prevents user data from being interpreted as Typst syntax.
    /// Characters that need escaping to prevent compilation errors:
    /// - `@` - label references
    /// - `<` and `>` - label definitions
    /// - `[` and `]` - content blocks (causes "unclosed delimiter")
    /// - `#` - code mode / function calls
    /// - `$` - math mode
    /// - `*` and `_` - emphasis markers
    /// - `\` - escape character itself
    fn escape_typst(s: &str) -> String {
        let mut result = String::with_capacity(s.len() + s.len() / 4);
        for c in s.chars() {
            match c {
                '@' => result.push_str("\\@"),
                '<' => result.push_str("\\<"),
                '>' => result.push_str("\\>"),
                '[' => result.push_str("\\["),
                ']' => result.push_str("\\]"),
                '#' => result.push_str("\\#"),
                '$' => result.push_str("\\$"),
                '*' => result.push_str("\\*"),
                '_' => result.push_str("\\_"),
                '\\' => result.push_str("\\\\"),
                _ => result.push(c),
            }
        }
        result
    }
}

/// Context for loop iterations
struct LoopContext<'a> {
    item: String,
    index: usize,
    #[allow(dead_code)]
    parent: Option<&'a LoopContext<'a>>,
}

#[cfg(test)]
mod engine_tests {
    use super::*;
    use crate::data::{ContactInfo, Stat, StyleHints};

    #[test]
    fn test_render_simple_text() {
        let data = SlickSheetData::default();
        let result = TemplateEngine::render("Hello World", &data).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_render_variable() {
        let data = SlickSheetData::new("My Title");
        let result = TemplateEngine::render("Title: {{title}}", &data).unwrap();
        assert_eq!(result, "Title: My Title");
    }

    #[test]
    fn test_render_variable_with_default() {
        let data = SlickSheetData::default();
        let result =
            TemplateEngine::render("Subtitle: {{subtitle | default: 'None'}}", &data).unwrap();
        assert_eq!(result, "Subtitle: None");
    }

    #[test]
    fn test_render_nested_variable() {
        let data = SlickSheetData::default().with_style(StyleHints {
            primary_color: Some("#ff0000".to_string()),
            ..Default::default()
        });
        let result = TemplateEngine::render("Color: {{style.primaryColor}}", &data).unwrap();
        // # is escaped to prevent Typst code mode
        assert_eq!(result, "Color: \\#ff0000");
    }

    #[test]
    fn test_render_conditional_true() {
        let data = SlickSheetData::new("Title").with_subtitle("Subtitle");
        let result = TemplateEngine::render("{{#if subtitle}}Has subtitle{{/if}}", &data).unwrap();
        assert_eq!(result, "Has subtitle");
    }

    #[test]
    fn test_render_conditional_false() {
        let data = SlickSheetData::new("Title");
        let result = TemplateEngine::render("{{#if subtitle}}Has subtitle{{/if}}", &data).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_render_conditional_else() {
        let data = SlickSheetData::new("Title");
        let result = TemplateEngine::render("{{#if subtitle}}yes{{else}}no{{/if}}", &data).unwrap();
        assert_eq!(result, "no");
    }

    #[test]
    fn test_render_loop() {
        let data = SlickSheetData::default()
            .with_feature("Feature 1")
            .with_feature("Feature 2");
        let result =
            TemplateEngine::render("{{#each features}}[{{this}}]{{/each}}", &data).unwrap();
        assert_eq!(result, "[Feature 1][Feature 2]");
    }

    #[test]
    fn test_render_contact() {
        let data = SlickSheetData::default().with_contact(ContactInfo {
            email: Some("test@example.com".to_string()),
            ..Default::default()
        });
        let result = TemplateEngine::render("Email: {{contact.email}}", &data).unwrap();
        // @ is escaped to prevent Typst label reference errors
        assert_eq!(result, "Email: test\\@example.com");
    }

    #[test]
    fn test_render_stats_length() {
        let data = SlickSheetData::default()
            .with_stat(Stat::new("50%", "Growth"))
            .with_stat(Stat::new("100", "Users"));
        let result = TemplateEngine::render("Stats: {{stats.length}}", &data).unwrap();
        assert_eq!(result, "Stats: 2");
    }

    #[test]
    fn test_render_complex_template() {
        let data = SlickSheetData::new("Product")
            .with_subtitle("Amazing Product")
            .with_feature("Fast")
            .with_feature("Reliable");

        let template = r#"# {{title}}
{{#if subtitle}}## {{subtitle}}{{/if}}
Features:
{{#each features}}- {{this}}
{{/each}}"#;

        let result = TemplateEngine::render(template, &data).unwrap();
        assert!(result.contains("# Product"));
        assert!(result.contains("## Amazing Product"));
        assert!(result.contains("- Fast"));
        assert!(result.contains("- Reliable"));
    }

    #[test]
    fn test_escape_typst_special_chars() {
        // Test that @ symbols are escaped to prevent label reference errors
        let data = SlickSheetData::default().with_contact(ContactInfo {
            email: Some("user@example.com".to_string()),
            website: Some("slicksheet.studio".to_string()),
            ..Default::default()
        });
        let result = TemplateEngine::render("Email: {{contact.email}}", &data).unwrap();
        assert_eq!(result, "Email: user\\@example.com");
    }

    #[test]
    fn test_escape_typst_angle_brackets() {
        // Test that < > are escaped to prevent label definition errors
        let data = SlickSheetData::new("Test <label> Title");
        let result = TemplateEngine::render("Title: {{title}}", &data).unwrap();
        assert_eq!(result, "Title: Test \\<label\\> Title");
    }

    #[test]
    fn test_render_image_reference() {
        // Image paths in JSON now include the extension
        let data = SlickSheetData::default().with_image("logo", "img_abc123.png");
        let result = TemplateEngine::render("#image(\"{{images.logo}}\")", &data).unwrap();
        // Image path should not be escaped since it's a safe identifier
        assert_eq!(result, "#image(\"img_abc123.png\")");
    }

    #[test]
    fn test_render_image_conditional() {
        // Test with image present
        let data_with_image = SlickSheetData::default().with_image("logo", "img_abc123");
        let template = "{{#if images.logo}}Has logo{{else}}No logo{{/if}}";
        let result = TemplateEngine::render(template, &data_with_image).unwrap();
        assert_eq!(result, "Has logo");

        // Test without image
        let data_without_image = SlickSheetData::default();
        let result = TemplateEngine::render(template, &data_without_image).unwrap();
        assert_eq!(result, "No logo");
    }

    #[test]
    fn test_render_images_conditional() {
        // Test with any images present
        let data_with_images = SlickSheetData::default()
            .with_image("logo", "img_1")
            .with_image("banner", "img_2");
        let template = "{{#if images}}Has images{{else}}No images{{/if}}";
        let result = TemplateEngine::render(template, &data_with_images).unwrap();
        assert_eq!(result, "Has images");

        // Test without images
        let data_without_images = SlickSheetData::default();
        let result = TemplateEngine::render(template, &data_without_images).unwrap();
        assert_eq!(result, "No images");
    }

    #[test]
    fn test_render_missing_image_returns_empty() {
        let data = SlickSheetData::default();
        let result = TemplateEngine::render("#image(\"{{images.logo}}\")", &data).unwrap();
        // Missing image reference should render as empty string
        assert_eq!(result, "#image(\"\")");
    }
}
