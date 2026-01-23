# AI Module Documentation

## Overview

The AI module (`src/ai/`) is an agentic AI system for document generation within Slick Sheet Studio. It provides:

- **OpenRouter API Client**: Integration with OpenRouter LLM API for chat completions
- **Agent Orchestration Loop**: Multi-iteration generation with error recovery
- **Prompt Management**: Task-specific prompt templates (generation, error recovery, visual verification, tool-based editing)
- **Visual Verification**: Output validation logic
- **AI Tools**: Read/write operations for JSON content and Typst templates with full validation

## Architecture

The module is organized into the following components:

### 1. Core Components

**`mod.rs`** - Module entry point and public API
- Exports: `AgentConfig`, `AgentLoop`, `AgentState`, `AgentResult`, `OpenRouterClient`, `OpenRouterConfig`, `ChatMessage`, `Role`, and tool types
- Marked with `#[allow(unused_imports)]` since not all exports are used internally (planned features)

**`client.rs`** - OpenRouter API client for LLM interactions
- Async HTTP client built on `gloo_net` for WASM compatibility
- Handles authentication and API request/response parsing

**`agent.rs`** - Main agent loop orchestration
- Manages multi-iteration document generation with error recovery
- Implements verification and retry logic

**`prompts.rs`** - Prompt template management
- Four template types: `TypstGeneration`, `ErrorRecovery`, `VisualVerification`, `ToolBasedEditing`
- Helper functions for generating system and user prompts

**`verify.rs`** - Visual verification logic
- Validates generated output against user intent
- Supports both basic SVG validation and vision LLM responses

**`tools/`** - AI tools subdirectory with four specialized tools for editing operations

---

## Public APIs

### Client Module (`client.rs`)

#### Types

```rust
pub enum Role {
    System,
    User,
    Assistant,
}

pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

pub struct OpenRouterConfig {
    pub api_key: String,
    pub base_url: String,           // Default: "https://openrouter.ai/api/v1"
    pub http_referer: String,
    pub x_title: String,
}

pub struct OpenRouterClient { ... }
```

#### Key Functions

**`ChatMessage`**
- `new(role: Role, content: String) -> Self`
- `system(content: String) -> Self` - Create system message
- `user(content: String) -> Self` - Create user message
- `assistant(content: String) -> Self` - Create assistant message (marked `#[allow(dead_code)]`)

**`OpenRouterConfig`**
- `default() -> Self` - Default configuration with placeholder API key
- `with_key(api_key: String) -> Self` - Create with API key

**`OpenRouterClient`**
- `new(config: OpenRouterConfig) -> Self`
- `config(&self) -> &OpenRouterConfig`
- `build_request_body(&self, model: &str, messages: &[ChatMessage]) -> String` - Build JSON request
- `parse_response(response: &str) -> Result<String, String>` - Parse and extract content from API response
- `async chat(&self, model: &str, messages: Vec<ChatMessage>) -> Result<String, String>` - Send chat request (WASM-compatible async)

### Agent Module (`agent.rs`)

#### Types

```rust
pub struct AgentConfig {
    pub max_iterations: usize,              // Default: 3
    pub model: String,                      // Default: "google/gemini-3-flash"
    pub enable_visual_verification: bool,   // Default: false (planned feature)
}

pub struct AgentState {
    pub iteration: usize,
    pub history: Vec<String>,               // All generated code
    pub last_error: Option<String>,
    pub last_svg: Option<String>,
    pub last_code: Option<String>,
}

pub enum AgentResult {
    Success {
        code: String,
        svg: String,
        iterations: usize,
    },
    MaxIterationsReached {
        last_code: Option<String>,
        last_error: Option<String>,
    },
    Error(String),
}

pub struct AgentLoop { ... }
```

#### Key Functions

**`AgentState`**
- `new() -> Self` - Create initial state
- `increment_iteration(&mut self)`
- `add_to_history(&mut self, code: String)`
- `should_continue(&self, config: &AgentConfig) -> bool` - Check if loop should continue

**`AgentLoop`**
- `new(client: OpenRouterClient, config: AgentConfig) -> Self`
- `state(&self) -> &AgentState` - Get current state
- `state_mut(&mut self) -> &mut AgentState` - Modify state
- `reset(&mut self)` - Reset to initial state
- `async run<F>(&mut self, request: &str, current_code: Option<&str>, compile_fn: F) -> AgentResult` where `F: FnMut(&str) -> Result<String, String>` - Run the agent loop

The `run` method is the main entry point. It:
1. Accepts a user request and optional current code
2. Accepts a compile function to test-compile Typst code
3. Iteratively generates and verifies code
4. Uses error recovery prompts when compilation fails
5. Returns success when verification passes or max iterations reached

### Prompts Module (`prompts.rs`)

#### Types

