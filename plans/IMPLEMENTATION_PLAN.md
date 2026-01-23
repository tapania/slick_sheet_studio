# Slick Sheet Studio - Multi-Phase Implementation Plan

**Created:** January 15, 2026
**Status:** Active Development Plan
**Target Completion:** 60-70% complete features fully exposed to users

---

## 1. Executive Summary

Slick Sheet Studio has a robust backend with comprehensive test coverage (~195 tests), but significant features remain inaccessible through the user interface. This plan addresses the gap between implemented capabilities and user-facing functionality through five sequential phases, each delivering testable end-to-end features.

### Current State Assessment

| Component | Backend | UI Accessible | Gap |
|-----------|---------|---------------|-----|
| Typst Compilation | 100% | 100% | None |
| Template System (Handlebars) | 100% | 0% | Critical |
| AI Agent Loop | 90% | 30% | High |
| AI Tools (ReadJson/WriteJson) | 100% | 0% | Critical |
| Visual Verification | 80% | 0% | High |
| Click-to-Edit | 100% | 0% (broken) | Critical |
| Error Display | 100% | 0% (broken) | Critical |
| Structured Data Editor | 100% | 0% | High |

### Key Deliverables

1. **Phase 1**: Fix broken core features (click-to-edit, error display)
2. **Phase 2**: CLI agent interface for development/testing
3. **Phase 3**: Activate template engine + structured editing in UI
4. **Phase 4**: Visual verification + advanced AI features
5. **Phase 5**: Polish, PWA enhancements, production readiness

---

## 2. Phase Overview

| Phase | Name | Duration | Parallel Tracks | Key Deliverables |
|-------|------|----------|-----------------|------------------|
| **1** | Critical Bug Fixes | 1-2 days | 2 | Click-to-edit working, error display visible |
| **2** | CLI Agent Interface | 2-3 days | 3 | `slick-cli` tool, AI testing without browser |
| **3** | Template Engine Activation | 3-4 days | 2 | JSON/Template split UI, structured data editor |
| **4** | Visual Verification | 2-3 days | 2 | Vision LLM integration, screenshot comparison |
| **5** | Polish & Production | 2-3 days | 3 | Syntax highlighting, PWA, performance |

**Total Estimated Duration:** 10-15 days

---

## 3. Phase 1: Critical Bug Fixes

**Objective:** Fix the two most critical broken features that prevent basic usability.

**Duration:** 1-2 days

### Parallel Track A: Click-to-Edit Fix

**Problem:** Click handler exists but clicking preview does nothing. SVG links are rendered but not intercepted.

**Root Cause Investigation:**
1. Check if `on_preview_click` handler is attached to the preview div
2. Verify DOM traversal finds `<a>` elements with `href` attributes
3. Confirm `parse_cmd_url()` correctly matches `cmd://edit/` URLs
4. Ensure `show_edit_modal` signal update triggers re-render

**Tasks:**

| Task | File | Description |
|------|------|-------------|
| 1.1 | `src/editor/mod.rs` | Add console logging to `on_preview_click` to verify handler fires |
| 1.2 | `src/editor/mod.rs` | Debug DOM traversal logic (lines 309-356) |
| 1.3 | `src/editor/mod.rs` | Verify `href` attribute extraction from SVG `<a>` elements |
| 1.4 | `src/editor/mod.rs` | Ensure `show_edit_modal.set(Some(...))` triggers component render |
| 1.5 | `src/editor/tests.rs` | Add browser test for click-to-edit using `wasm-bindgen-test` |

**Acceptance Criteria:**
- [x] Clicking a `cmd://edit/title` link in preview opens EditModal
- [x] EditModal pre-fills with current field value
- [x] Saving from EditModal updates Typst source
- [x] Preview re-renders after edit save
- [x] Browser test passes: click link -> modal opens -> save -> source updated

### Parallel Track B: Error Display Fix

**Problem:** Error signal is set but no UI renders it.

