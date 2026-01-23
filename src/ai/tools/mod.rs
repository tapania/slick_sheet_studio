//! AI Tools for JSON and Template operations
//!
//! This module provides tools for the AI agent to read and write
//! JSON content data and Typst templates with validation.
//!
//! Note: These tools are part of the public API for planned AI features.

#![allow(dead_code)]

mod generate_image;
mod read_json;
mod read_template;
mod write_json;
mod write_template;

#[cfg(test)]
mod tests;

pub use generate_image::GenerateImageTool;
pub use read_json::ReadJsonTool;
pub use read_template::ReadTemplateTool;
pub use write_json::WriteJsonTool;
pub use write_template::WriteTemplateTool;

/// Result of a tool execution
#[derive(Debug, Clone)]
pub enum ToolResult {
    /// Tool executed successfully
    Success(String),
    /// Tool execution failed with error
    Error(String),
}

impl ToolResult {
    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        matches!(self, ToolResult::Success(_))
    }

    /// Get the message (success or error)
    pub fn message(&self) -> &str {
        match self {
            ToolResult::Success(msg) => msg,
            ToolResult::Error(msg) => msg,
        }
    }
}

/// Trait for AI tools
pub trait AiTool {
    /// Get the tool name
    fn name(&self) -> &'static str;

    /// Get the tool description for the AI
    fn description(&self) -> &'static str;
}
