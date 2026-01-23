//! AI Image Generation using OpenRouter's Gemini model
//!
//! This module provides image generation capabilities using the
//! `google/gemini-3-pro-image-preview` model via OpenRouter API.

use serde::{Deserialize, Serialize};

use super::client::{ChatMessage, OpenRouterConfig, Role};

/// Model for image generation
pub const IMAGE_MODEL: &str = "google/gemini-3-pro-image-preview";

/// Request body for image generation (with modalities)
#[derive(Debug, Serialize)]
struct ImageGenRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    modalities: Vec<&'a str>,
}

/// Response from image generation
#[derive(Debug, Deserialize)]
struct ImageGenResponse {
    choices: Option<Vec<ImageChoice>>,
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
struct ImageChoice {
    message: ImageMessage,
}

#[derive(Debug, Deserialize)]
struct ImageMessage {
    #[allow(dead_code)]
    content: serde_json::Value,
    /// Images array - the actual location of generated images
    #[serde(default)]
    images: Option<Vec<ImageItem>>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}

/// Image item in the images array
#[derive(Debug, Deserialize)]
struct ImageItem {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    item_type: String,
    image_url: ImageUrl,
}

/// Image URL container
#[derive(Debug, Deserialize)]
struct ImageUrl {
    /// Data URL like "data:image/png;base64,..."
    url: String,
}

/// Image generator using OpenRouter API
pub struct ImageGenerator {
    config: OpenRouterConfig,
}

impl ImageGenerator {
    /// Create a new image generator with the given configuration
    pub fn new(config: OpenRouterConfig) -> Self {
        Self { config }
    }

    /// Build the request body JSON for image generation
    fn build_request_body(&self, prompt: &str) -> String {
        let messages = vec![ChatMessage {
            role: Role::User,
            content: prompt.to_string(),
        }];

        let request = ImageGenRequest {
            model: IMAGE_MODEL,
            messages: &messages,
            modalities: vec!["image", "text"],
        };

        serde_json::to_string(&request).unwrap_or_default()
    }

    /// Parse the API response and extract image data
    fn parse_response(response: &str) -> Result<(Vec<u8>, String), String> {
        let parsed: ImageGenResponse = serde_json::from_str(response)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Check for API error
        if let Some(error) = parsed.error {
            return Err(error.message);
        }

        // Extract content from choices
        let choices = parsed
            .choices
            .ok_or_else(|| "No choices in response".to_string())?;

        if choices.is_empty() {
            return Err("Empty choices array".to_string());
        }

        let message = &choices[0].message;

        // Format 1: Images array (OpenRouter/Gemini format)
        // Response has: message.images[].image_url.url = "data:image/png;base64,..."
        if let Some(images) = &message.images {
            if let Some(first_image) = images.first() {
                let data_url = &first_image.image_url.url;
                if let Some(image_data) = extract_data_url(data_url) {
                    return Ok(image_data);
                }
            }
        }

        // Format 2: String content with data URL (fallback)
        if let Some(text) = message.content.as_str() {
            if let Some(image_data) = extract_data_url(text) {
                return Ok(image_data);
            }
        }

        Err(
            "No image found in response. Check that the model supports image generation."
                .to_string(),
        )
    }

    /// Generate an image from a text prompt
    ///
    /// Returns the image bytes and MIME type
    #[cfg(target_arch = "wasm32")]
    pub async fn generate(&self, prompt: &str) -> Result<(Vec<u8>, String), String> {
        use gloo_net::http::Request;
        use wasm_bindgen::JsValue;

        let body = self.build_request_body(prompt);
        let url = format!("{}/chat/completions", self.config.base_url);

        // Log for debugging
        web_sys::console::log_1(&JsValue::from_str(&format!(
            "Image generation request: url={}, model={}",
            url, IMAGE_MODEL
        )));

        // Build request with headers and body
        let response = Request::post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", self.config.api_key))
            .header("HTTP-Referer", &self.config.http_referer)
            .header("X-Title", &self.config.x_title)
            .body(body)
            .map_err(|e| format!("Failed to build request: {:?}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {:?}", e))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {:?}", e))?;

        // Log response for debugging
        web_sys::console::log_1(&JsValue::from_str(&format!(
            "Image generation response (status {}): {}",
            status,
            if text.len() > 500 {
                format!("{}...", &text[..500])
            } else {
                text.clone()
            }
        )));

        // Handle authentication errors
        if status == 401 {
            return Err(format!(
                "Authentication failed (401). Your API key may be invalid. \
                Expected format: sk-or-v1-xxxxx (get one at openrouter.ai/keys). \
                Server message: {}",
                text
            ));
        }

        Self::parse_response(&text)
    }