**Root Cause Investigation:**
1. Find where `error` signal is rendered in the view
2. Check if conditional render logic is correct
3. Verify CSS doesn't hide the error element

**Tasks:**

| Task | File | Description |
|------|------|-------------|
| 1.6 | `src/editor/mod.rs` | Locate error display code or add if missing |
| 1.7 | `src/editor/mod.rs` | Add error banner above preview pane |
| 1.8 | `styles.css` | Add error banner styling (red background, white text) |
| 1.9 | `src/editor/tests.rs` | Add test for error display on invalid Typst |

**Acceptance Criteria:**
- [x] Invalid Typst code shows red error banner with error message
- [x] Error clears when code is fixed and recompiled
- [x] Error message includes line number if available
- [x] Multiple errors display as newline-separated list

### Phase 1 Dependencies

- None (this is the first phase)

### Phase 1 Testing Strategy

```bash
# Unit tests for click-to-edit URL parsing
cargo test editor::tests::test_parse_cmd_url

# Browser tests for interaction
wasm-pack test --headless --firefox --test browser_tests

# Manual verification
trunk serve
# 1. Load Product Sheet template
# 2. Click on title in preview -> modal should open
# 3. Edit title, save -> preview should update
# 4. Enter invalid Typst syntax -> error should display
```

---

## 4. Phase 2: CLI Agent Interface

**Objective:** Create a command-line tool for testing AI features without browser automation.

**Duration:** 2-3 days

### Parallel Track A: CLI Tool Core

**Purpose:** Enable AI feature development and testing from the terminal.

**Tasks:**

| Task | File | Description |
|------|------|-------------|
| 2.1 | `src/bin/slick-cli.rs` | Create CLI entry point with clap argument parsing |
| 2.2 | `src/bin/slick-cli.rs` | Implement `read-json` command |
| 2.3 | `src/bin/slick-cli.rs` | Implement `write-json` command |
| 2.4 | `src/bin/slick-cli.rs` | Implement `read-template` command |
| 2.5 | `src/bin/slick-cli.rs` | Implement `write-template` command |
| 2.6 | `src/bin/slick-cli.rs` | Implement `render` command (JSON + template -> Typst) |
| 2.7 | `src/bin/slick-cli.rs` | Implement `compile` command (Typst -> SVG/PDF/PNG) |
| 2.8 | `src/bin/slick-cli.rs` | Implement `agent` command (run AI agent loop) |

**CLI Specification:**

```bash
# Read current JSON content
slick-cli read-json --project ./project.json

# Write new JSON content (validates before accepting)
slick-cli write-json --project ./project.json --input ./new-data.json

# Read current template
slick-cli read-template --project ./project.json

# Write new template (validates before accepting)
slick-cli write-template --project ./project.json --input ./new-template.typ

# Render template with data
slick-cli render --data ./data.json --template ./template.typ --output ./output.typ

# Compile to outputs
slick-cli compile --input ./document.typ --output-svg ./out.svg
slick-cli compile --input ./document.typ --output-pdf ./out.pdf
slick-cli compile --input ./document.typ --output-png ./out.png

# Run AI agent loop
slick-cli agent --project ./project.json --prompt "Make the title red" --max-iterations 3
slick-cli agent --project ./project.json --prompt-file ./prompt.txt --model "anthropic/claude-4.5-haiku"
```

### Parallel Track B: Environment Configuration

**Purpose:** Set up API key loading and test environment.

**Tasks:**

| Task | File | Description |
|------|------|-------------|
| 2.9 | `.env.testing.template` | Create template file with required variables |
| 2.10 | `src/bin/slick-cli.rs` | Load API key from `.env.testing` or `OPENROUTER_API_KEY` env var |
| 2.11 | `src/bin/slick-cli.rs` | Add `--dry-run` flag to preview AI actions without executing |
| 2.12 | `tests/cli_integration.rs` | Create CLI integration test suite |

**.env.testing Template:**

