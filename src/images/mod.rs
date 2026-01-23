//! Image support module for Slick Sheet Studio
//!
//! This module provides:
//! - Image metadata and format validation
//! - IndexedDB-based image storage
//! - Image cache for synchronous access in VirtualWorld

mod loader;
mod store;

pub use loader::ImageCache;
pub use store::ImageStore;

use serde::{Deserialize, Serialize};

/// Maximum allowed image size in bytes (10 MB)
pub const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024;

/// Supported image formats
pub const SUPPORTED_FORMATS: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/svg+xml",
    "image/gif",
    "image/webp",
];

/// Supported file extensions
pub const SUPPORTED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "svg", "gif", "webp"];

/// Metadata for a stored image
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageMetadata {
    /// Unique identifier (e.g., "img_a1b2c3d4")
    pub id: String,
    /// Original filename
    pub filename: String,
    /// MIME type (e.g., "image/png")
    pub mime_type: String,
    /// Size in bytes
    pub size: usize,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Full prompt used to generate the image (None for uploaded images)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_prompt: Option<String>,
    /// Short AI-generated description of the image content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt_description: Option<String>,
}

impl ImageMetadata {
    /// Create new image metadata
    pub fn new(id: String, filename: String, mime_type: String, size: usize) -> Self {
        let created_at = js_sys::Date::new_0()
            .to_iso_string()
            .as_string()
            .unwrap_or_else(|| "unknown".to_string());

        Self {
            id,
            filename,
            mime_type,
            size,
            created_at,
            generation_prompt: None,
            alt_description: None,
        }
    }

    /// Create new image metadata for a generated image
    pub fn new_generated(
        id: String,
        filename: String,
        mime_type: String,
        size: usize,
        generation_prompt: String,
        alt_description: String,
    ) -> Self {
        let created_at = js_sys::Date::new_0()
            .to_iso_string()
            .as_string()
            .unwrap_or_else(|| "unknown".to_string());

        Self {
            id,
            filename,
            mime_type,
            size,
            created_at,
            generation_prompt: Some(generation_prompt),
            alt_description: Some(alt_description),
        }
    }
}

/// Check if a MIME type is supported
pub fn is_supported_mime_type(mime_type: &str) -> bool {
    SUPPORTED_FORMATS.contains(&mime_type)
}

/// Check if a file extension is supported
pub fn is_supported_extension(extension: &str) -> bool {
    SUPPORTED_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}

/// Detect MIME type from file bytes using magic numbers
pub fn detect_mime_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() < 12 {
        return None;
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some("image/png");
    }

    // JPEG: FF D8 FF
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("image/jpeg");
    }

    // GIF: 47 49 46 38 (GIF8)
    if bytes.starts_with(&[0x47, 0x49, 0x46, 0x38]) {
        return Some("image/gif");
    }

    // WebP: 52 49 46 46 ... 57 45 42 50 (RIFF....WEBP)
    if bytes.len() >= 12 && bytes.starts_with(&[0x52, 0x49, 0x46, 0x46]) && &bytes[8..12] == b"WEBP"
    {
        return Some("image/webp");
    }

    // SVG: Check for XML/SVG header
    if let Ok(text) = std::str::from_utf8(&bytes[..bytes.len().min(1024)]) {
        let text_lower = text.to_lowercase();
        if text_lower.contains("<svg")
            || text_lower.contains("<?xml") && text_lower.contains("<svg")
        {
            return Some("image/svg+xml");
        }
    }

    None
}

/// Get file extension from MIME type
pub fn extension_from_mime_type(mime_type: &str) -> &'static str {
    match mime_type {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        _ => "bin",
    }
}

/// Generate a unique image ID
pub fn generate_image_id() -> String {
    // Use crypto.getRandomValues for better randomness in browser
    let random_part = match web_sys::window() {
        Some(window) => match window.crypto() {
            Ok(crypto) => {
                let mut array = [0u8; 8];
                if crypto.get_random_values_with_u8_array(&mut array).is_ok() {
                    hex_encode(&array)
                } else {
                    fallback_random_id()
                }
            }
            Err(_) => fallback_random_id(),
        },
        None => fallback_random_id(),
    };

    format!("img_{}", random_part)
}

