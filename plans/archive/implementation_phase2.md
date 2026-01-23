# Phase 2: The Editor Core
# Goal: Build the split-pane editor with live preview and click-to-edit

# =============================================================================
# Agent Definitions
# =============================================================================

agent model-builder:
  model: opus
  prompt: |
    You are a Rust data modeling expert.
    You design clean structs with proper serde serialization.
    Write tests FIRST (TDD), then implement to pass tests.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent signal-architect:
  model: opus
  prompt: |
    You are a Leptos reactive programming expert.
    You design signal-based state management.
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

agent link-interceptor:
  model: opus
  prompt: |
    You are a DOM event handling specialist.
    You intercept browser events and custom URL schemes.
    Write tests FIRST (TDD), then implement to pass tests.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent browser-tester:
  model: opus
  prompt: |
    You are a browser automation testing specialist.
    You use @dev-browser to verify UI behavior in real browsers.
    You test user interactions, click events, and modal dialogs.
    Always run @code-simplifier after writing code.

agent verifier:
  model: opus
  prompt: |
    You are a strict build and test verifier.
    You ensure all tests pass, clippy has zero warnings,
    code is formatted, and features work correctly.
    Use @dev-browser for browser verification.
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

block browser-test:
  session: browser-tester
    prompt: |
      Use @dev-browser to test the running application:
      1. Navigate to http://localhost:8080
      2. Wait for WASM to load
      3. Verify page elements are present
      4. Take screenshot for verification
  session: browser-tester
    prompt: "Perform interaction tests and validate results"

# =============================================================================
# Phase 2 Workflow
# =============================================================================

# Step 1: Content Model (foundation for editor)
let content-model = do:
  session model-builder:
    prompt: |
      Create src/editor/content.rs with Content model:

      TDD Requirements (write tests in src/editor/tests.rs FIRST):
      1. Test: Content::new() creates default content
      2. Test: Content serializes to JSON correctly
      3. Test: Content deserializes from JSON correctly
      4. Test: Content round-trip preserves all fields
      5. Test: Content::to_typst() generates valid Typst code

      Content struct fields:
      - title: String
      - subtitle: Option<String>
      - body: String
      - image_url: Option<String>
      - metadata: HashMap<String, String>

      Implementation:
      - Derive Serialize, Deserialize, Clone, Debug
      - to_typst() method generates Typst markup with cmd:// links

      Loop until all tests pass.

  do tdd-cycle
  do lint-pass

# Step 2: Parallel UI Components
parallel-do:

  # Agent A: Code editor pane
  let code-pane = do:
    session ui-builder:
      prompt: |
        Create src/editor/code.rs with CodeEditor component:

        Requirements:
        1. Textarea for raw Typst code input
        2. Leptos signal for content binding
        3. Optional: line numbers display
        4. Monospace font styling

        Component signature:
        #[component]
        pub fn CodeEditor(
            content: RwSignal<String>,
            on_change: Callback<String>,
        ) -> impl IntoView

        Keep it simple - this is a textarea wrapper.

    do lint-pass

  # Agent B: Preview pane
  let preview-pane = do:
    session ui-builder:
      prompt: |
        Create src/editor/preview.rs with Preview component:

        Requirements:
        1. Div container for SVG output
        2. Use inner_html to render SVG string
        3. Accept SVG content as signal
        4. Handle loading/error states

        Component signature:
        #[component]
        pub fn Preview(
            svg: Signal<Option<String>>,
            error: Signal<Option<String>>,
        ) -> impl IntoView

        Display error message if compilation failed.

    do lint-pass

  # Agent C: Signal state management
  let state = do:
    session signal-architect:
      prompt: |
        Create src/editor/state.rs with EditorState:

        TDD Requirements (write tests FIRST):
        1. Test: EditorState::new() initializes with defaults
        2. Test: update_content triggers recompile signal
        3. Test: debounce delays rapid updates (300ms)
        4. Test: manual_mode disables auto-recompile

        EditorState struct:
        - content: RwSignal<Content>
        - typst_source: RwSignal<String>
        - svg_output: RwSignal<Option<String>>
        - error: RwSignal<Option<String>>
        - auto_preview: RwSignal<bool>
        - last_compile: RwSignal<Instant>

        Methods:
        - update_content(content: Content)
        - trigger_compile()
        - set_auto_preview(enabled: bool)

        Loop until all tests pass.

    do tdd-cycle
    do lint-pass

