//! AI module - Agentic AI system for document generation
//!
//! This module provides:
//! - OpenRouter client for LLM API calls
//! - Prompt templates for different tasks
//! - Visual verification logic
//! - Agent orchestration loop
//! - AI tools for JSON and template operations

pub mod agent;
pub mod client;
pub mod image_gen;
pub mod prompts;
pub mod tools;
pub mod verify;

#[cfg(test)]
mod tests;

pub use agent::{AgentConfig, AgentLoop, AgentResult};
pub use client::{OpenRouterClient, OpenRouterConfig};

// Re-exports for public API (not all used internally yet)
#[allow(unused_imports)]
pub use agent::AgentState;
#[allow(unused_imports)]
pub use client::{ChatMessage, Role};
#[allow(unused_imports)]
pub use image_gen::{generate_alt_description, ImageGenerator, IMAGE_MODEL};
#[allow(unused_imports)]
pub use prompts::generate_system_prompt;
#[allow(unused_imports)]
pub use prompts::{generate_tool_editing_prompt, generate_user_prompt, PromptTemplate};
#[allow(unused_imports)]
pub use tools::{
    AiTool, GenerateImageTool, ReadJsonTool, ReadTemplateTool, ToolResult, WriteJsonTool,
    WriteTemplateTool,
};
#[allow(unused_imports)]
pub use verify::{verify_change, VerificationResult};
