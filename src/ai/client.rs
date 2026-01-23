//! OpenRouter API client for LLM interactions

use serde::{Deserialize, Serialize};

/// Message roles in a chat conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A chat message with role and content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl ChatMessage {
    /// Create a new chat message
    pub fn new(role: Role, content: String) -> Self {
        Self { role, content }
    }

    /// Create a system message
    pub fn system(content: String) -> Self {
        Self::new(Role::System, content)
    }

    /// Create a user message
    pub fn user(content: String) -> Self {
        Self::new(Role::User, content)
    }

    /// Create an assistant message (public API)
    #[allow(dead_code)]
    pub fn assistant(content: String) -> Self {
        Self::new(Role::Assistant, content)
    }
}

/// Configuration for the OpenRouter client
#[derive(Debug, Clone)]
pub struct OpenRouterConfig {
    /// API key for authentication
    pub api_key: String,
    /// Base URL for the API
    pub base_url: String,
    /// HTTP referer for OpenRouter attribution
    pub http_referer: String,
    /// App title for OpenRouter attribution
    pub x_title: String,
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            http_referer: "https://slicksheetstudio.app".to_string(),
            x_title: "Slick Sheet Studio".to_string(),
        }
    }
}

impl OpenRouterConfig {
    /// Create config with an API key
    pub fn with_key(api_key: String) -> Self {
        Self {
            api_key,
            ..Default::default()
        }
    }
}

/// OpenRouter API client
#[derive(Debug, Clone)]
pub struct OpenRouterClient {
    config: OpenRouterConfig,
}

/// Request body for chat completions
#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
}

/// Response from chat completions
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Option<Vec<Choice>>,
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}

impl OpenRouterClient {
    /// Create a new OpenRouter client
    pub fn new(config: OpenRouterConfig) -> Self {
        Self { config }
    }

    /// Get the client configuration (public API)
    #[allow(dead_code)]
    pub fn config(&self) -> &OpenRouterConfig {
        &self.config
    }

    /// Build the request body JSON for a chat completion
    pub fn build_request_body(&self, model: &str, messages: &[ChatMessage]) -> String {
        let request = ChatCompletionRequest { model, messages };
        serde_json::to_string(&request).unwrap_or_default()
    }

    /// Parse the API response and extract the content
    pub fn parse_response(response: &str) -> Result<String, String> {
        let parsed: ChatCompletionResponse = serde_json::from_str(response)
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

        Ok(choices[0].message.content.clone())
    }

    /// Send a chat completion request (async, for WASM)
    #[cfg(target_arch = "wasm32")]
    pub async fn chat(&self, model: &str, messages: Vec<ChatMessage>) -> Result<String, String> {
        use gloo_net::http::Request;
        use wasm_bindgen::JsValue;

        let body = self.build_request_body(model, &messages);
        let url = format!("{}/chat/completions", self.config.base_url);

        // Log for debugging - show key prefix to help identify issues
        let key_preview = if self.config.api_key.len() > 10 {
            format!(
                "{}...{}",
                &self.config.api_key[..8],
                &self.config.api_key[self.config.api_key.len() - 4..]
            )
        } else {
            "[too short]".to_string()
        };
        web_sys::console::log_1(&JsValue::from_str(&format!(
            "OpenRouter request: url={}, model={}, key={} (len={})",
            url,
            model,
            key_preview,
            self.config.api_key.len()
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
            "Response (status {}): {}",
            status,
            if text.len() > 300 {
                &text[..300]
            } else {
                &text
            }
        )));

        // If we got a 401, provide more helpful error
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

    /// Send a chat completion request (async, for native builds)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn chat(&self, model: &str, messages: Vec<ChatMessage>) -> Result<String, String> {
        let body = self.build_request_body(model, &messages);
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
