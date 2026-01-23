//! Slick Sheet Studio - Library crate
//!
//! This library provides:
//! - Typst compilation via VirtualWorld
//! - Template engine with Handlebars-style syntax
//! - Data models for slick sheet content
//! - AI agent integration
//! - Persistence (save/load/export)

pub mod ai;
pub mod data;
pub mod images;
pub mod persistence;
pub mod template;
pub mod templates;
pub mod world;

// Re-export editor for WASM builds and tests
#[cfg(any(target_arch = "wasm32", test))]
pub mod editor;