# Step 3: Link Interception (after UI components)
let link-handler = do:
  session link-interceptor:
    prompt: |
      Create src/editor/links.rs with cmd:// link handling:

      TDD Requirements (write tests FIRST):
      1. Test: parse_cmd_url("cmd://edit/title") → Some(EditCommand::Title)
      2. Test: parse_cmd_url("https://example.com") → None
      3. Test: parse_cmd_url("cmd://edit/body") → Some(EditCommand::Body)
      4. Test: EditCommand enum covers all editable fields

      Implementation:
      - EditCommand enum: Title, Subtitle, Body, Image, Metadata(String)
      - parse_cmd_url(url: &str) -> Option<EditCommand>
      - setup_link_interceptor() - attaches click handler to preview container
      - On cmd:// click: prevent default, emit edit event

      Use web_sys for DOM event handling.

      Loop until all tests pass.

  do tdd-cycle
  do lint-pass

# Step 4: Modal Editor (for click-to-edit)
let modal = do:
  session ui-builder:
    prompt: |
      Create src/editor/modal.rs with EditModal component:

      Requirements:
      1. Modal overlay that appears on cmd:// click
      2. Input field for editing the selected field
      3. Save/Cancel buttons
      4. Closes on save or cancel
      5. Updates Content signal on save

      Component signature:
      #[component]
      pub fn EditModal(
          field: Signal<Option<EditCommand>>,
          content: RwSignal<Content>,
          on_close: Callback<()>,
      ) -> impl IntoView

      Keep styling minimal but functional.

  do lint-pass

# Step 5: Split Pane Layout
let layout = do:
  session ui-builder:
    prompt: |
      Create src/editor/mod.rs with main Editor component:

      Requirements:
      1. Split pane layout: left = code, right = preview
      2. Wire up EditorState
      3. Connect CodeEditor ↔ state ↔ Preview
      4. Integrate link interception
      5. Show EditModal when field clicked
      6. Toggle for auto-preview vs manual refresh
      7. Manual refresh button

      Component signature:
      #[component]
      pub fn Editor() -> impl IntoView

      This is the main editor orchestration component.

  do lint-pass

# Step 6: Integration with App
let integration = do:
  session ui-builder:
    prompt: |
      Update src/app.rs to use new Editor:

      1. Replace simple textarea with Editor component
      2. Initialize EditorState with VirtualWorld
      3. Connect compile pipeline to state changes
      4. Handle debounced vs manual preview mode

      Verify: Full editor workflow works end-to-end.

  do lint-pass

# Step 7: Browser Testing
let browser-tests = do:
  session browser-tester:
    prompt: |
      Start trunk serve in background, then use @dev-browser to test:

      Test Scenarios:
      1. Navigate to http://localhost:8080
      2. Verify split pane layout is visible (code left, preview right)
      3. Type Typst code in left pane
      4. Verify preview updates after debounce delay
      5. Toggle auto-preview off
      6. Verify typing no longer auto-updates preview
      7. Click manual refresh button
      8. Verify preview updates
      9. Click on editable text in preview (cmd:// link)
      10. Verify edit modal opens
      11. Modify text in modal, click Save
      12. Verify preview reflects the change
      13. Click Cancel in modal
      14. Verify no changes applied
      15. Take screenshots at each major step

      Use @dev-browser for all browser interactions.
      Loop until all browser tests pass.

  do browser-test

# Step 8: Final Verification
do:
  session verifier:
    prompt: |
      Verify Phase 2 is COMPLETE:

      Automated Checks:
      [ ] cargo fmt --check passes
      [ ] cargo clippy -- -D warnings passes
      [ ] cargo test passes (all tests green)
      [ ] Content model serialization tests pass
      [ ] Signal state tests pass
      [ ] Link parsing tests pass
      [ ] trunk serve runs without errors

      Browser Verification (use @dev-browser):
      [ ] Navigate to http://localhost:8080
      [ ] Split pane editor displays correctly
      [ ] Typing in code pane updates preview (debounced)
      [ ] Auto/manual preview toggle works
      [ ] Clicking text in SVG opens edit modal
      [ ] Editing in modal updates preview
      [ ] Modal Save/Cancel work correctly
      [ ] Take final screenshot for documentation

      If ANY check fails:
      1. Identify the failing component
      2. Fix the issue
      3. Re-run ALL checks

      Loop until ALL checks pass.

      Output: "PHASE 2 COMPLETE" only when everything passes.

# =============================================================================
# Exit Criteria
# =============================================================================

# Phase 2 is complete when:
# - Content model with full serde support
# - Split pane editor with code + preview
# - cmd:// link interception working
# - Edit modal opens on click, saves changes
# - Debounced auto-preview OR manual refresh
# - All tests pass, zero warnings
# - @dev-browser tests confirm all UI interactions work