```bash
# Copy to .env.testing and fill in your API key
OPENROUTER_API_KEY=sk-or-v1-your-key-here

# Optional: Override default model
SLICK_AI_MODEL=google/gemini-3-flash

# Optional: Override max iterations
SLICK_AI_MAX_ITERATIONS=3
```

### Parallel Track C: AI Tool Integration

**Purpose:** Connect existing AI tools to CLI and expose full functionality.

**Tasks:**

| Task | File | Description |
|------|------|-------------|
| 2.13 | `src/ai/tools/mod.rs` | Add `execute_from_cli()` method to each tool |
| 2.14 | `src/ai/agent.rs` | Add tool execution mode to AgentLoop |
| 2.15 | `src/ai/prompts.rs` | Activate `ToolBasedEditing` prompt template |
| 2.16 | `src/bin/slick-cli.rs` | Implement `--tool-mode` flag for tool-based editing |
| 2.17 | `tests/ai_tool_tests.rs` | End-to-end tests for AI tool execution via CLI |

**Acceptance Criteria:**
- [x] `slick-cli read-json` outputs current project JSON
- [x] `slick-cli write-json` validates and updates project file
- [x] `slick-cli compile` produces valid SVG, PDF outputs (PNG not implemented)
- [x] `slick-cli agent` runs AI loop with real API (using `.env.testing`)
- [x] All CLI commands have `--help` documentation
- [ ] Integration tests pass with mocked API responses (skipped - real API tested)

### Phase 2 Dependencies

- Phase 1 complete (error handling needed for CLI feedback)

### Phase 2 Testing Strategy

```bash
# Load environment
source .env.testing

# Test read commands
slick-cli read-json --project test-fixtures/product-sheet.json
slick-cli read-template --project test-fixtures/product-sheet.json

# Test compilation
slick-cli compile --input test-fixtures/hello.typ --output-svg /tmp/test.svg
slick-cli compile --input test-fixtures/hello.typ --output-pdf /tmp/test.pdf

# Test AI agent (real API)
slick-cli agent --project test-fixtures/product-sheet.json \
  --prompt "Change the title to 'Test Product'" \
  --max-iterations 3 \
  --output ./output-project.json

# Verify output
diff test-fixtures/product-sheet.json ./output-project.json

# Run integration tests
cargo test --test cli_integration
```

---

## 5. Phase 3: Template Engine Activation

**Objective:** Expose the JSON/Template separation architecture through the UI.

**Duration:** 3-4 days

### Parallel Track A: UI Data/Template Split

**Purpose:** Replace raw Typst editing with structured JSON + template editing.

**Tasks:**

| Task | File | Description |
|------|------|-------------|
| 3.1 | `src/editor/mod.rs` | Add tab switcher: "Content (JSON)" / "Template" / "Output (Typst)" |
| 3.2 | `src/editor/mod.rs` | Create JSON editor panel with syntax highlighting (if available) |
| 3.3 | `src/editor/mod.rs` | Create template editor panel |
| 3.4 | `src/editor/mod.rs` | Add "Typst Output" read-only panel showing rendered template |
| 3.5 | `src/editor/state.rs` | Wire `content_data`, `template_source`, `typst_source` signals together |
| 3.6 | `src/editor/mod.rs` | Update compile flow: JSON + Template -> Typst -> SVG |
| 3.7 | `src/templates/mod.rs` | Update template selection to load both template AND default JSON |

**UI Layout Change:**

```
Before:
+------------------+------------------+------------------+
| Typst Code       | Preview          | AI Assistant     |
+------------------+------------------+------------------+

After:
+------------------+------------------+------------------+
| [Content|Template|Typst]           |                  |
| JSON Editor      | Preview          | AI Assistant     |
| OR               | (click to edit)  |                  |
| Template Editor  |                  |                  |
| OR               |                  |                  |
| Typst (readonly) |                  |                  |
+------------------+------------------+------------------+
```

### Parallel Track B: Structured Data Editor

**Purpose:** Provide form-based editing for SlickSheetData fields.

**Tasks:**

