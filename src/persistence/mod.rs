//! Persistence module - Save/load and export functionality
//!
//! This module provides:
//! - Project save/load with JSON format
//! - PDF export
//! - File handling utilities
#![allow(dead_code)]

pub mod export;
pub mod project;

#[cfg(test)]
mod tests;

pub use export::pdf_bytes_from_source;
pub use export::pdf_data_url;
pub use project::Project;
pub use project::ProjectMetadata;
