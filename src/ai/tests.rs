//! Tests for the AI module

use super::agent::{AgentConfig, AgentLoop, AgentState};
use super::client::{ChatMessage, OpenRouterClient, OpenRouterConfig, Role};
use super::prompts::{generate_system_prompt, generate_user_prompt, PromptTemplate};
use super::verify::{verify_change, VerificationResult};

// ============================================================================
// OpenRouter Client Tests
// ============================================================================

#[test]
fn test_openrouter_config_default() {
    let config = OpenRouterConfig::default();
    assert_eq!(config.base_url, "https://openrouter.ai/api/v1");
    assert!(config.api_key.is_empty());
}

#[test]
fn test_openrouter_config_with_key() {
    let config = OpenRouterConfig::with_key("test-key-123".to_string());
    assert_eq!(config.api_key, "test-key-123");
    assert_eq!(config.base_url, "https://openrouter.ai/api/v1");
}

#[test]
fn test_chat_message_creation() {
    let msg = ChatMessage::new(Role::User, "Hello".to_string());
    assert_eq!(msg.role, Role::User);
    assert_eq!(msg.content, "Hello");
}

#[test]
fn test_chat_message_system() {
    let msg = ChatMessage::system("You are a helpful assistant.".to_string());
    assert_eq!(msg.role, Role::System);
    assert_eq!(msg.content, "You are a helpful assistant.");
}

#[test]
fn test_chat_message_user() {
    let msg = ChatMessage::user("Generate a title.".to_string());
    assert_eq!(msg.role, Role::User);
    assert_eq!(msg.content, "Generate a title.");
}

#[test]
fn test_chat_message_assistant() {
    let msg = ChatMessage::assistant("Here is the title.".to_string());
    assert_eq!(msg.role, Role::Assistant);
    assert_eq!(msg.content, "Here is the title.");
}

#[test]
fn test_openrouter_client_creation() {
    let config = OpenRouterConfig::with_key("test-key".to_string());
    let client = OpenRouterClient::new(config);
    assert!(!client.config().api_key.is_empty());
}

#[test]
fn test_openrouter_client_build_request_body() {
    let config = OpenRouterConfig::with_key("test-key".to_string());
    let client = OpenRouterClient::new(config);

    let messages = vec![
        ChatMessage::system("You are a Typst expert.".to_string()),
        ChatMessage::user("Generate a title.".to_string()),
    ];

    let body = client.build_request_body("google/gemini-3-flash-preview", &messages);

    assert!(body.contains("google/gemini-3-flash-preview"));
    assert!(body.contains("You are a Typst expert."));
    assert!(body.contains("Generate a title."));
}

#[test]
fn test_openrouter_client_parse_response_valid() {
    let response = r#"{
        "choices": [{
            "message": {
                "content": "= Hello World"
            }
        }]
    }"#;

    let result = OpenRouterClient::parse_response(response);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "= Hello World");
}

#[test]
fn test_openrouter_client_parse_response_empty_choices() {
    let response = r#"{"choices": []}"#;

    let result = OpenRouterClient::parse_response(response);
    assert!(result.is_err());
}

#[test]
fn test_openrouter_client_parse_response_invalid_json() {
    let response = "not valid json";

    let result = OpenRouterClient::parse_response(response);
    assert!(result.is_err());
}

#[test]
fn test_openrouter_client_parse_response_error_field() {
    let response = r#"{"error": {"message": "Invalid API key"}}"#;

    let result = OpenRouterClient::parse_response(response);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid API key"));
}

// ============================================================================
// Prompt Template Tests
// ============================================================================

#[test]
fn test_prompt_template_typst_generation() {
    let template = PromptTemplate::TypstGeneration;
    let prompt = template.system_prompt();

    assert!(prompt.contains("Typst"));
    assert!(prompt.contains("markup"));
}

#[test]
fn test_prompt_template_error_recovery() {
    let template = PromptTemplate::ErrorRecovery;
    let prompt = template.system_prompt();

    assert!(prompt.contains("error"));
    assert!(prompt.contains("fix"));
}

#[test]
fn test_prompt_template_visual_verification() {
    let template = PromptTemplate::VisualVerification;
    let prompt = template.system_prompt();

    assert!(prompt.contains("visual") || prompt.contains("image") || prompt.contains("verify"));
}

#[test]
fn test_generate_system_prompt_includes_context() {
    let prompt = generate_system_prompt(PromptTemplate::TypstGeneration);

    assert!(!prompt.is_empty());
    assert!(prompt.len() > 50); // Should be substantial
}

#[test]
fn test_generate_user_prompt_for_title() {
    let prompt = generate_user_prompt("Create a bold title for a product launch", None);

    assert!(prompt.contains("product launch"));
}

#[test]
fn test_generate_user_prompt_with_current_code() {
    let current = "= Old Title";
    let prompt = generate_user_prompt("Make the title italic", Some(current));

    assert!(prompt.contains("Old Title"));
    assert!(prompt.contains("italic"));
}