| Task | File | Description |
|------|------|-------------|
| 3.8 | `src/editor/data_editor.rs` | Create new component for structured data editing |
| 3.9 | `src/editor/data_editor.rs` | Title/subtitle/body text inputs |
| 3.10 | `src/editor/data_editor.rs` | Sections editor (add/remove/reorder) |
| 3.11 | `src/editor/data_editor.rs` | Features list editor |
| 3.12 | `src/editor/data_editor.rs` | Stats editor with color pickers |
| 3.13 | `src/editor/data_editor.rs` | Contact info form |
| 3.14 | `src/editor/data_editor.rs` | Style hints (colors, fonts) |
| 3.15 | `src/editor/mod.rs` | Add "Form" / "JSON" toggle within Content tab |

**Acceptance Criteria:**
- [ ] Selecting a template loads BOTH template and default JSON data
- [x] Editing JSON updates preview via template engine (wired up)
- [x] Editing template updates preview with current JSON data (wired up)
- [x] "Typst Output" tab shows rendered template
- [ ] Form editor can modify all SlickSheetData fields (deferred)
- [ ] Click-to-edit in preview updates JSON, not raw Typst (deferred)
- [ ] AI chat uses tool-based editing (read/write JSON and template) (Phase 4)

### Phase 3 Dependencies

- Phase 1 complete (click-to-edit must work)
- Phase 2 complete (CLI for testing AI tools)

### Phase 3 Testing Strategy

```bash
# Unit tests for template engine
cargo test template::

# Integration test: JSON + template -> Typst -> SVG
cargo test --test template_integration

# Browser tests for new UI
wasm-pack test --headless --firefox --test editor_tabs

# Manual verification
trunk serve
# 1. Select "Product Sheet" template
# 2. Switch to "Content" tab - JSON editor should show
# 3. Edit title in JSON - preview should update
# 4. Switch to "Template" tab - template editor should show
# 5. Edit template styling - preview should update
# 6. Click title in preview - should update JSON, not raw Typst

# CLI verification of AI tool mode
slick-cli agent --project ./test-project.json \
  --prompt "Add a new feature: Fast Performance" \
  --tool-mode
# Should use write_json tool instead of raw Typst editing
```

---

## 6. Phase 4: Visual Verification

**Objective:** Enable AI visual verification for quality assurance.

**Duration:** 2-3 days

### Parallel Track A: Vision Integration

**Purpose:** Allow AI to verify changes visually using a vision LLM.

**Tasks:**

| Task | File | Description |
|------|------|-------------|
| 4.1 | `src/ai/agent.rs` | Activate `enable_visual_verification` flag |
| 4.2 | `src/ai/verify.rs` | Implement screenshot capture (SVG to PNG) |
| 4.3 | `src/ai/verify.rs` | Send PNG to vision LLM with verification prompt |
| 4.4 | `src/ai/verify.rs` | Parse verification response (JSON or text) |
| 4.5 | `src/ai/agent.rs` | Add retry loop based on verification result |
| 4.6 | `src/editor/settings_modal.rs` | Add "Enable Visual Verification" toggle |
| 4.7 | `src/editor/settings_modal.rs` | Add vision model selector (for verification) |

**Vision Models to Support:**
- `google/gemini-3-flash` (default, fast)
- `openai/gpt-5.2-mini` (alternative)
- `anthropic/claude-4.5-haiku` (alternative)

### Parallel Track B: CLI Visual Verification

**Purpose:** Support visual verification in CLI for automated testing.

**Tasks:**

| Task | File | Description |
|------|------|-------------|
| 4.8 | `src/bin/slick-cli.rs` | Add `--visual-verify` flag to `agent` command |
| 4.9 | `src/bin/slick-cli.rs` | Implement screenshot save (SVG -> PNG via `resvg` or similar) |
| 4.10 | `src/bin/slick-cli.rs` | Add `--save-screenshots` flag to save verification images |
| 4.11 | `tests/visual_verification_tests.rs` | E2E tests with real vision API |

