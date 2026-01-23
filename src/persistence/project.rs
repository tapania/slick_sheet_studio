//! Project data structure for save/load functionality

use serde::{Deserialize, Serialize};

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectMetadata {
    /// Project name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Project format version
    pub version: String,
    /// Creation timestamp (ISO 8601)
    pub created_at: Option<String>,
    /// Last modified timestamp (ISO 8601)
    pub modified_at: Option<String>,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            name: "Untitled Project".to_string(),
            description: None,
            version: "1.0.0".to_string(),
            created_at: None,
            modified_at: None,
        }
    }
}

impl ProjectMetadata {
    /// Create metadata with a specific name and current timestamp
    fn new_with_name(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            ..Default::default()
        }
    }
}

/// A slick sheet project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    /// Project metadata
    pub metadata: ProjectMetadata,
    /// Typst source code
    pub source: String,
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}

impl Project {
    /// Create a new project with default content
    pub fn new() -> Self {
        Self {
            metadata: ProjectMetadata::new_with_name("Untitled Project"),
            source: DEFAULT_SOURCE.to_string(),
        }
    }

    /// Create a project with a specific name
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            metadata: ProjectMetadata::new_with_name(name),
            source: DEFAULT_SOURCE.to_string(),
        }
    }

    /// Create a project from Typst source
    pub fn from_source(name: impl Into<String>, source: String) -> Self {
        Self {
            metadata: ProjectMetadata::new_with_name(name),
            source,
        }
    }

    /// Serialize to pretty JSON
    pub fn to_json_pretty(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("Serialization failed: {e}"))
    }

    /// Serialize to compact JSON
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Serialization failed: {e}"))
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Deserialization failed: {e}"))
    }

    /// Update the modified timestamp
    pub fn touch(&mut self) {
        self.metadata.modified_at = Some(chrono::Utc::now().to_rfc3339());
    }
}

/// Default Typst source for new projects
const DEFAULT_SOURCE: &str = r##"#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "Inter", size: 11pt)

= Hello World

Welcome to Slick Sheet Studio!

Edit this document to create your slick sheet.
"##;
