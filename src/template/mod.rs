//! Template engine for rendering Typst with data
//!
//! This module provides:
//! - Handlebars-style template parsing
//! - Data binding and rendering
//! - Template validation

mod engine;
mod parser;
mod validation;

#[cfg(test)]
mod tests;

pub use engine::TemplateEngine;
pub use validation::validate_template;

// Re-exports for public API (not all used internally yet)
#[allow(unused_imports)]
pub use parser::{parse_template, TemplateNode};
#[allow(unused_imports)]
pub use validation::TemplateValidationError;