**CLI Usage:**

```bash
# Run agent with visual verification
slick-cli agent --project ./project.json \
  --prompt "Make the header larger and blue" \
  --visual-verify \
  --save-screenshots ./screenshots/

# Screenshots saved:
# ./screenshots/iteration-1-before.png
# ./screenshots/iteration-1-after.png
# ./screenshots/iteration-2-after.png (if retry needed)
```

**Acceptance Criteria:**
- [ ] AI agent can request visual verification after changes
- [ ] Vision LLM receives PNG screenshot with verification prompt
- [ ] Failed verification triggers retry with feedback
- [ ] Max 2 verification retries (then accept current result)
- [ ] CLI supports `--visual-verify` with screenshot saving
- [ ] UI settings allow enabling/disabling visual verification
- [ ] Visual verification adds ~2-5 seconds per iteration

### Phase 4 Dependencies

- Phase 3 complete (template engine needed for structured edits)
- Phase 2 complete (CLI needed for testing)

### Phase 4 Testing Strategy

```bash
# Load environment
source .env.testing

# Test visual verification via CLI
slick-cli agent --project test-fixtures/product-sheet.json \
  --prompt "Change the title color to red" \
  --visual-verify \
  --save-screenshots ./test-screenshots/ \
  --max-iterations 3

# Verify screenshots were saved
ls ./test-screenshots/
# Should see: iteration-1-before.png, iteration-1-after.png, etc.

# Run visual verification tests
cargo test --test visual_verification_tests

# Manual browser test
trunk serve
# 1. Enable visual verification in settings
# 2. Ask AI to make a visual change
# 3. Observe "Verifying..." state in chat panel
# 4. Confirm change matches request
```

---

## 7. Phase 5: Polish & Production Readiness

**Objective:** Add polish features and ensure production quality.

**Duration:** 2-3 days

### Parallel Track A: Editor Enhancements

**Purpose:** Improve code editing experience.

**Tasks:**

| Task | File | Description |
|------|------|-------------|
| 5.1 | `src/editor/code_editor.rs` | Add line numbers to code editor |
| 5.2 | `src/editor/code_editor.rs` | Add basic syntax highlighting (keywords, strings) |
| 5.3 | `src/editor/mod.rs` | Add keyboard shortcuts (Cmd+S save, Cmd+E export) |
| 5.4 | `src/editor/mod.rs` | Add undo/redo buttons (visual, uses browser history) |
| 5.5 | `src/editor/mod.rs` | Add project renaming capability |
| 5.6 | `src/editor/mod.rs` | Add zoom controls for preview (+/- buttons, keyboard) |

### Parallel Track B: PWA & Offline

**Purpose:** Enhance offline capability and PWA experience.

**Tasks:**

| Task | File | Description |
|------|------|-------------|
| 5.7 | `sw.js` | Update service worker to cache all required assets |
| 5.8 | `index.html` | Add PWA manifest with icons |
| 5.9 | `src/persistence/mod.rs` | Implement IndexedDB storage for projects |
| 5.10 | `src/persistence/mod.rs` | Add auto-save every 30 seconds |
| 5.11 | `src/editor/mod.rs` | Show "Saved" indicator with timestamp |
| 5.12 | `src/editor/mod.rs` | Add "Restore from backup" option |

### Parallel Track C: Performance & Quality

**Purpose:** Ensure production-ready performance and code quality.

**Tasks:**

| Task | File | Description |
|------|------|-------------|
| 5.13 | All modules | Run `cargo clippy -- -D warnings` and fix all warnings |
| 5.14 | All modules | Run `cargo fmt` for consistent formatting |
| 5.15 | `Cargo.toml` | Optimize release build settings |
| 5.16 | `trunk.toml` | Configure WASM optimization (wasm-opt) |
| 5.17 | Tests | Ensure all 195+ tests pass |
| 5.18 | Tests | Add missing tests for new features |
| 5.19 | Documentation | Update CLAUDE.md with new CLI commands |

