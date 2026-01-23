//! Content model for structured slick sheet data

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Content model for a slick sheet
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Content {
    /// Main title
    pub title: String,
    /// Optional subtitle
    pub subtitle: Option<String>,
    /// Main body text
    pub body: String,
    /// Optional image URL
    pub image_url: Option<String>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Default for Content {
    fn default() -> Self {
        Self {
            title: "Hello World".to_string(),
            subtitle: None,
            body: "Welcome to Slick Sheet Studio!".to_string(),
            image_url: None,
            metadata: HashMap::new(),
        }
    }
}

impl Content {
    /// Create a new Content with default values
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert content to Typst markup with cmd:// edit links
    pub fn to_typst(&self) -> String {
        let mut parts = Vec::new();

        // Page setup
        parts.push("#set page(width: 8.5in, height: 11in, margin: 0.75in)".to_string());
        parts.push("#set text(font: \"Inter\", size: 11pt)".to_string());
        parts.push(String::new());

        // Title with edit link
        parts.push(format!(
            "= #link(\"cmd://edit/title\")[{}]",
            escape_typst(&self.title)
        ));

        // Subtitle if present
        if let Some(subtitle) = &self.subtitle {
            parts.push(format!(
                "_#link(\"cmd://edit/subtitle\")[{}]_",
                escape_typst(subtitle)
            ));
        }

        parts.push(String::new());

        // Body with edit link
        parts.push(format!(
            "#link(\"cmd://edit/body\")[{}]",
            escape_typst(&self.body)
        ));

        // Image if present
        if let Some(image_url) = &self.image_url {
            parts.push(String::new());
            parts.push(format!(
                "#link(\"cmd://edit/image\")[#image(\"{}\", width: 100%)]",
                escape_typst(image_url)
            ));
        }

        // Metadata
        if !self.metadata.is_empty() {
            parts.push(String::new());
            parts.push("== Details".to_string());
            for (key, value) in &self.metadata {
                parts.push(format!(
                    "- *{}*: #link(\"cmd://edit/meta/{}\")[{}]",
                    escape_typst(key),
                    escape_typst(key),
                    escape_typst(value)
                ));
            }
        }

        parts.join("\n")
    }
}

/// Escape special Typst characters in text
fn escape_typst(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('#', "\\#")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('"', "\\\"")
}
