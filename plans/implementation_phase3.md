# Phase 3: The Agentic AI Layer
# Goal: Enable AI-driven layout generation with visual verification loop

# =============================================================================
# Agent Definitions
# =============================================================================

agent api-client-builder:
  model: opus
  prompt: |
    You are an HTTP client specialist for WASM.
    You build robust API clients with proper error handling.
    Write tests FIRST (TDD), then implement to pass tests.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent prompt-engineer:
  model: opus
  prompt: |
    You are an LLM prompt engineering expert.
    You design effective prompts for code generation.
    Write tests FIRST (TDD), then implement to pass tests.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent agent-architect:
  model: opus
  prompt: |
    You are an agentic systems architect.
    You design agent loops with proper state machines.
    Write tests FIRST (TDD), then implement to pass tests.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent vision-integrator:
  model: opus
  prompt: |
    You are a vision API integration specialist.
    You handle image encoding and vision model calls.
    Write tests FIRST (TDD), then implement to pass tests.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent ui-builder:
  model: opus
  prompt: |
    You are a Leptos component developer.
    You build clean, functional UI components.
    Focus on functionality over styling.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent verifier:
  model: opus
  prompt: |
    You are a strict build and test verifier.
    You ensure all tests pass, clippy has zero warnings,
    code is formatted, and the agent loop works correctly.
    Loop until ALL requirements are satisfied.

# =============================================================================
# Reusable Blocks
# =============================================================================

block tdd-cycle:
  session: current-agent
    prompt: "Write failing test for the next requirement"
  session: current-agent
    prompt: "Implement minimal code to pass the test"
  session: current-agent
    prompt: "Run @code-simplifier to simplify the implementation"
  session: current-agent
    prompt: "Refactor if needed while keeping tests green"

block lint-pass:
  session: current-agent
    prompt: "Run cargo fmt --check, fix any formatting issues"
  session: current-agent
    prompt: "Run cargo clippy -- -D warnings, fix all warnings"

# =============================================================================
# Phase 3 Workflow
# =============================================================================

# Step 1: OpenRouter API Client
let api-client = do:
  session api-client-builder:
    prompt: |
      Create src/ai/client.rs with OpenRouter client:

      TDD Requirements (write tests in src/ai/tests.rs FIRST):
      1. Test: OpenRouterClient::new(api_key) creates client
      2. Test: build_request() constructs correct JSON structure
      3. Test: parse_response() extracts content from valid response
      4. Test: parse_response() returns error for invalid JSON
      5. Test: parse_response() handles rate limit errors
      6. Test: supports_vision() returns true for vision models

      OpenRouterClient struct:
      - api_key: String
      - base_url: &'static str = "https://openrouter.ai/api/v1"
      - model: String

      Methods:
      - async complete(messages: Vec<Message>) -> Result<String, ApiError>
      - async complete_with_image(messages: Vec<Message>, image_base64: &str) -> Result<String, ApiError>

      Message struct:
      - role: Role (System, User, Assistant)
      - content: String

      Use reqwest with WASM feature.
      Mock HTTP in tests using a trait-based approach.

      Loop until all tests pass.

  do tdd-cycle
  do lint-pass

# Step 2: Parallel - Prompts and Vision
parallel-do:

  # Agent A: Prompt Templates
  let prompts = do:
    session prompt-engineer:
      prompt: |
        Create src/ai/prompts.rs with model-specific prompts:

        TDD Requirements (write tests FIRST):
        1. Test: get_system_prompt(Model::Claude) returns Claude-optimized prompt
        2. Test: get_system_prompt(Model::GPT4) returns GPT4-optimized prompt
        3. Test: get_system_prompt(Model::Gemini) returns Gemini-optimized prompt
        4. Test: build_generation_prompt includes current code
        5. Test: build_verification_prompt includes user intent
        6. Test: build_error_recovery_prompt includes error details

        Model enum: Claude, GPT4, Gemini

        Prompt templates should:
        - Instruct model to output ONLY valid Typst code
        - Include examples of Typst syntax
        - For verification: ask if rendered output matches intent
        - For recovery: include compile error, ask for fix

        Store prompts as const &str for efficiency.

        Loop until all tests pass.

    do tdd-cycle
    do lint-pass

  # Agent B: Vision/Verification
  let vision = do:
    session vision-integrator:
      prompt: |
        Create src/ai/verify.rs with visual verification:

        TDD Requirements (write tests FIRST):
        1. Test: svg_to_png(svg) returns valid PNG bytes (use mock)
        2. Test: encode_image(bytes) returns valid base64
        3. Test: build_verification_request includes image
        4. Test: parse_verification_response extracts yes/no/feedback
        5. Test: VerificationResult enum has Success, Failure(feedback)

        VerificationResult enum:
        - Success
        - Failure { feedback: String }
        - Uncertain { feedback: String }

        Methods:
        - async verify_change(
            client: &OpenRouterClient,
            original_svg: &str,
            new_svg: &str,
            user_intent: &str,
          ) -> Result<VerificationResult, ApiError>

        For SVG→PNG: use web_sys canvas API in WASM.
        For tests: mock the canvas rendering.

        Loop until all tests pass.

    do tdd-cycle
    do lint-pass