    /// Generate an image from a text prompt (native builds)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn generate(&self, prompt: &str) -> Result<(Vec<u8>, String), String> {
        let body = self.build_request_body(prompt);
        let url = format!("{}/chat/completions", self.config.base_url);

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("HTTP-Referer", &self.config.http_referer)
            .header("X-Title", &self.config.x_title)
            .body(body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        Self::parse_response(&text)
    }
}

/// Decode base64 string to bytes
fn decode_base64(encoded: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| format!("Base64 decode error: {}", e))
}

/// Model for alt description generation (fast, cheap model)
pub const ALT_DESCRIPTION_MODEL: &str = "google/gemini-2.0-flash-001";

/// Generate a short alt description for an image based on its generation prompt
///
/// Uses Gemini Flash for fast, cost-effective text generation
#[cfg(target_arch = "wasm32")]
pub async fn generate_alt_description(
    config: &OpenRouterConfig,
    prompt: &str,
) -> Result<String, String> {
    use gloo_net::http::Request;

    let system_prompt =
        "Generate a short alt description (10-15 words max) for an AI-generated image. \
        The description should be concise and describe what the image shows. \
        Output ONLY the description text, nothing else.";

    let user_prompt = format!("The image was generated with this prompt: {}", prompt);

    let messages = vec![
        ChatMessage {
            role: Role::System,
            content: system_prompt.to_string(),
        },
        ChatMessage {
            role: Role::User,
            content: user_prompt,
        },
    ];

    #[derive(serde::Serialize)]
    struct AltDescRequest<'a> {
        model: &'a str,
        messages: &'a [ChatMessage],
    }

    let request = AltDescRequest {
        model: ALT_DESCRIPTION_MODEL,
        messages: &messages,
    };

    let body = serde_json::to_string(&request).map_err(|e| format!("Serialize error: {}", e))?;
    let url = format!("{}/chat/completions", config.base_url);

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", config.api_key))
        .header("HTTP-Referer", &config.http_referer)
        .header("X-Title", &config.x_title)
        .body(body)
        .map_err(|e| format!("Failed to build request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {:?}", e))?;

    // Parse response to extract content
    #[derive(serde::Deserialize)]
    struct AltDescResponse {
        choices: Option<Vec<AltDescChoice>>,
    }

    #[derive(serde::Deserialize)]
    struct AltDescChoice {
        message: AltDescMessage,
    }

    #[derive(serde::Deserialize)]
    struct AltDescMessage {
        content: String,
    }

    let parsed: AltDescResponse =
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse response: {}", e))?;

    let choices = parsed
        .choices
        .ok_or_else(|| "No choices in response".to_string())?;

    if choices.is_empty() {
        return Err("Empty choices array".to_string());
    }

    // Clean up the response (remove quotes, trim whitespace)
    let alt = choices[0].message.content.trim();
    let alt = alt.trim_matches('"');
    Ok(alt.to_string())
}