**Acceptance Criteria:**
- [ ] Line numbers visible in code editor
- [ ] Basic syntax highlighting for Typst keywords
- [ ] Cmd+S saves project, Cmd+E exports PDF
- [ ] Project name editable from status bar
- [ ] Preview zoom with +/- buttons and keyboard shortcuts
- [ ] PWA installable with proper icons
- [ ] Projects auto-save to IndexedDB every 30 seconds
- [ ] Offline mode works for editing (AI disabled)
- [ ] Zero clippy warnings
- [ ] All tests pass
- [ ] trunk build --release produces optimized WASM

### Phase 5 Dependencies

- All previous phases complete

### Phase 5 Testing Strategy

```bash
# Code quality
cargo fmt --check
cargo clippy -- -D warnings

# All tests
cargo test

# Release build
trunk build --release
ls -la dist/  # Check bundle size

# PWA verification
# 1. Open in Chrome
# 2. Check DevTools > Application > Service Workers
# 3. Verify all assets cached
# 4. Go offline, verify app still works

# Auto-save verification
trunk serve
# 1. Make edits
# 2. Wait 30 seconds
# 3. Refresh page
# 4. Verify content restored
```

---

## 8. CLI Tool Specification (Detailed)

### 8.1 Architecture

```
slick-cli (binary)
├── Commands
│   ├── read-json      - Output current JSON data
│   ├── write-json     - Validate and write new JSON
│   ├── read-template  - Output current template
│   ├── write-template - Validate and write new template
│   ├── render         - Render template with data
│   ├── compile        - Compile Typst to SVG/PDF/PNG
│   └── agent          - Run AI agent loop
├── Config
│   ├── --project      - Path to project.json file
│   ├── --env          - Path to .env file (default: .env.testing)
│   └── --verbose      - Enable debug output
└── Output
    ├── stdout         - Command output (JSON, Typst, etc.)
    ├── stderr         - Errors and warnings
    └── exit codes     - 0 success, 1 validation error, 2 system error
```

### 8.2 Command Reference

#### `read-json`

```bash
slick-cli read-json --project <path>

# Options:
#   --project <path>   Path to project JSON file (required)
#   --pretty           Pretty-print JSON output (default: true)
#   --compact          Compact JSON output

# Output: JSON content data to stdout
# Exit: 0 on success, 2 if file not found
```

#### `write-json`

```bash
slick-cli write-json --project <path> --input <path>
slick-cli write-json --project <path> --stdin

# Options:
#   --project <path>   Path to project JSON file (required)
#   --input <path>     Path to new JSON file
#   --stdin            Read JSON from stdin
#   --no-validate      Skip validation (not recommended)
#   --dry-run          Validate without writing

# Validation steps:
# 1. Parse JSON syntax
# 2. Validate SlickSheetData schema
# 3. Render with current template
# 4. Test compile to SVG

# Output: Success message or validation errors
# Exit: 0 on success, 1 on validation error, 2 on system error
```

#### `read-template`

```bash
slick-cli read-template --project <path>

# Output: Template Typst code to stdout
# Exit: 0 on success, 2 if file not found
```

#### `write-template`

```bash
slick-cli write-template --project <path> --input <path>
slick-cli write-template --project <path> --stdin

# Validation steps:
# 1. Validate Handlebars-style syntax
# 2. Render with current JSON data
# 3. Test compile to SVG

# Output: Success message or validation errors
# Exit: 0 on success, 1 on validation error, 2 on system error
```

#### `render`

```bash
slick-cli render --data <path> --template <path> --output <path>
slick-cli render --data <path> --template <path> --stdout

# Options:
#   --data <path>      Path to JSON data file (required)
#   --template <path>  Path to template file (required)
#   --output <path>    Output Typst file path
#   --stdout           Write to stdout instead of file

# Output: Rendered Typst code
# Exit: 0 on success, 1 on render error
```

#### `compile`

