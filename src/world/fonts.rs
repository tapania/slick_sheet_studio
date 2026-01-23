//! Font loading for the VirtualWorld
//!
//! This module handles embedded fonts and font book creation for Typst.

use typst::text::{Font, FontBook};

/// Embedded font data
const FONT_DATA: &[&[u8]] = &[
    include_bytes!("../../assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../../assets/fonts/Inter-Bold.ttf"),
    include_bytes!("../../assets/fonts/Inter-Italic.ttf"),
    include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf"),
];

/// Font loader that manages embedded fonts
pub struct FontLoader {
    fonts: Vec<Font>,
}

impl FontLoader {
    /// Create a new font loader with embedded fonts
    pub fn new() -> Self {
        let fonts = FONT_DATA
            .iter()
            .filter_map(|data| Font::new((*data).into(), 0))
            .collect();
        Self { fonts }
    }

    /// Get all loaded fonts
    #[allow(dead_code)]
    pub fn fonts(&self) -> &[Font] {
        &self.fonts
    }

    /// Get a font by index
    pub fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    /// Build a FontBook from loaded fonts
    pub fn font_book(&self) -> FontBook {
        let mut book = FontBook::new();
        for font in &self.fonts {
            book.push(font.info().clone());
        }
        book
    }
}

impl Default for FontLoader {
    fn default() -> Self {
        Self::new()
    }
}