/// Fallback random ID generation using JS Math.random
fn fallback_random_id() -> String {
    let random = js_sys::Math::random();
    let id = (random * 1_000_000_000_000.0) as u64;
    format!("{:012x}", id)
}

/// Simple hex encoding
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Errors that can occur during image operations
#[derive(Debug, Clone)]
pub enum ImageError {
    /// File is too large
    FileTooLarge(usize),
    /// Unsupported format
    UnsupportedFormat(String),
    /// Storage error
    StorageError(String),
    /// Image not found
    NotFound(String),
    /// Invalid data
    InvalidData(String),
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::FileTooLarge(size) => {
                write!(
                    f,
                    "File too large: {} bytes (max: {} bytes)",
                    size, MAX_IMAGE_SIZE
                )
            }
            ImageError::UnsupportedFormat(fmt) => {
                write!(f, "Unsupported image format: {}", fmt)
            }
            ImageError::StorageError(msg) => {
                write!(f, "Storage error: {}", msg)
            }
            ImageError::NotFound(id) => {
                write!(f, "Image not found: {}", id)
            }
            ImageError::InvalidData(msg) => {
                write!(f, "Invalid image data: {}", msg)
            }
        }
    }
}

impl std::error::Error for ImageError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_mime_type() {
        assert!(is_supported_mime_type("image/png"));
        assert!(is_supported_mime_type("image/jpeg"));
        assert!(is_supported_mime_type("image/svg+xml"));
        assert!(is_supported_mime_type("image/gif"));
        assert!(is_supported_mime_type("image/webp"));
        assert!(!is_supported_mime_type("image/tiff"));
        assert!(!is_supported_mime_type("application/pdf"));
    }

    #[test]
    fn test_is_supported_extension() {
        assert!(is_supported_extension("png"));
        assert!(is_supported_extension("PNG"));
        assert!(is_supported_extension("jpg"));
        assert!(is_supported_extension("jpeg"));
        assert!(is_supported_extension("svg"));
        assert!(is_supported_extension("gif"));
        assert!(is_supported_extension("webp"));
        assert!(!is_supported_extension("tiff"));
        assert!(!is_supported_extension("bmp"));
    }

    #[test]
    fn test_detect_mime_type_png() {
        let png_header = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(detect_mime_type(&png_header), Some("image/png"));
    }

    #[test]
    fn test_detect_mime_type_jpeg() {
        let jpeg_header = [
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
        ];
        assert_eq!(detect_mime_type(&jpeg_header), Some("image/jpeg"));
    }

    #[test]
    fn test_detect_mime_type_gif() {
        let gif_header = [
            0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(detect_mime_type(&gif_header), Some("image/gif"));
    }

    #[test]
    fn test_detect_mime_type_webp() {
        let webp_header = [
            0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50,
        ];
        assert_eq!(detect_mime_type(&webp_header), Some("image/webp"));
    }

    #[test]
    fn test_detect_mime_type_svg() {
        let svg_data = b"<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";
        assert_eq!(detect_mime_type(svg_data), Some("image/svg+xml"));
    }

    #[test]
    fn test_detect_mime_type_unknown() {
        let unknown_data = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
        ];
        assert_eq!(detect_mime_type(&unknown_data), None);
    }

    #[test]
    fn test_extension_from_mime_type() {
        assert_eq!(extension_from_mime_type("image/png"), "png");
        assert_eq!(extension_from_mime_type("image/jpeg"), "jpg");
        assert_eq!(extension_from_mime_type("image/gif"), "gif");
        assert_eq!(extension_from_mime_type("image/webp"), "webp");
        assert_eq!(extension_from_mime_type("image/svg+xml"), "svg");
        assert_eq!(extension_from_mime_type("unknown"), "bin");
    }

    #[test]
    fn test_image_error_display() {
        let err = ImageError::FileTooLarge(20_000_000);
        assert!(err.to_string().contains("20000000"));
        assert!(err.to_string().contains("too large"));

        let err = ImageError::UnsupportedFormat("image/tiff".to_string());
        assert!(err.to_string().contains("tiff"));

        let err = ImageError::NotFound("img_123".to_string());
        assert!(err.to_string().contains("img_123"));
    }
}