```bash
slick-cli compile --input <path> [--output-svg <path>] [--output-pdf <path>] [--output-png <path>]

# Options:
#   --input <path>      Path to Typst file (required)
#   --output-svg <path> Output SVG file
#   --output-pdf <path> Output PDF file
#   --output-png <path> Output PNG file (requires resvg)
#   --width <px>        PNG width in pixels (default: 816, letter at 96dpi)
#   --height <px>       PNG height in pixels (default: 1056)

# At least one output format required
# Exit: 0 on success, 1 on compile error
```

#### `agent`

```bash
slick-cli agent --project <path> --prompt <text> [options]

# Required:
#   --project <path>    Path to project JSON file
#   --prompt <text>     AI prompt text
#   OR --prompt-file <path>  Read prompt from file

# Options:
#   --model <id>        AI model (default: from env or google/gemini-3-flash)
#   --max-iterations <n> Max agent iterations (default: 3)
#   --tool-mode         Use tool-based editing (read/write JSON/template)
#   --visual-verify     Enable visual verification
#   --save-screenshots <dir>  Save verification screenshots
#   --output <path>     Write modified project to path
#   --dry-run           Show what would change without applying

# Environment:
#   OPENROUTER_API_KEY  API key (required, or from --env file)
#   SLICK_AI_MODEL      Default model override
#   SLICK_AI_MAX_ITERATIONS  Default max iterations override

# Output: Agent progress, final result, any errors
# Exit: 0 on success, 1 on agent failure, 2 on system error
```

### 8.3 Project File Format

```json
{
  "name": "My Project",
  "description": "Optional project description",
  "version": "1.0.0",
  "created_at": "2026-01-15T10:30:00Z",
  "modified_at": "2026-01-15T14:45:00Z",
  "data": {
    "title": "Product Name",
    "subtitle": "Tagline",
    "body": "Description...",
    "sections": [...],
    "features": [...],
    "stats": [...],
    "contact": {...},
    "style": {...}
  },
  "template": "// Typst template with {{placeholders}}\n...",
  "typst_source": "// Rendered Typst code (optional, regenerated)\n..."
}
```

---

## 9. Testing Strategy

### 9.1 Test Categories

| Category | Tool | Purpose | Location |
|----------|------|---------|----------|
| Unit Tests | `cargo test` | Individual function testing | `src/*/tests.rs` |
| Integration Tests | `cargo test --test` | Cross-module testing | `tests/` |
| Browser Tests | `wasm-pack test` | DOM interaction testing | `tests/wasm/` |
| CLI Tests | `cargo test --test cli` | CLI command testing | `tests/cli_integration.rs` |
| AI Tests | CLI + real API | AI feature E2E testing | `tests/ai_e2e.rs` |
| Visual Tests | CLI + screenshots | Screenshot comparison | `tests/visual/` |

### 9.2 Test Environment Setup

```bash
# 1. Create test environment file
cp .env.testing.template .env.testing
# Edit .env.testing with your OPENROUTER_API_KEY

# 2. Run all non-API tests
cargo test

# 3. Run API-dependent tests (uses real API, costs money)
source .env.testing
cargo test --test ai_integration -- --ignored

# 4. Run browser tests
wasm-pack test --headless --firefox

# 5. Run CLI tests
cargo test --test cli_integration
```

### 9.3 Test Coverage Targets

| Module | Current | Target | Notes |
|--------|---------|--------|-------|
| `data/` | 90% | 90% | Maintain |
| `template/` | 85% | 90% | Add edge cases |
| `ai/` | 80% | 85% | Add tool tests |
| `editor/` | 30% | 60% | Add component tests |
| `persistence/` | 70% | 80% | Add IndexedDB tests |
| `world/` | 75% | 80% | Add link tests |
| `bin/slick-cli` | 0% | 80% | New, full coverage |

### 9.4 Continuous Integration

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - name: Format check
        run: cargo fmt --check
      - name: Clippy
        run: cargo clippy -- -D warnings
      - name: Unit tests
        run: cargo test
      - name: WASM tests
        run: wasm-pack test --headless --firefox
      - name: Build release
        run: trunk build --release