# Step 3: Agent State Machine
let agent-core = do:
  session agent-architect:
    prompt: |
      Create src/ai/mod.rs with agent orchestration:

      TDD Requirements (write tests FIRST):
      1. Test: Agent::new(config) creates agent with settings
      2. Test: AgentState starts at Idle
      3. Test: process_request transitions to Generating
      4. Test: on generation success, transitions to Compiling
      5. Test: on compile success, transitions to Verifying
      6. Test: on verification success, transitions to Complete
      7. Test: on verification failure, transitions to Generating (retry)
      8. Test: after max_iterations, transitions to Failed
      9. Test: error context accumulates across retries

      AgentState enum:
      - Idle
      - Generating { attempt: u32 }
      - Compiling
      - Verifying
      - Complete { new_code: String }
      - Failed { reason: String }

      AgentConfig struct:
      - max_iterations: u32 (from settings)
      - model: Model
      - client: OpenRouterClient

      Agent struct:
      - config: AgentConfig
      - state: AgentState
      - history: Vec<AgentStep>

      AgentStep for debugging:
      - action: String
      - result: String
      - timestamp: Instant

      Main method:
      - async run(
          current_code: &str,
          user_request: &str,
          world: &VirtualWorld,
        ) -> Result<String, AgentError>

      The run() method implements the full loop:
      1. Generate new Typst code
      2. Compile with VirtualWorld
      3. If compile fails → feed error to LLM, retry
      4. Render to SVG/PNG
      5. Call vision verification
      6. If verification fails → feed feedback to LLM, retry
      7. If success → return new code
      8. If max iterations → return error

      Loop until all tests pass.

  do tdd-cycle
  do lint-pass

# Step 4: Settings UI
let settings-ui = do:
  session ui-builder:
    prompt: |
      Create src/ai/settings.rs with settings modal:

      Requirements:
      1. Modal for AI configuration
      2. API key input (password field)
      3. Model selector dropdown (Claude/GPT4/Gemini)
      4. Max iterations slider (1-10, default 5)
      5. Save to localStorage
      6. Load from localStorage on init

      Components:
      - SettingsModal component
      - AiSettings struct (Serialize/Deserialize)
      - load_settings() -> Option<AiSettings>
      - save_settings(settings: &AiSettings)

      Use web_sys for localStorage access.

  do lint-pass

# Step 5: Chat Interface
let chat-ui = do:
  session ui-builder:
    prompt: |
      Create src/ai/chat.rs with chat interface:

      Requirements:
      1. Input field for user prompts
      2. Send button
      3. Display agent status/progress
      4. Show iteration count during processing
      5. Display final result or error
      6. History of recent prompts (optional)

      Components:
      - ChatPanel component
      - Connect to Agent::run()
      - Show spinner during processing
      - Display agent history steps for debugging

      Integrate into main Editor layout.

  do lint-pass

# Step 6: Integration
let integration = do:
  session agent-architect:
    prompt: |
      Integrate AI agent with Editor:

      In src/app.rs or src/editor/mod.rs:
      1. Add settings button → opens SettingsModal
      2. Add chat panel to editor layout
      3. On user prompt:
         a. Get current Typst code from editor
         b. Call Agent::run()
         c. On success: update editor with new code
         d. On failure: display error in chat

      Wire up all components end-to-end.

  do lint-pass

# Step 7: Final Verification
do:
  session verifier:
    prompt: |
      Verify Phase 3 is COMPLETE:

      Checklist:
      [ ] cargo fmt --check passes
      [ ] cargo clippy -- -D warnings passes
      [ ] cargo test passes (all tests green)
      [ ] OpenRouter client tests pass (mocked)
      [ ] Prompt template tests pass
      [ ] Vision verification tests pass (mocked)
      [ ] Agent state machine tests pass
      [ ] trunk serve runs without errors
      [ ] Settings modal saves/loads API key
      [ ] Chat panel accepts prompts
      [ ] Agent loop executes (with real API key):
          - Generates code
          - Compiles
          - Verifies with vision
          - Retries on failure
          - Completes or fails gracefully
      [ ] "Make the title red" works end-to-end

      If ANY check fails:
      1. Identify the failing component
      2. Fix the issue
      3. Re-run ALL checks

      Loop until ALL checks pass.

      Output: "PHASE 3 COMPLETE" only when everything passes.

# =============================================================================
# Exit Criteria
# =============================================================================

# Phase 3 is complete when:
# - OpenRouter client working (all models)
# - Model-specific prompt templates
# - Visual verification with vision LLM
# - Agent state machine with retry logic
# - Settings modal with localStorage persistence
# - Chat interface for user interaction
# - Full agent loop: generate → compile → verify → retry/complete
# - All tests pass with mocked external calls
# - Manual verification with real API key works
