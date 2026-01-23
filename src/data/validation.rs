//! Validation logic for slick sheet data

use super::schema::{SectionType, SlickSheetData};
use thiserror::Error;

/// Validation error types
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ValidationError {
    /// Title is required
    #[error("Title is required and cannot be empty")]
    EmptyTitle,

    /// Section has no heading
    #[error("Section at index {0} is missing a heading")]
    EmptySectionHeading(usize),

    /// List section has no items
    #[error("List section '{0}' has no items")]
    EmptyListItems(String),

    /// Table section has no rows
    #[error("Table section '{0}' has no rows")]
    EmptyTableRows(String),

    /// Table section missing column count
    #[error("Table section '{0}' is missing column count")]
    MissingTableColumns(String),

    /// Stat has empty value
    #[error("Stat at index {0} has empty value")]
    EmptyStatValue(usize),

    /// Stat has empty label
    #[error("Stat at index {0} has empty label")]
    EmptyStatLabel(usize),

    /// Invalid color format
    #[error("Invalid color format: '{0}' (expected hex color like #ffffff)")]
    InvalidColorFormat(String),
}

/// Validate a SlickSheetData instance
///
/// Returns a list of validation errors, empty if valid
pub fn validate_schema(data: &SlickSheetData) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Title is required
    if data.title.trim().is_empty() {
        errors.push(ValidationError::EmptyTitle);
    }

    // Validate sections
    for (i, section) in data.sections.iter().enumerate() {
        if section.heading.trim().is_empty() {
            errors.push(ValidationError::EmptySectionHeading(i));
        }

        match section.section_type {
            SectionType::List => {
                if section.items.as_ref().is_none_or(|items| items.is_empty()) {
                    errors.push(ValidationError::EmptyListItems(section.heading.clone()));
                }
            }
            SectionType::Table => {
                if section.rows.as_ref().is_none_or(|rows| rows.is_empty()) {
                    errors.push(ValidationError::EmptyTableRows(section.heading.clone()));
                }
                if section.columns.is_none() {
                    errors.push(ValidationError::MissingTableColumns(
                        section.heading.clone(),
                    ));
                }
            }
            SectionType::Text | SectionType::Quote => {
                // Text and quote sections don't have additional requirements
            }
        }
    }

    // Validate stats
    for (i, stat) in data.stats.iter().enumerate() {
        if stat.value.trim().is_empty() {
            errors.push(ValidationError::EmptyStatValue(i));
        }
        if stat.label.trim().is_empty() {
            errors.push(ValidationError::EmptyStatLabel(i));
        }
        if let Some(color) = &stat.color {
            if !is_valid_hex_color(color) {
                errors.push(ValidationError::InvalidColorFormat(color.clone()));
            }
        }
    }

    // Validate style colors
    if let Some(style) = &data.style {
        if let Some(color) = &style.primary_color {
            if !is_valid_hex_color(color) {
                errors.push(ValidationError::InvalidColorFormat(color.clone()));
            }
        }
        if let Some(color) = &style.accent_color {
            if !is_valid_hex_color(color) {
                errors.push(ValidationError::InvalidColorFormat(color.clone()));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check if a string is a valid hex color (e.g., #ffffff or #fff)
fn is_valid_hex_color(color: &str) -> bool {
    if !color.starts_with('#') {
        return false;
    }

    let hex_part = &color[1..];
    let valid_length = hex_part.len() == 3 || hex_part.len() == 6;
    let all_hex = hex_part.chars().all(|c| c.is_ascii_hexdigit());

    valid_length && all_hex
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use crate::data::schema::{Section, Stat, StyleHints};

    #[test]
    fn test_valid_data() {
        let data = SlickSheetData {
            title: "Test Title".to_string(),
            ..Default::default()
        };

        assert!(validate_schema(&data).is_ok());
    }

    #[test]
    fn test_empty_title() {
        let data = SlickSheetData {
            title: "".to_string(),
            ..Default::default()
        };

        let result = validate_schema(&data);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains(&ValidationError::EmptyTitle));
    }

    #[test]
    fn test_empty_section_heading() {
        let data = SlickSheetData {
            title: "Test".to_string(),
            sections: vec![Section {
                heading: "".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = validate_schema(&data);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains(&ValidationError::EmptySectionHeading(0)));
    }

    #[test]
    fn test_empty_list_items() {
        let data = SlickSheetData {
            title: "Test".to_string(),
            sections: vec![Section {
                heading: "Features".to_string(),
                section_type: SectionType::List,
                items: Some(vec![]),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = validate_schema(&data);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains(&ValidationError::EmptyListItems("Features".to_string())));
    }

    #[test]
    fn test_empty_stat_value() {
        let data = SlickSheetData {
            title: "Test".to_string(),
            stats: vec![Stat {
                value: "".to_string(),
                label: "Test".to_string(),
                color: None,
            }],
            ..Default::default()
        };

        let result = validate_schema(&data);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains(&ValidationError::EmptyStatValue(0)));
    }

    #[test]
    fn test_invalid_color_format() {
        let data = SlickSheetData {
            title: "Test".to_string(),
            style: Some(StyleHints {
                primary_color: Some("invalid".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = validate_schema(&data);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains(&ValidationError::InvalidColorFormat("invalid".to_string())));
    }

    #[test]
    fn test_valid_hex_colors() {
        assert!(is_valid_hex_color("#fff"));
        assert!(is_valid_hex_color("#ffffff"));
        assert!(is_valid_hex_color("#e94560"));
        assert!(is_valid_hex_color("#ABC"));
        assert!(is_valid_hex_color("#abcdef"));
    }

    #[test]
    fn test_invalid_hex_colors() {
        assert!(!is_valid_hex_color("fff"));
        assert!(!is_valid_hex_color("#ff"));
        assert!(!is_valid_hex_color("#fffffff"));
        assert!(!is_valid_hex_color("#gggggg"));
        assert!(!is_valid_hex_color(""));
    }
}
