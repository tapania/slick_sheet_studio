//! Data model for slick sheet content
//!
//! This module provides:
//! - `SlickSheetData` struct for JSON content storage
//! - Schema validation for data integrity
//! - Default data generators for templates
#![allow(dead_code)]

mod defaults;
mod schema;
mod validation;

#[cfg(test)]
mod tests;

pub use defaults::default_data_for_template;
pub use schema::{Section, SectionType, SlickSheetData};

// Public API - not all used internally yet
#[allow(unused_imports)]
pub use schema::{ContactInfo, Stat, StyleHints};
#[allow(unused_imports)]
pub use validation::{validate_schema, ValidationError};
