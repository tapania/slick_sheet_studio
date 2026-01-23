//! Write template tool

#![allow(dead_code)]

use super::{AiTool, ToolResult};
use crate::data::SlickSheetData;
use crate::template::{validate_template, TemplateEngine};

/// Tool for writing a new Typst template
pub struct WriteTemplateTool;

impl WriteTemplateTool {
    /// Execute the write_template tool
    ///
    /// Validates the template before accepting:
    /// 1. Validate template syntax (Handlebars placeholders)
    /// 2. Test rendering with current data
    /// 3. Test Typst compilation
    pub fn execute(
        new_template: &str,
        current_data: &SlickSheetData,
        compile_fn: impl FnOnce(&str) -> Result<String, Vec<String>>,
    ) -> Result<String, ToolResult> {
        // Step 1: Validate template syntax
        if let Err(errors) = validate_template(new_template) {
            let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            return Err(ToolResult::Error(format!(
                "Template syntax errors:\n{}",
                error_messages.join("\n")
            )));
        }

        // Step 2: Test rendering with current data
        let rendered = match TemplateEngine::render(new_template, current_data) {
            Ok(r) => r,
            Err(errors) => {
                return Err(ToolResult::Error(format!(
                    "Template rendering failed:\n{}",
                    errors.join("\n")
                )));
            }
        };

        // Step 3: Test Typst compilation
        if let Err(errors) = compile_fn(&rendered) {
            return Err(ToolResult::Error(format!(
                "Typst compilation failed:\n{}",
                errors.join("\n")
            )));
        }

        // Success - return the validated template
        Ok(new_template.to_string())
    }

    /// Execute without compilation test (for simpler validation)
    pub fn execute_without_compile(
        new_template: &str,
        current_data: &SlickSheetData,
    ) -> Result<String, ToolResult> {
        // Step 1: Validate template syntax
        if let Err(errors) = validate_template(new_template) {
            let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            return Err(ToolResult::Error(format!(
                "Template syntax errors:\n{}",
                error_messages.join("\n")
            )));
        }

        // Step 2: Test rendering with current data
        if let Err(errors) = TemplateEngine::render(new_template, current_data) {
            return Err(ToolResult::Error(format!(
                "Template rendering failed:\n{}",
                errors.join("\n")
            )));
        }

        Ok(new_template.to_string())
    }
}

impl AiTool for WriteTemplateTool {
    fn name(&self) -> &'static str {
        "write_template"
    }

    fn description(&self) -> &'static str {
        r#"Update the Typst template.

IMPORTANT: Always write the COMPLETE template, not partial updates.
The system validates and test-compiles your template before accepting it.

Templates use Handlebars-style placeholders:
- {{field}} - Simple value substitution
- {{#if field}}...{{/if}} - Conditional sections
- {{#each items}}...{{/each}} - Loop over arrays
- {{field | default: 'value'}} - Default values

Example template structure:
```
#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "{{style.fontFamily | default: 'Inter'}}", size: 11pt)

#align(center)[
  #text(size: 24pt, weight: "bold")[{{title}}]
  {{#if subtitle}}
  #v(0.5em)
  #text(fill: gray)[{{subtitle}}]
  {{/if}}
]

{{body}}

{{#if features}}
== Features
{{#each features}}
- {{this}}
{{/each}}
{{/if}}
```"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_template_valid() {
        let template = "{{title}}";
        let data = SlickSheetData::new("Test");

        let result = WriteTemplateTool::execute_without_compile(template, &data);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), template);
    }

    #[test]
    fn test_write_template_with_conditionals() {
        let template = "{{#if subtitle}}{{subtitle}}{{else}}No subtitle{{/if}}";
        let data = SlickSheetData::new("Test");

        let result = WriteTemplateTool::execute_without_compile(template, &data);

        assert!(result.is_ok());
    }

    #[test]
    fn test_write_template_with_loops() {
        let template = "{{#each features}}{{this}}{{/each}}";
        let data = SlickSheetData::new("Test")
            .with_feature("A")
            .with_feature("B");

        let result = WriteTemplateTool::execute_without_compile(template, &data);

        assert!(result.is_ok());
    }

    #[test]
    fn test_write_template_invalid_syntax() {
        let template = "{{#if unclosed}}content";
        let data = SlickSheetData::new("Test");

        let result = WriteTemplateTool::execute_without_compile(template, &data);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message().contains("syntax"));
    }

    #[test]
    fn test_write_template_empty() {
        let template = "";
        let data = SlickSheetData::new("Test");

        let result = WriteTemplateTool::execute_without_compile(template, &data);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message().contains("empty"));
    }

    #[test]
    fn test_write_template_complex() {
        let template = r#"#set page(width: 8.5in)
#text(fill: rgb("{{style.primaryColor | default: '#000'}}"))[{{title}}]
{{#if subtitle}}Subtitle: {{subtitle}}{{/if}}
{{#each features}}- {{this}}
{{/each}}"#;

        let data = SlickSheetData::new("Product")
            .with_subtitle("Tagline")
            .with_feature("Fast")
            .with_feature("Reliable");

        let result = WriteTemplateTool::execute_without_compile(template, &data);

        assert!(result.is_ok());
    }
}
