//! Agent orchestration for AI-driven document generation

use super::client::{ChatMessage, OpenRouterClient};
use super::prompts::{
    generate_error_recovery_prompt, generate_system_prompt, generate_user_prompt, PromptTemplate,
};
use super::verify::{verify_change, VerificationResult};

/// Configuration for the agent loop
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Maximum number of iterations before giving up
    pub max_iterations: usize,
    /// Model to use for generation
    pub model: String,
    /// Whether to use visual verification (planned feature)
    #[allow(dead_code)]
    pub enable_visual_verification: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 3,
            model: "google/gemini-3-flash-preview".to_string(),
            enable_visual_verification: false,
        }
    }
}

/// Current state of the agent loop
#[derive(Debug, Clone)]
pub struct AgentState {
    /// Current iteration number
    pub iteration: usize,
    /// History of generated code
    pub history: Vec<String>,
    /// Last compilation error, if any
    pub last_error: Option<String>,
    /// Last generated SVG, if any
    pub last_svg: Option<String>,
    /// Last generated Typst code
    pub last_code: Option<String>,
}

impl AgentState {
    /// Create a new initial state
    pub fn new() -> Self {
        Self {
            iteration: 0,
            history: Vec::new(),
            last_error: None,
            last_svg: None,
            last_code: None,
        }
    }

    /// Increment the iteration counter
    pub fn increment_iteration(&mut self) {
        self.iteration += 1;
    }

    /// Add code to history
    pub fn add_to_history(&mut self, code: String) {
        self.history.push(code);
    }

    /// Check if the loop should continue
    pub fn should_continue(&self, config: &AgentConfig) -> bool {
        self.iteration < config.max_iterations
    }
}

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of an agent loop iteration
#[derive(Debug)]
pub enum AgentResult {
    /// Successfully generated and verified code
    Success {
        code: String,
        svg: String,
        iterations: usize,
    },
    /// Generation failed after max iterations
    MaxIterationsReached {
        last_code: Option<String>,
        last_error: Option<String>,
    },
    /// A fatal error occurred
    Error(String),
}

/// The main agent loop for AI-driven generation
pub struct AgentLoop {
    client: OpenRouterClient,
    config: AgentConfig,
    state: AgentState,
}

impl AgentLoop {
    /// Create a new agent loop
    pub fn new(client: OpenRouterClient, config: AgentConfig) -> Self {
        Self {
            client,
            config,
            state: AgentState::new(),
        }
    }

    /// Get the current state (public API for monitoring)
    #[allow(dead_code)]
    pub fn state(&self) -> &AgentState {
        &self.state
    }

    /// Get mutable state (public API for external control)
    #[allow(dead_code)]
    pub fn state_mut(&mut self) -> &mut AgentState {
        &mut self.state
    }

    /// Reset the agent state
    pub fn reset(&mut self) {
        self.state = AgentState::new();
    }

    /// Run the agent loop with a compile function
    ///
    /// The compile function takes Typst code and returns either SVG or an error.
    pub async fn run<F>(
        &mut self,
        request: &str,
        current_code: Option<&str>,
        mut compile_fn: F,
    ) -> AgentResult
    where
        F: FnMut(&str) -> Result<String, String>,
    {
        self.reset();

        while self.state.should_continue(&self.config) {
            self.state.increment_iteration();

            // Determine which prompt to use
            let (system_prompt, user_prompt) = if let Some(error) = &self.state.last_error {
                // Error recovery mode
                let code = self.state.last_code.as_deref().unwrap_or("");
                (
                    generate_system_prompt(PromptTemplate::ErrorRecovery),
                    generate_error_recovery_prompt(code, error),
                )
            } else {
                // Normal generation mode
                (
                    generate_system_prompt(PromptTemplate::TypstGeneration),
                    generate_user_prompt(request, current_code),
                )
            };

            // Build messages
            let messages = vec![
                ChatMessage::system(system_prompt),
                ChatMessage::user(user_prompt),
            ];

            // Call the LLM
            let generated_code = match self.client.chat(&self.config.model, messages).await {
                Ok(code) => code,
                Err(e) => return AgentResult::Error(format!("LLM request failed: {}", e)),
            };

            // Clean up the code (remove markdown fences if present)
            let cleaned_code = clean_code(&generated_code);

            // Store the code
            self.state.last_code = Some(cleaned_code.clone());
            self.state.add_to_history(cleaned_code.clone());

            // Try to compile
            match compile_fn(&cleaned_code) {
                Ok(svg) => {
                    self.state.last_svg = Some(svg.clone());
                    self.state.last_error = None;

                    // Verify the output
                    let verification = verify_change(request, Some(svg.clone()), None);

                    match verification {
                        VerificationResult::Success { .. } => {
                            return AgentResult::Success {
                                code: cleaned_code,
                                svg,
                                iterations: self.state.iteration,
                            };
                        }
                        VerificationResult::NeedsRetry { reason, .. } => {
                            // Set up for retry with the reason as context
                            self.state.last_error =
                                Some(format!("Verification failed: {}", reason));
                        }
                        VerificationResult::Failed { error } => {
                            self.state.last_error = Some(error);
                        }
                    }
                }
                Err(error) => {
                    self.state.last_error = Some(error);
                    self.state.last_svg = None;
                }
            }
        }

        // Max iterations reached
        AgentResult::MaxIterationsReached {
            last_code: self.state.last_code.clone(),
            last_error: self.state.last_error.clone(),
        }
    }
}

/// Clean generated code by removing markdown code fences
fn clean_code(code: &str) -> String {
    let code = code.trim();

    // Remove opening fence (```typst, ```typ, or ```)
    let code = code
        .strip_prefix("```typst")
        .or_else(|| code.strip_prefix("```typ"))
        .or_else(|| code.strip_prefix("```"))
        .unwrap_or(code);

    // Remove closing fence
    let code = code.strip_suffix("```").unwrap_or(code);

    code.trim().to_string()
}

#[cfg(test)]
mod clean_code_tests {
    use super::clean_code;

    #[test]
    fn test_clean_code_no_fences() {
        let input = "= Hello World";
        assert_eq!(clean_code(input), "= Hello World");
    }

    #[test]
    fn test_clean_code_with_typst_fence() {
        let input = "```typst\n= Hello World\n```";
        assert_eq!(clean_code(input), "= Hello World");
    }

    #[test]
    fn test_clean_code_with_generic_fence() {
        let input = "```\n= Hello World\n```";
        assert_eq!(clean_code(input), "= Hello World");
    }
}