```rust
pub enum PromptTemplate {
    TypstGeneration,        // Generate new Typst markup
    ErrorRecovery,          // Fix errors in existing code
    VisualVerification,     // Verify output matches intent (planned)
    ToolBasedEditing,       // Tool-based editing mode (planned)
}
```

#### Key Functions

- `generate_system_prompt(template: PromptTemplate) -> String` - Get system prompt for a template type
- `generate_user_prompt(request: &str, current_code: Option<&str>) -> String` - Generate user prompt with optional context
- `generate_error_recovery_prompt(code: &str, error: &str) -> String` - Generate error recovery prompt
- `generate_visual_verification_prompt(original_request: &str, image_base64: &str) -> String` - Vision verification prompt (planned)
- `generate_tool_editing_prompt(request: &str, current_json: &str, current_template: &str) -> String` - Tool-based editing prompt (planned)

### Verification Module (`verify.rs`)

#### Types

```rust
pub enum VerificationResult {
    Success {
        confidence: f64,           // 0.0 - 1.0
        message: String,
    },
    NeedsRetry {
        reason: String,
        suggestion: String,
    },
    Failed {
        error: String,
    },
}
```

#### Key Functions

- `verify_change(request: &str, svg_output: Option<String>, compilation_error: Option<String>) -> VerificationResult` - Basic verification (checks SVG validity)
- `parse_verification_response(response: &str) -> VerificationResult` - Parse vision LLM response (JSON or text)

**Verification Logic**:
- If compilation error exists -> `Failed`
- If no SVG output -> `NeedsRetry` ("No output generated")
- If empty SVG -> `NeedsRetry` ("Empty SVG output")
- If valid SVG (contains `<svg` and >100 bytes) -> `Success` (confidence: 0.8)
- Otherwise -> `NeedsRetry` ("SVG output appears incomplete")

### Tools Module (`tools/`)

#### Trait

```rust
pub trait AiTool {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

pub enum ToolResult {
    Success(String),
    Error(String),
}
```

#### Tool Implementations

**1. ReadJsonTool**
- **Name**: `"read_json"`
- **Purpose**: Serialize current `SlickSheetData` to JSON
- **API**: `ReadJsonTool::execute(data: &SlickSheetData) -> ToolResult`
- **Validation**: Automatically validates JSON serialization

**2. WriteJsonTool**
- **Name**: `"write_json"`
- **Purpose**: Validate and write new JSON content data
- **API**:
  - `execute(new_json: &str, current_template: &str, compile_fn: impl FnOnce(&str) -> Result<String, Vec<String>>) -> Result<SlickSheetData, ToolResult>`
  - `execute_without_compile(new_json: &str) -> Result<SlickSheetData, ToolResult>`
- **Validation Steps**:
  1. Parse JSON syntax
  2. Validate schema (required fields, data types)
  3. Test rendering with current template
  4. Test Typst compilation (with compile_fn)
- **Required Fields**: `title` (non-empty string)
- **Optional Fields**: `subtitle`, `body`, `sections`, `features`, `stats`, `contact`, `style`

**3. ReadTemplateTool**
- **Name**: `"read_template"`
- **Purpose**: Return current Typst template
- **API**: `ReadTemplateTool::execute(template: &str) -> ToolResult`
- **Template Syntax Supported**:
  - `{{field}}` - Simple substitution
  - `{{field.subfield}}` - Nested access
  - `{{#if field}}...{{/if}}` - Conditionals
  - `{{#each items}}...{{/each}}` - Loops
  - `{{field | default: 'value'}}` - Default values

**4. WriteTemplateTool**
- **Name**: `"write_template"`
- **Purpose**: Validate and write new Typst template
- **API**:
  - `execute(new_template: &str, current_data: &SlickSheetData, compile_fn: impl FnOnce(&str) -> Result<String, Vec<String>>) -> Result<String, ToolResult>`
  - `execute_without_compile(new_template: &str, current_data: &SlickSheetData) -> Result<String, ToolResult>`
- **Validation Steps**:
  1. Validate template syntax
  2. Test rendering with current data
  3. Test Typst compilation (with compile_fn)
- **Constraints**: Cannot be empty

---

## Data Structures

### Key Types from Integration

The module integrates with `SlickSheetData` from `crate::data`:

```rust
pub struct SlickSheetData {
    pub title: String,
    pub subtitle: Option<String>,
    pub body: String,
    pub sections: Vec<Section>,
    pub features: Vec<String>,
    pub stats: Vec<Stat>,
    pub contact: ContactInfo,
    pub style: StyleHints,
}
```

The module also works with Typst compilation through a generic compile function interface.

---

## Integration Points

### Editor Integration

The AI module is used by the editor component (`src/editor/mod.rs`):

