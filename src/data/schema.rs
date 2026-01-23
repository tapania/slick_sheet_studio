//! Schema definitions for slick sheet data
//!
//! This module provides data structures for slick sheet content.
//! Some builder methods and accessors are provided for public API
//! but not yet used internally.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main data model for a slick sheet document
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SlickSheetData {
    /// Main title of the document
    pub title: String,

    /// Optional subtitle or tagline
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,

    /// Main body text
    #[serde(default)]
    pub body: String,

    /// Structured content sections
    #[serde(default)]
    pub sections: Vec<Section>,

    /// Key-value metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Feature list items
    #[serde(default)]
    pub features: Vec<String>,

    /// Statistics/metrics to display
    #[serde(default)]
    pub stats: Vec<Stat>,

    /// Contact information
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact: Option<ContactInfo>,

    /// Styling hints for the template
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleHints>,

    /// Image references: semantic name -> image ID
    /// Example: { "logo": "img_abc123", "banner": "img_def456" }
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub images: HashMap<String, String>,
}

impl SlickSheetData {
    /// Create a new SlickSheetData with a title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Builder method to set subtitle
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Builder method to set body
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    /// Builder method to add a section
    pub fn with_section(mut self, section: Section) -> Self {
        self.sections.push(section);
        self
    }

    /// Builder method to add a feature
    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    /// Builder method to add a stat
    pub fn with_stat(mut self, stat: Stat) -> Self {
        self.stats.push(stat);
        self
    }

    /// Builder method to set contact info
    pub fn with_contact(mut self, contact: ContactInfo) -> Self {
        self.contact = Some(contact);
        self
    }

    /// Builder method to set style hints
    pub fn with_style(mut self, style: StyleHints) -> Self {
        self.style = Some(style);
        self
    }

    /// Builder method to add an image reference
    pub fn with_image(mut self, name: impl Into<String>, image_id: impl Into<String>) -> Self {
        self.images.insert(name.into(), image_id.into());
        self
    }
}

/// A content section in the document
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Section {
    /// Section heading
    pub heading: String,

    /// Section content (for text type)
    #[serde(default)]
    pub content: String,

    /// Type of section
    #[serde(default, rename = "type")]
    pub section_type: SectionType,

    /// List items (for list type)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<String>>,

    /// Table rows (for table type)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<Vec<String>>>,

    /// Number of columns (for table type)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<usize>,
}

impl Default for Section {
    fn default() -> Self {
        Self {
            heading: String::new(),
            content: String::new(),
            section_type: SectionType::Text,
            items: None,
            rows: None,
            columns: None,
        }
    }
}

impl Section {
    /// Create a new text section
    pub fn text(heading: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            heading: heading.into(),
            content: content.into(),
            section_type: SectionType::Text,
            ..Default::default()
        }
    }

    /// Create a new list section
    pub fn list(heading: impl Into<String>, items: Vec<String>) -> Self {
        Self {
            heading: heading.into(),
            section_type: SectionType::List,
            items: Some(items),
            ..Default::default()
        }
    }

    /// Create a new table section
    pub fn table(heading: impl Into<String>, rows: Vec<Vec<String>>, columns: usize) -> Self {
        Self {
            heading: heading.into(),
            section_type: SectionType::Table,
            rows: Some(rows),
            columns: Some(columns),
            ..Default::default()
        }
    }

    /// Create a new quote section
    pub fn quote(heading: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            heading: heading.into(),
            content: content.into(),
            section_type: SectionType::Quote,
            ..Default::default()
        }
    }
}

/// Type of content section
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SectionType {
    /// Plain text content
    #[default]
    Text,
    /// Bulleted list
    List,
    /// Table data
    Table,
    /// Quote/testimonial
    Quote,
}

/// A statistic or metric to display
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stat {
    /// The value (e.g., "95%", "$1M", "2x")
    pub value: String,

    /// Label describing the stat
    pub label: String,

    /// Optional color for the value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl Stat {
    /// Create a new stat
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            color: None,
        }
    }

    /// Create a stat with a custom color
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// Contact information for the document
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ContactInfo {
    /// Email address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Phone number
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// Website URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,

    /// Physical address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

impl ContactInfo {
    /// Create contact info with email
    pub fn with_email(email: impl Into<String>) -> Self {
        Self {
            email: Some(email.into()),
            ..Default::default()
        }
    }
}

/// Style hints for template rendering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StyleHints {
    /// Primary color (hex format, e.g., "#e94560")
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "primaryColor"
    )]
    pub primary_color: Option<String>,

    /// Accent color
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "accentColor"
    )]
    pub accent_color: Option<String>,

    /// Font family name
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "fontFamily")]
    pub font_family: Option<String>,
}

impl StyleHints {
    /// Get the primary color or a default
    pub fn primary_color_or_default(&self) -> &str {
        self.primary_color.as_deref().unwrap_or("#e94560")
    }

    /// Get the accent color or a default
    pub fn accent_color_or_default(&self) -> &str {
        self.accent_color.as_deref().unwrap_or("#4ecca3")
    }

    /// Get the font family or a default
    pub fn font_family_or_default(&self) -> &str {
        self.font_family.as_deref().unwrap_or("Inter")
    }
}