/// Generate a short alt description for an image based on its generation prompt (native builds)
#[cfg(not(target_arch = "wasm32"))]
pub async fn generate_alt_description(
    config: &OpenRouterConfig,
    prompt: &str,
) -> Result<String, String> {
    let system_prompt =
        "Generate a short alt description (10-15 words max) for an AI-generated image. \
        The description should be concise and describe what the image shows. \
        Output ONLY the description text, nothing else.";

    let user_prompt = format!("The image was generated with this prompt: {}", prompt);

    let messages = vec![
        ChatMessage {
            role: Role::System,
            content: system_prompt.to_string(),
        },
        ChatMessage {
            role: Role::User,
            content: user_prompt,
        },
    ];

    #[derive(serde::Serialize)]
    struct AltDescRequest<'a> {
        model: &'a str,
        messages: &'a [ChatMessage],
    }

    let request = AltDescRequest {
        model: ALT_DESCRIPTION_MODEL,
        messages: &messages,
    };

    let body = serde_json::to_string(&request).map_err(|e| format!("Serialize error: {}", e))?;
    let url = format!("{}/chat/completions", config.base_url);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("HTTP-Referer", &config.http_referer)
        .header("X-Title", &config.x_title)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Parse response to extract content
    #[derive(serde::Deserialize)]
    struct AltDescResponse {
        choices: Option<Vec<AltDescChoice>>,
    }

    #[derive(serde::Deserialize)]
    struct AltDescChoice {
        message: AltDescMessage,
    }

    #[derive(serde::Deserialize)]
    struct AltDescMessage {
        content: String,
    }

    let parsed: AltDescResponse =
        serde_json::from_str(&text).map_err(|e| format!("Failed to parse response: {}", e))?;

    let choices = parsed
        .choices
        .ok_or_else(|| "No choices in response".to_string())?;

    if choices.is_empty() {
        return Err("Empty choices array".to_string());
    }

    // Clean up the response (remove quotes, trim whitespace)
    let alt = choices[0].message.content.trim();
    let alt = alt.trim_matches('"');
    Ok(alt.to_string())
}

/// Extract image data from a data URL string
/// Handles formats like: "![image](data:image/png;base64,...)" or "data:image/png;base64,..."
fn extract_data_url(text: &str) -> Option<(Vec<u8>, String)> {
    // Look for data URL pattern
    let data_url_start = text.find("data:")?;
    let after_data = &text[data_url_start + 5..];

    // Extract MIME type (between "data:" and ";base64,")
    let semicolon = after_data.find(';')?;
    let mime_type = after_data[..semicolon].to_string();

    // Find base64 data
    let base64_marker = "base64,";
    let base64_start = after_data.find(base64_marker)? + base64_marker.len();
    let after_base64 = &after_data[base64_start..];

    // Find end of base64 data (could end with ), ", or whitespace)
    let end = after_base64
        .find(|c: char| c == ')' || c == '"' || c == '\'' || c.is_whitespace())
        .unwrap_or(after_base64.len());

    let base64_data = &after_base64[..end];

    // Decode base64
    let bytes = decode_base64(base64_data).ok()?;

    Some((bytes, mime_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_data_url_markdown() {
        let text = "Here's your image: ![image](data:image/png;base64,iVBORw0KGgo=)";
        let result = extract_data_url(text);
        assert!(result.is_some());
        let (bytes, mime) = result.unwrap();
        assert_eq!(mime, "image/png");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_extract_data_url_raw() {
        let text = "data:image/jpeg;base64,/9j/4AAQSkZJRg==";
        let result = extract_data_url(text);
        assert!(result.is_some());
        let (bytes, mime) = result.unwrap();
        assert_eq!(mime, "image/jpeg");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_extract_data_url_no_match() {
        let text = "Just some regular text without images";
        let result = extract_data_url(text);
        assert!(result.is_none());
    }

    #[test]
    fn test_build_request_body() {
        let config = OpenRouterConfig::with_key("test-key".to_string());
        let generator = ImageGenerator::new(config);
        let body = generator.build_request_body("A red apple");

        assert!(body.contains("google/gemini-3-pro-image-preview"));
        assert!(body.contains("A red apple"));
        assert!(body.contains("modalities"));
        assert!(body.contains("image"));
        assert!(body.contains("text"));
    }

    #[test]
    fn test_parse_response_error() {
        let response = r#"{"error": {"message": "Rate limit exceeded"}}"#;
        let result = ImageGenerator::parse_response(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Rate limit"));
    }

    #[test]
    fn test_parse_response_no_choices() {
        let response = r#"{"choices": null}"#;
        let result = ImageGenerator::parse_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_response_empty_choices() {
        let response = r#"{"choices": []}"#;
        let result = ImageGenerator::parse_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_response_with_images_array() {
        // This is the actual format returned by OpenRouter/Gemini
        let response = r#"{
            "choices": [{
                "message": {
                    "content": "",
                    "images": [{
                        "type": "image_url",
                        "image_url": {
                            "url": "data:image/png;base64,iVBORw0KGgo="
                        }
                    }]
                }
            }]
        }"#;
        let result = ImageGenerator::parse_response(response);
        assert!(result.is_ok());
        let (bytes, mime) = result.unwrap();
        assert_eq!(mime, "image/png");
        assert!(!bytes.is_empty());
    }
}
