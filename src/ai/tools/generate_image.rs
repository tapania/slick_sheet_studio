//! Generate Image AI tool
//!
//! Tool for generating images using AI via OpenRouter's Gemini model.

#![allow(dead_code)]

use super::{AiTool, ToolResult};
use crate::ai::client::OpenRouterConfig;
use crate::ai::image_gen::ImageGenerator;

/// Tool for generating images from text prompts
pub struct GenerateImageTool;

impl GenerateImageTool {
    /// Execute the image generation tool
    ///
    /// Returns the image bytes and MIME type on success
    pub async fn execute(api_key: &str, prompt: &str) -> Result<(Vec<u8>, String), String> {
        // Validate inputs
        if api_key.trim().is_empty() {
            return Err("API key is required for image generation".to_string());
        }

        if prompt.trim().is_empty() {
            return Err("Image prompt cannot be empty".to_string());
        }

        let config = OpenRouterConfig::with_key(api_key.to_string());
        let generator = ImageGenerator::new(config);
        generator.generate(prompt).await
    }

    /// Execute and return a ToolResult
    pub async fn execute_as_result(api_key: &str, prompt: &str) -> ToolResult {
        match Self::execute(api_key, prompt).await {
            Ok((bytes, mime_type)) => ToolResult::Success(format!(
                "Generated image: {} bytes, type: {}",
                bytes.len(),
                mime_type
            )),
            Err(e) => ToolResult::Error(e),
        }
    }
}

impl AiTool for GenerateImageTool {
    fn name(&self) -> &'static str {
        "generate_image"
    }

    fn description(&self) -> &'static str {
        "Generate an image using AI from a text description. Takes a prompt describing the desired image and returns the generated image data."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_name() {
        let tool = GenerateImageTool;
        assert_eq!(tool.name(), "generate_image");
    }

    #[test]
    fn test_tool_description() {
        let tool = GenerateImageTool;
        assert!(tool.description().contains("Generate an image"));
        assert!(tool.description().contains("text description"));
    }
}