```rust
// From editor/mod.rs
use crate::ai::{AgentConfig, AgentLoop, OpenRouterClient, OpenRouterConfig};

// Usage pattern:
let config = OpenRouterConfig::with_key(api_key);
let client = OpenRouterClient::new(config);
let agent_config = AgentConfig { ... };
let mut agent = AgentLoop::new(client, agent_config);

let result = agent.run(
    "User request",
    Some(current_code),
    |code| compile_typst(code)  // Compile function
).await;
```

---

## Code Cleanup Features

The module includes a helper function `clean_code(code: &str) -> String` that:
- Removes markdown code fences (`` ```typst ``, `` ```typ ``, `` ``` ``)
- Trims whitespace
- Returns clean Typst code

---

## Test Coverage

The module includes comprehensive test suites covering:

### Client Tests (17 tests)
- Configuration creation and defaults
- Chat message creation (system, user, assistant)
- Request body building
- Response parsing (valid, empty, invalid, errors)

### Prompt Tests (6 tests)
- Template system prompts
- User prompt generation
- Error recovery prompts

### Verification Tests (6 tests)
- Result types (success, retry, failed)
- Change verification logic
- Empty/error handling

### Agent Tests (11 tests)
- Config and state management
- Iteration tracking
- History management
- Loop continuation logic
- State reset

### Tool Tests (13 tests)
- Round-trip JSON serialization
- Template validation with data
- Tool naming and descriptions
- Validation error handling
- JSON/template validation errors

**Total**: Approximately 50+ unit tests focused on core functionality.

---

## Key Architectural Patterns

### 1. Error Recovery Loop
The agent implements a sophisticated error recovery pattern:
- Initial generation uses `TypstGeneration` prompt
- If compilation fails, switches to `ErrorRecovery` prompt with error details
- Retries up to `max_iterations` times
- Terminates on success or max iterations

### 2. Multi-Stage Tool Validation
Both `WriteJsonTool` and `WriteTemplateTool` implement multi-stage validation:
1. Syntax validation (JSON/template structure)
2. Schema validation (required fields, types)
3. Rendering test (template engine works)
4. Compilation test (Typst produces valid output)

### 3. Async/WASM Compatible
- Uses `gloo_net` for HTTP (WASM-compatible)
- All async operations marked with `async fn`
- No blocking I/O operations

### 4. Dead Code Markers
Many features are marked with `#[allow(dead_code)]` indicating planned features:
- `VisualVerification` prompt template
- `ToolBasedEditing` prompt template
- Vision LLM verification functions
- Tool-based editing prompts

---

## Supported LLM Models

According to the tests, the following models are supported:
- `google/gemini-3-flash` (default)
- `anthropic/claude-4.5-haiku`
- `openai/gpt-5.2-mini`

Models are configured via `AgentConfig::model` and can be changed per run.

---

## Error Handling Strategy

### Client Errors
- Network failures: Wrapped in `Result` with descriptive messages
- API errors: Extracted from response and propagated
- JSON parsing: Returns descriptive parse errors

### Agent Errors
- Compilation failures: Captured and used for error recovery
- Max iterations: Returns partial result with last code/error
- Verification failures: Triggers retry with suggestion

### Tool Errors
- JSON validation: Returns validation error details
- Template syntax: Detailed syntax error reporting
- Rendering failures: Template engine error propagation
- Compilation failures: Wrapped with context

---

## Design Notes

### Dead Code Markers
The module includes several `#[allow(dead_code)]` markers for:
1. **Planned vision verification**: `enable_visual_verification`, `VisualVerification` prompt, vision functions
2. **Planned tool-based editing**: `ToolBasedEditing` prompt, tool editing prompt generator
3. **Rarely-used public methods**: `config()`, `state_mut()`, `assistant()` message creation

These are intentionally kept in the codebase for future feature implementation.

### Handlebars-Style Templating
The template tools use Handlebars-style syntax:
- Variable interpolation: `{{field}}`
- Nested access: `{{object.field}}`
- Conditionals: `{{#if condition}}...{{/if}}`
- Loops: `{{#each array}}...{{/each}}`
- Defaults: `{{field | default: 'value'}}`

This simplifies template creation for Typst markup generation.

---

## Dependency Usage

- **serde/serde_json**: JSON serialization/deserialization
- **gloo_net**: WASM-compatible HTTP client
- **Other crate functions**: Integration with `crate::data`, `crate::template` modules

---

## Summary

The AI module provides a complete agentic system for document generation with:
- **Robust error recovery**: Multi-iteration retry with contextual prompts
- **Comprehensive validation**: Multi-stage validation for JSON and templates
- **WASM support**: Fully async and compatible with browser execution
- **Flexible tool system**: Extensible tool architecture for future features
- **Production-ready**: Comprehensive test coverage and error handling

The system is designed to iteratively generate and refine Typst documents through LLM interactions, with automatic error correction and output verification.
