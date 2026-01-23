//! Template validation logic

// Allow unused items that are part of the public API for testing/future use
#![allow(dead_code)]

use super::parser::{extract_variables, parse_template, ParseError};
use crate::data::SlickSheetData;
use thiserror::Error;

/// Template validation errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum TemplateValidationError {
    /// Parse error in template
    #[error("Template parse error: {0}")]
    ParseError(String),

    /// Unknown variable in template
    #[error("Unknown variable: {0}")]
    UnknownVariable(String),

    /// Template is empty
    #[error("Template cannot be empty")]
    EmptyTemplate,
}

impl From<ParseError> for TemplateValidationError {
    fn from(err: ParseError) -> Self {
        TemplateValidationError::ParseError(err.to_string())
    }
}

/// Known valid variable paths that templates can use
const KNOWN_VARIABLES: &[&str] = &[
    // Top-level fields
    "title",
    "subtitle",
    "body",
    // Arrays
    "sections",
    "features",
    "stats",
    // Nested style fields
    "style",
    "style.primaryColor",
    "style.primary_color",
    "style.accentColor",
    "style.accent_color",
    "style.fontFamily",
    "style.font_family",
    // Nested contact fields
    "contact",
    "contact.email",
    "contact.phone",
    "contact.website",
    "contact.address",
    // Array lengths
    "sections.length",
    "features.length",
    "stats.length",
    // Loop variables
    "this",
    "@index",
    // Section fields (used in loops)
    "heading",
    "content",
    "type",
    "items",
    "rows",
    "columns",
    // Stat fields (used in loops)
    "value",
    "label",
    "color",
];

/// Validate a template string
///
/// Checks for:
/// - Valid syntax (parseable)
/// - Known variable names (warns about unknowns)
pub fn validate_template(template: &str) -> Result<Vec<String>, Vec<TemplateValidationError>> {
    if template.trim().is_empty() {
        return Err(vec![TemplateValidationError::EmptyTemplate]);
    }

    // Try to parse the template
    let nodes = parse_template(template)
        .map_err(|e| vec![TemplateValidationError::ParseError(e.to_string())])?;

    // Extract all variables used
    let used_vars = extract_variables(&nodes);

    // Check for unknown variables - generate warnings (not errors) for metadata keys
    let warnings: Vec<String> = used_vars
        .iter()
        .filter(|var| !is_known_variable(var))
        .map(|var| format!("Unknown variable '{}' (might be a metadata key)", var))
        .collect();

    Ok(warnings)
}

/// Check if a variable path is known
fn is_known_variable(var: &str) -> bool {
    KNOWN_VARIABLES.contains(&var)
}

/// Validate that a template compiles successfully with data
pub fn validate_template_with_data(
    template: &str,
    data: &SlickSheetData,
    compile_fn: impl FnOnce(&str) -> Result<String, Vec<String>>,
) -> Result<(), Vec<String>> {
    use super::engine::TemplateEngine;

    // First render the template with data
    let rendered = TemplateEngine::render(template, data)?;

    // Then try to compile the result
    compile_fn(&rendered)?;

    Ok(())
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_valid_template() {
        let result = validate_template("Hello {{title}}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_template() {
        let result = validate_template("");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err()[0],
            TemplateValidationError::EmptyTemplate
        ));
    }

    #[test]
    fn test_template_with_syntax_error() {
        let result = validate_template("{{#if unclosed}}content");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err()[0],
            TemplateValidationError::ParseError(_)
        ));
    }

    #[test]
    fn test_known_variables() {
        assert!(is_known_variable("title"));
        assert!(is_known_variable("subtitle"));
        assert!(is_known_variable("style.primaryColor"));
        assert!(is_known_variable("contact.email"));
        assert!(is_known_variable("this"));
    }

    #[test]
    fn test_unknown_variable_warning() {
        // Unknown variables generate warnings, not errors
        let result = validate_template("{{unknownField}}");
        assert!(result.is_ok());
        let warnings = result.unwrap();
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("unknownField"));
    }

    #[test]
    fn test_template_with_all_features() {
        let template = r#"
            {{title}}
            {{#if subtitle}}{{subtitle}}{{/if}}
            {{body}}
            {{#each features}}{{this}}{{/each}}
            {{#if contact}}{{contact.email}}{{/if}}
            {{style.primaryColor | default: '#000'}}
        "#;
        let result = validate_template(template);
        assert!(result.is_ok());
    }
}
