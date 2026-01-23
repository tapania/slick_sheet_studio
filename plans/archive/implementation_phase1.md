# Phase 1: The Rust Foundation (VirtualWorld)
# Goal: Get Typst compiling a string to SVG inside a web browser

# =============================================================================
# Agent Definitions
# =============================================================================

agent scaffold-builder:
  model: opus
  prompt: |
    You are a Rust/WASM project scaffolding expert.
    You set up Leptos projects with proper Trunk configuration.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent world-implementer:
  model: opus
  prompt: |
    You are a Typst integration specialist.
    You implement the typst::World trait for WASM environments.
    Write tests FIRST (TDD), then implement to pass tests.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent font-loader:
  model: opus
  prompt: |
    You are a font handling expert for WASM applications.
    You embed fonts and implement loading logic.
    Write tests FIRST (TDD), then implement to pass tests.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent render-pipeline:
  model: opus
  prompt: |
    You are a rendering pipeline specialist.
    You connect Typst compilation to SVG output.
    Write tests FIRST (TDD), then implement to pass tests.
    Use insta for snapshot testing of SVG output.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent browser-tester:
  model: opus
  prompt: |
    You are a browser automation testing specialist.
    You use @dev-browser to verify UI behavior in real browsers.
    You test user interactions and visual rendering.
    Always run @code-simplifier after writing code.

agent verifier:
  model: opus
  prompt: |
    You are a strict build and test verifier.
    You ensure all tests pass, clippy has zero warnings,
    code is formatted, and the app runs correctly.
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
# Phase 1 Workflow
# =============================================================================

# Step 1: Project Scaffold (sequential - must complete first)
let scaffold = do:
  session scaffold-builder:
    prompt: |
      Create Leptos project scaffold:
      1. Initialize Cargo.toml with dependencies from CLAUDE.md approved list
      2. Create Trunk.toml for WASM build
      3. Create index.html entry point
      4. Create src/main.rs with minimal Leptos app
      5. Create src/lib.rs for library root (testing)
      6. Verify: trunk build --release compiles

      Loop until scaffold compiles successfully.

  do lint-pass

# Step 2: Parallel Implementation (after scaffold)
# These can run in parallel as they're independent modules

parallel-do:

  # Agent A: VirtualWorld implementation
  let world = do:
    session world-implementer:
      prompt: |
        Implement src/world/mod.rs with VirtualWorld struct:

        TDD Requirements (write tests in src/world/tests.rs FIRST):
        1. Test: VirtualWorld::new() creates valid instance
        2. Test: VirtualWorld implements typst::World trait
        3. Test: world.main_source() returns the main .typ content
        4. Test: world.source(id) returns correct file content
        5. Test: world.file(id) returns file bytes
        6. Test: world.font(idx) returns font data

        Implementation:
        - In-memory HashMap for virtual file system
        - Method to set main source content
        - Method to add files to virtual FS

        Loop until all tests pass.

    do tdd-cycle
    do lint-pass

  # Agent B: Font loading
  let fonts = do:
    session font-loader:
      prompt: |
        Implement src/world/fonts.rs with font loading:

        TDD Requirements (write tests FIRST):
        1. Test: load_embedded_fonts() returns Vec<Font>
        2. Test: embedded fonts include at least 2 fonts
        3. Test: FontLoader::get(idx) returns correct font

        Implementation:
        - Embed Inter and JetBrains Mono using include_bytes!
        - Parse fonts using typst's font loading
        - Provide iterator over available fonts

        Loop until all tests pass.

    do tdd-cycle
    do lint-pass

# Step 3: Render Pipeline (after world and fonts complete)
let pipeline = do:
  session render-pipeline:
    prompt: |
      Implement Typst → SVG render pipeline:

      TDD Requirements (write tests FIRST):
      1. Test: compile("Hello") returns Ok(Document)
      2. Test: compile with syntax error returns Err
      3. Test: render_svg(doc) returns valid SVG string
      4. Test: SVG contains expected text content
      5. Snapshot test: "Hello World" SVG matches baseline

      Implementation in src/world/mod.rs or src/render.rs:
      - compile(source: &str) -> Result<Document, Vec<SourceDiagnostic>>
      - render_svg(doc: &Document) -> String
      - Use typst::compile and typst_svg::svg

      Loop until all tests pass including snapshot.

  do tdd-cycle
  do lint-pass

# Step 4: Integration (connect to Leptos UI)
let integration = do:
  session scaffold-builder:
    prompt: |
      Integrate VirtualWorld with Leptos:

      In src/app.rs:
      1. Create simple textarea for Typst input
      2. Create div for SVG output
      3. On input change, compile and render
      4. Display errors if compilation fails

      This is UI glue code - focus on functionality over style.

      Verify: trunk serve shows working editor.

  do lint-pass

# Step 5: Browser Testing
let browser-tests = do:
  session browser-tester:
    prompt: |
      Start trunk serve in background, then use @dev-browser to test:

      Test Scenarios:
      1. Navigate to http://localhost:8080
      2. Wait for page load and WASM initialization
      3. Verify textarea is present and editable
      4. Verify SVG output container exists
      5. Type "Hello World" into textarea
      6. Verify SVG output updates with rendered text
      7. Type invalid Typst syntax
      8. Verify error message is displayed
      9. Take screenshot of working state

      Use @dev-browser for all browser interactions.
      Loop until all browser tests pass.

  do browser-test

# Step 6: Final Verification (strict)
do:
  session verifier:
    prompt: |
      Verify Phase 1 is COMPLETE:

      Automated Checks:
      [ ] cargo fmt --check passes
      [ ] cargo clippy -- -D warnings passes
      [ ] cargo test passes (all tests green)
      [ ] wasm-pack build compiles
      [ ] trunk serve runs without errors

      Browser Verification (use @dev-browser):
      [ ] Navigate to http://localhost:8080
      [ ] Typing in textarea updates SVG preview
      [ ] "Hello World" renders correctly
      [ ] Error handling works for invalid syntax
      [ ] Take final screenshot for documentation

      [ ] Snapshot tests match baselines

      If ANY check fails:
      1. Identify the failing component
      2. Fix the issue
      3. Re-run ALL checks

      Loop until ALL checks pass.

      Output: "PHASE 1 COMPLETE" only when everything passes.

# =============================================================================
# Exit Criteria
# =============================================================================

# Phase 1 is complete when:
# - All unit tests pass
# - All snapshot tests pass
# - Zero clippy warnings
# - Code is formatted
# - @dev-browser tests confirm working UI
# - trunk serve shows working "Hello World" → SVG
