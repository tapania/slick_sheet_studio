//! Read template tool

#![allow(dead_code)]

use super::{AiTool, ToolResult};

/// Tool for reading the current Typst template
pub struct ReadTemplateTool;

impl ReadTemplateTool {
    /// Execute the read_template tool
    pub fn execute(template: &str) -> ToolResult {
        ToolResult::Success(template.to_string())
    }
}

impl AiTool for ReadTemplateTool {
    fn name(&self) -> &'static str {
        "read_template"
    }

    fn description(&self) -> &'static str {
        r#"Read the current Typst template. Returns the full template source with placeholders.

Templates use Handlebars-style placeholders:
- {{field}} - Simple value substitution (e.g., {{title}}, {{body}})
- {{field.subfield}} - Nested values (e.g., {{style.primaryColor}}, {{contact.email}})
- {{#if field}}...{{/if}} - Conditional sections
- {{#if field}}...{{else}}...{{/if}} - Conditional with else
- {{#each items}}...{{/each}} - Loop over arrays
- {{this}} - Current item in a loop
- {{field | default: 'value'}} - Default values if field is empty

Available data fields:
- title, subtitle, body (strings)
- features (array of strings)
- sections (array of section objects)
- stats (array of {value, label, color?})
- contact.email, contact.phone, contact.website, contact.address
- style.primaryColor, style.accentColor, style.fontFamily"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_template() {
        let template = "#set page(width: 8.5in)\n{{title}}";
        let result = ReadTemplateTool::execute(template);

        assert!(result.is_success());
        assert_eq!(result.message(), template);
    }

    #[test]
    fn test_read_empty_template() {
        let result = ReadTemplateTool::execute("");
        assert!(result.is_success());
        assert_eq!(result.message(), "");
    }
}
