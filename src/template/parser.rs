//! Template parser for Handlebars-style syntax

// Allow unused items that are part of the public API for testing/future use
#![allow(dead_code)]

use std::collections::HashSet;

/// A node in the parsed template AST
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateNode {
    /// Raw text content
    Text(String),

    /// Simple variable substitution: {{field}} or {{field.subfield}}
    Variable {
        path: Vec<String>,
        default: Option<String>,
    },

    /// Conditional block: {{#if field}}...{{/if}}
    Conditional {
        path: Vec<String>,
        then_branch: Vec<TemplateNode>,
        else_branch: Vec<TemplateNode>,
    },

    /// Loop block: {{#each field}}...{{/each}}
    Loop {
        path: Vec<String>,
        body: Vec<TemplateNode>,
    },
}

/// Parse a template string into a list of nodes
pub fn parse_template(input: &str) -> Result<Vec<TemplateNode>, ParseError> {
    let mut parser = Parser::new(input);
    parser.parse_nodes(&[])
}

/// Extract all variable paths used in a template
pub fn extract_variables(nodes: &[TemplateNode]) -> HashSet<String> {
    let mut vars = HashSet::new();
    collect_variables(nodes, &mut vars);
    vars
}

fn collect_variables(nodes: &[TemplateNode], vars: &mut HashSet<String>) {
    for node in nodes {
        match node {
            TemplateNode::Variable { path, .. } => {
                vars.insert(path.join("."));
            }
            TemplateNode::Conditional {
                path,
                then_branch,
                else_branch,
            } => {
                vars.insert(path.join("."));
                collect_variables(then_branch, vars);
                collect_variables(else_branch, vars);
            }
            TemplateNode::Loop { path, body } => {
                vars.insert(path.join("."));
                collect_variables(body, vars);
            }
            TemplateNode::Text(_) => {}
        }
    }
}

/// Parse error types
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Unclosed tag
    UnclosedTag { tag: String, position: usize },
    /// Unexpected closing tag
    UnexpectedClosingTag { expected: String, found: String },
    /// Invalid syntax
    InvalidSyntax { message: String, position: usize },
    /// Empty variable name
    EmptyVariableName { position: usize },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnclosedTag { tag, position } => {
                write!(f, "Unclosed tag '{}' at position {}", tag, position)
            }
            ParseError::UnexpectedClosingTag { expected, found } => {
                write!(f, "Expected closing tag '{}', found '{}'", expected, found)
            }
            ParseError::InvalidSyntax { message, position } => {
                write!(f, "Invalid syntax at position {}: {}", position, message)
            }
            ParseError::EmptyVariableName { position } => {
                write!(f, "Empty variable name at position {}", position)
            }
        }
    }
}