```

---

## 10. Risk Assessment

### 10.1 Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Click-to-edit bug complex | Medium | High | Add extensive logging, browser devtools debugging |
| Template engine performance | Low | Medium | Benchmark, optimize hot paths |
| Vision API costs | Medium | Low | Use test budget, cache responses in tests |
| WASM bundle size increase | Medium | Medium | Monitor size, use wasm-opt aggressively |
| IndexedDB browser compatibility | Low | Medium | Feature detection, fallback to localStorage |

### 10.2 Schedule Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Phase 1 bugs harder than expected | Medium | High | Allocate extra day buffer |
| AI tool integration complexity | Medium | Medium | Start with simple tools, add complexity |
| UI redesign scope creep | High | Medium | Strict scope control, defer nice-to-haves |

### 10.3 External Dependencies

| Dependency | Risk | Mitigation |
|------------|------|------------|
| OpenRouter API | Low | Multiple model fallbacks |
| Typst crate updates | Low | Pin version, test upgrades |
| Leptos framework | Low | Stable release, good docs |

---

## 11. Success Metrics

### 11.1 Phase Completion Criteria

| Phase | Complete When |
|-------|---------------|
| Phase 1 | Click-to-edit works, errors display, all existing tests pass |
| Phase 2 | CLI tool works, AI agent runs via CLI, integration tests pass |
| Phase 3 | Template engine in UI, structured editor works, AI uses tools |
| Phase 4 | Visual verification works in CLI and browser |
| Phase 5 | All polish features done, PWA works, release build optimized |

### 11.2 Quality Gates

Every phase must pass:
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] All existing tests pass (no regressions)
- [ ] New tests for phase features pass
- [ ] Manual testing checklist complete
- [ ] Documentation updated

### 11.3 Final Success Criteria

- [ ] All features from `final_verdict.md` "Not Accessible" list are accessible
- [ ] Click-to-edit works reliably
- [ ] Error messages display to users
- [ ] AI can edit JSON and templates via tools
- [ ] Visual verification optional but functional
- [ ] CLI tool fully functional for testing
- [ ] 200+ tests passing
- [ ] Zero clippy warnings
- [ ] PWA installable and works offline
- [ ] Release build under 10MB WASM

---

## 12. Appendix

### A. File Locations Reference

```
Key files to modify:
├── src/editor/mod.rs           # Click-to-edit, error display, UI layout
├── src/editor/state.rs         # Signal wiring for JSON/template
├── src/editor/data_editor.rs   # NEW: Structured data form editor
├── src/ai/agent.rs             # Visual verification flag, tool mode
├── src/ai/prompts.rs           # Activate ToolBasedEditing prompt
├── src/ai/verify.rs            # Vision LLM integration
├── src/bin/slick-cli.rs        # NEW: CLI tool entry point
├── tests/cli_integration.rs    # NEW: CLI tests
├── tests/visual_verification.rs # NEW: Visual tests
└── .env.testing                # NEW: API key for testing
```

### B. Command Cheatsheet

```bash
# Development
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH"
trunk serve                     # Start dev server
cargo test                      # Run all tests
cargo clippy -- -D warnings     # Lint check

# CLI Tool
cargo build --bin slick-cli     # Build CLI
./target/debug/slick-cli --help # Show commands

# Testing with API
source .env.testing
slick-cli agent --project ./test.json --prompt "Test prompt"
```

### C. Related Documents

- `/Users/taala/repos/slick_sheet_studio/reviews/final_verdict.md` - Current state analysis
- `/Users/taala/repos/slick_sheet_studio/plans/archive/HLPP.md` - Original architecture
- `/Users/taala/repos/slick_sheet_studio/plans/archive/JSON_PLAN.md` - JSON/Template architecture
- `/Users/taala/repos/slick_sheet_studio/CLAUDE.md` - Development guidelines