// ============================================================================
// Visual Verification Tests
// ============================================================================

#[test]
fn test_verification_result_success() {
    let result = VerificationResult::Success {
        confidence: 0.95,
        message: "Changes match intent".to_string(),
    };

    assert!(result.is_success());
    assert!(!result.needs_retry());
}

#[test]
fn test_verification_result_needs_retry() {
    let result = VerificationResult::NeedsRetry {
        reason: "Title not visible".to_string(),
        suggestion: "Increase font size".to_string(),
    };

    assert!(!result.is_success());
    assert!(result.needs_retry());
}

#[test]
fn test_verification_result_failed() {
    let result = VerificationResult::Failed {
        error: "Compilation failed".to_string(),
    };

    assert!(!result.is_success());
    assert!(!result.needs_retry());
}

#[test]
fn test_verify_change_with_compilation_error() {
    // If there's a compilation error, verification should fail
    let result = verify_change(
        "Make a title",
        None, // No SVG because compilation failed
        Some("Unexpected token at line 1".to_string()),
    );

    assert!(!result.is_success());
}

#[test]
fn test_verify_change_with_empty_svg() {
    let result = verify_change("Add content", Some("".to_string()), None);

    // Empty SVG should trigger retry
    assert!(result.needs_retry());
}

// ============================================================================
// Agent Loop Tests
// ============================================================================

#[test]
fn test_agent_config_default() {
    let config = AgentConfig::default();

    assert!(config.max_iterations >= 1);
    assert!(config.max_iterations <= 10);
}

#[test]
fn test_agent_config_custom() {
    let config = AgentConfig {
        max_iterations: 5,
        model: "anthropic/claude-3.5-haiku".to_string(),
        enable_visual_verification: true,
    };

    assert_eq!(config.max_iterations, 5);
    assert_eq!(config.model, "anthropic/claude-3.5-haiku");
}

#[test]
fn test_agent_state_initial() {
    let state = AgentState::new();

    assert_eq!(state.iteration, 0);
    assert!(state.history.is_empty());
    assert!(state.last_error.is_none());
    assert!(state.last_svg.is_none());
}

#[test]
fn test_agent_state_increment_iteration() {
    let mut state = AgentState::new();
    state.increment_iteration();

    assert_eq!(state.iteration, 1);
}

#[test]
fn test_agent_state_add_history() {
    let mut state = AgentState::new();
    state.add_to_history("Generated code".to_string());

    assert_eq!(state.history.len(), 1);
    assert_eq!(state.history[0], "Generated code");
}

#[test]
fn test_agent_state_should_continue_under_max() {
    let config = AgentConfig {
        max_iterations: 3,
        ..Default::default()
    };
    let mut state = AgentState::new();
    state.increment_iteration();

    assert!(state.should_continue(&config));
}

#[test]
fn test_agent_state_should_not_continue_at_max() {
    let config = AgentConfig {
        max_iterations: 3,
        ..Default::default()
    };
    let mut state = AgentState::new();
    state.iteration = 3;

    assert!(!state.should_continue(&config));
}

#[test]
fn test_agent_loop_creation() {
    let client_config = OpenRouterConfig::with_key("test".to_string());
    let client = OpenRouterClient::new(client_config);
    let agent_config = AgentConfig::default();

    let loop_instance = AgentLoop::new(client, agent_config);

    assert_eq!(loop_instance.state().iteration, 0);
}

#[test]
fn test_agent_loop_reset() {
    let client_config = OpenRouterConfig::with_key("test".to_string());
    let client = OpenRouterClient::new(client_config);
    let agent_config = AgentConfig::default();

    let mut loop_instance = AgentLoop::new(client, agent_config);
    loop_instance.state_mut().increment_iteration();
    loop_instance.state_mut().add_to_history("test".to_string());

    loop_instance.reset();

    assert_eq!(loop_instance.state().iteration, 0);
    assert!(loop_instance.state().history.is_empty());
}

// ============================================================================
// Integration-style Tests (with mocks)
// ============================================================================

#[test]
fn test_error_recovery_prompt_includes_error() {
    let error = "Unexpected closing bracket at line 5";
    let code = "= Title\n\n#text[Hello]]";

    let prompt = super::prompts::generate_error_recovery_prompt(code, error);

    assert!(prompt.contains(error));
    assert!(prompt.contains(code));
}

#[test]
fn test_agent_config_model_options() {
    // Verify all supported models can be configured
    let models = vec![
        "google/gemini-3-flash-preview",
        "anthropic/claude-3.5-haiku",
        "openai/gpt-4o-mini",
    ];

    for model in models {
        let config = AgentConfig {
            model: model.to_string(),
            ..Default::default()
        };
        assert_eq!(config.model, model);
    }
}