impl std::error::Error for ParseError {}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn remaining(&self) -> &str {
        &self.input[self.pos..]
    }

    fn parse_nodes(&mut self, end_tags: &[&str]) -> Result<Vec<TemplateNode>, ParseError> {
        let mut nodes = Vec::new();

        while self.pos < self.input.len() {
            // Check for end tags
            for end_tag in end_tags {
                if self.remaining().starts_with(end_tag) {
                    return Ok(nodes);
                }
            }

            // Check for template tags
            if self.remaining().starts_with("{{") {
                if let Some(node) = self.parse_tag()? {
                    nodes.push(node);
                }
            } else {
                // Parse raw text until next {{ or end
                let text = self.parse_text();
                if !text.is_empty() {
                    nodes.push(TemplateNode::Text(text));
                }
            }
        }

        Ok(nodes)
    }

    fn parse_text(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.input.len() && !self.remaining().starts_with("{{") {
            self.pos += 1;
        }
        self.input[start..self.pos].to_string()
    }

    fn parse_tag(&mut self) -> Result<Option<TemplateNode>, ParseError> {
        let tag_start = self.pos;

        // Skip {{
        self.pos += 2;

        // Skip whitespace
        self.skip_whitespace();

        // Check for block tags
        if self.remaining().starts_with('#') {
            self.pos += 1;
            return self.parse_block_tag(tag_start);
        }

        // Check for closing tags
        if self.remaining().starts_with('/') {
            // Don't advance, let the parent handle it
            self.pos = tag_start;
            return Ok(None);
        }

        // Parse variable
        self.parse_variable(tag_start)
    }

    fn parse_variable(&mut self, tag_start: usize) -> Result<Option<TemplateNode>, ParseError> {
        self.skip_whitespace();

        let var_start = self.pos;

        // Read variable path until delimiter
        while self.pos < self.input.len() {
            let c = self.current_char();
            if c == '}' || c == '|' || c.is_whitespace() {
                break;
            }
            self.pos += 1;
        }

        // Parse path by splitting on dots
        let path_str = self.input[var_start..self.pos].trim();
        let path: Vec<String> = path_str
            .split('.')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        if path.is_empty() {
            return Err(ParseError::EmptyVariableName {
                position: var_start,
            });
        }

        self.skip_whitespace();

        // Parse optional default value: | default: 'value'
        let default = self.parse_default_value();

        self.skip_whitespace();

        // Expect }}
        if !self.remaining().starts_with("}}") {
            return Err(ParseError::InvalidSyntax {
                message: "Expected '}}' to close variable tag".to_string(),
                position: tag_start,
            });
        }
        self.pos += 2;

        Ok(Some(TemplateNode::Variable { path, default }))
    }

    fn parse_default_value(&mut self) -> Option<String> {
        if !self.remaining().starts_with('|') {
            return None;
        }
        self.pos += 1;
        self.skip_whitespace();

        if !self.remaining().starts_with("default:") {
            return None;
        }
        self.pos += 8; // skip "default:"
        self.skip_whitespace();

        // Parse quoted string
        let quote = self.current_char();
        if quote != '\'' && quote != '"' {
            return None;
        }
        self.pos += 1;
        let default_start = self.pos;

        while self.pos < self.input.len() && self.current_char() != quote {
            self.pos += 1;
        }

        let value = self.input[default_start..self.pos].to_string();

        if self.pos < self.input.len() {
            self.pos += 1; // skip closing quote
        }

        Some(value)
    }

    fn current_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or('\0')
    }

    fn parse_block_tag(&mut self, tag_start: usize) -> Result<Option<TemplateNode>, ParseError> {
        self.skip_whitespace();

        // Get the block type (if, each, etc.)
        let type_start = self.pos;
        while self.pos < self.input.len() {
            let c = self.current_char();
            if c.is_whitespace() || c == '}' {
                break;
            }
            self.pos += 1;
        }
        let block_type = &self.input[type_start..self.pos];

        self.skip_whitespace();

        // Get the variable path
        let path_start = self.pos;
        while self.pos < self.input.len() {
            let c = self.current_char();
            if c == '}' || c.is_whitespace() {
                break;
            }
            self.pos += 1;
        }
        let path_str = self.input[path_start..self.pos].trim();
        let path: Vec<String> = path_str.split('.').map(|s| s.to_string()).collect();

        self.skip_whitespace();

        // Expect }}
        if !self.remaining().starts_with("}}") {
            return Err(ParseError::InvalidSyntax {
                message: format!("Expected '}}' after block tag '{}'", block_type),
                position: self.pos,
            });
        }
        self.pos += 2;

        match block_type {
            "if" => self.parse_if_block(path, tag_start),
            "each" => self.parse_each_block(path, tag_start),
            _ => Err(ParseError::InvalidSyntax {
                message: format!("Unknown block type: {}", block_type),
                position: type_start,
            }),
        }
    }

    fn parse_if_block(
        &mut self,
        path: Vec<String>,
        tag_start: usize,
    ) -> Result<Option<TemplateNode>, ParseError> {
        // Parse the then branch until {{else}} or {{/if}}
        let then_branch = self.parse_nodes(&["{{else}}", "{{/if}}"])?;

        let mut else_branch = Vec::new();

        // Check if we hit {{else}}
        if self.remaining().starts_with("{{else}}") {
            self.pos += 8; // skip {{else}}
            else_branch = self.parse_nodes(&["{{/if}}"])?;
        }

        // Expect {{/if}}
        if !self.remaining().starts_with("{{/if}}") {
            return Err(ParseError::UnclosedTag {
                tag: "if".to_string(),
                position: tag_start,
            });
        }
        self.pos += 7; // skip {{/if}}

        Ok(Some(TemplateNode::Conditional {
            path,
            then_branch,
            else_branch,
        }))
    }

    fn parse_each_block(
        &mut self,
        path: Vec<String>,
        tag_start: usize,
    ) -> Result<Option<TemplateNode>, ParseError> {
        // Parse the body until {{/each}}
        let body = self.parse_nodes(&["{{/each}}"])?;

        // Expect {{/each}}
        if !self.remaining().starts_with("{{/each}}") {
            return Err(ParseError::UnclosedTag {
                tag: "each".to_string(),
                position: tag_start,
            });
        }
        self.pos += 9; // skip {{/each}}

        Ok(Some(TemplateNode::Loop { path, body }))
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.current_char().is_whitespace() {
            self.pos += 1;
        }
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_parse_simple_text() {
        let result = parse_template("Hello World").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], TemplateNode::Text("Hello World".to_string()));
    }

    #[test]
    fn test_parse_simple_variable() {
        let result = parse_template("{{title}}").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            TemplateNode::Variable { path, default } => {
                assert_eq!(path, &vec!["title".to_string()]);
                assert!(default.is_none());
            }
            _ => panic!("Expected Variable node"),
        }
    }

    #[test]
    fn test_parse_nested_variable() {
        let result = parse_template("{{style.primaryColor}}").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            TemplateNode::Variable { path, default } => {
                assert_eq!(path, &vec!["style".to_string(), "primaryColor".to_string()]);
                assert!(default.is_none());
            }
            _ => panic!("Expected Variable node"),
        }
    }

    #[test]
    fn test_parse_variable_with_default() {
        let result = parse_template("{{title | default: 'Untitled'}}").unwrap();
        match &result[0] {
            TemplateNode::Variable { path, default } => {
                assert_eq!(path, &vec!["title".to_string()]);
                assert_eq!(default, &Some("Untitled".to_string()));
            }
            _ => panic!("Expected Variable node"),
        }
    }

    #[test]
    fn test_parse_if_block() {
        let result = parse_template("{{#if subtitle}}has subtitle{{/if}}").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            TemplateNode::Conditional {
                path,
                then_branch,
                else_branch,
            } => {
                assert_eq!(path, &vec!["subtitle".to_string()]);
                assert_eq!(then_branch.len(), 1);
                assert!(else_branch.is_empty());
            }
            _ => panic!("Expected Conditional node"),
        }
    }

    #[test]
    fn test_parse_if_else_block() {
        let result = parse_template("{{#if title}}yes{{else}}no{{/if}}").unwrap();
        match &result[0] {
            TemplateNode::Conditional {
                then_branch,
                else_branch,
                ..
            } => {
                assert_eq!(then_branch.len(), 1);
                assert_eq!(else_branch.len(), 1);
            }
            _ => panic!("Expected Conditional node"),
        }
    }

    #[test]
    fn test_parse_each_block() {
        let result = parse_template("{{#each features}}item{{/each}}").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            TemplateNode::Loop { path, body } => {
                assert_eq!(path, &vec!["features".to_string()]);
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected Loop node"),
        }
    }

    #[test]
    fn test_parse_mixed_content() {
        let result = parse_template("Title: {{title}}, Subtitle: {{subtitle}}").unwrap();
        assert_eq!(result.len(), 4); // text, var, text, var
    }

    #[test]
    fn test_extract_variables() {
        let nodes = parse_template(
            "{{title}} {{#if subtitle}}{{subtitle}}{{/if}} {{#each features}}{{this}}{{/each}}",
        )
        .unwrap();
        let vars = extract_variables(&nodes);

        assert!(vars.contains("title"));
        assert!(vars.contains("subtitle"));
        assert!(vars.contains("features"));
        assert!(vars.contains("this"));
    }

    #[test]
    fn test_unclosed_if() {
        let result = parse_template("{{#if title}}content");
        assert!(result.is_err());
    }

    #[test]
    fn test_unclosed_each() {
        let result = parse_template("{{#each items}}content");
        assert!(result.is_err());
    }
}
