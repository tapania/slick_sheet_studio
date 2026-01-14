# Phase 5: Persistence & PWA
# Goal: Production-ready file handling and offline support

# =============================================================================
# Agent Definitions
# =============================================================================

agent persistence-architect:
  model: opus
  prompt: |
    You are a browser storage and File API specialist.
    You implement robust save/load with proper error handling.
    Write tests FIRST (TDD), then implement to pass tests.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent pdf-exporter:
  model: opus
  prompt: |
    You are a Typst PDF export specialist.
    You handle document export with proper formatting.
    Write tests FIRST (TDD), then implement to pass tests.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent pwa-specialist:
  model: opus
  prompt: |
    You are a Progressive Web App specialist.
    You implement service workers and offline caching.
    Write tests FIRST (TDD), then implement to pass tests.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent font-loader:
  model: opus
  prompt: |
    You are an async font loading specialist.
    You handle network font fetching with caching.
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
    code is formatted, and offline mode works correctly.
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
# Phase 5 Workflow
# =============================================================================

# Step 1: Project Data Model
let project-model = do:
  session persistence-architect:
    prompt: |
      Create src/persistence/project.rs with Project model:

      TDD Requirements (write tests in src/persistence/tests.rs FIRST):
      1. Test: Project::new() creates default project
      2. Test: Project serializes to valid JSON
      3. Test: Project deserializes from JSON
      4. Test: Round-trip preserves all fields
      5. Test: Project includes version for migrations
      6. Test: migrate() handles older versions

      Project struct:
      - version: u32 (current: 1)
      - name: String
      - content: Content
      - typst_source: String
      - template_id: Option<String>
      - image_refs: Vec<ImageRef>
      - created_at: String (ISO 8601)
      - modified_at: String (ISO 8601)
      - metadata: HashMap<String, String>

      ImageRef struct:
      - id: String
      - original_name: String
      - url: String (external URL, not blob)

      Note: Blob URLs are recreated on load from external refs.

      Loop until all tests pass.

  do tdd-cycle
  do lint-pass

# Step 2: Parallel - Save/Load and Export
parallel-do:

  # Agent A: File System Access API
  let file-operations = do:
    session persistence-architect:
      prompt: |
        Create src/persistence/mod.rs with file operations:

        TDD Requirements (write tests FIRST):
        1. Test: save_project() produces valid JSON
        2. Test: load_project() restores Project
        3. Test: save prompts for file location (mock)
        4. Test: load reads from selected file (mock)
        5. Test: handles missing file gracefully
        6. Test: handles corrupted JSON gracefully

        Methods:
        - async save_project(project: &Project) -> Result<(), PersistenceError>
        - async load_project() -> Result<Project, PersistenceError>
        - async save_project_as(project: &Project) -> Result<(), PersistenceError>

        Use File System Access API (showSaveFilePicker, showOpenFilePicker).
        Fallback to download link for browsers without FSA API.

        PersistenceError enum:
        - UserCancelled
        - IoError(String)
        - ParseError(String)
        - VersionError { found: u32, expected: u32 }

        Loop until all tests pass.

    do tdd-cycle
    do lint-pass

  # Agent B: PDF Export
  let pdf-export = do:
    session pdf-exporter:
      prompt: |
        Create src/persistence/export.rs with PDF export:

        TDD Requirements (write tests FIRST):
        1. Test: export_pdf() returns valid PDF bytes
        2. Test: PDF contains rendered content
        3. Test: export triggers download

        Methods:
        - export_pdf(world: &VirtualWorld) -> Result<Vec<u8>, ExportError>
        - trigger_download(bytes: &[u8], filename: &str)

        Use typst::export::pdf for PDF generation.
        Use web_sys to create download link.

        ExportError enum:
        - CompileError(Vec<SourceDiagnostic>)
        - RenderError(String)

        Loop until all tests pass.

    do tdd-cycle
    do lint-pass

# Step 3: Service Worker
let service-worker = do:
  session pwa-specialist:
    prompt: |
      Create sw.js service worker for PWA:

      Requirements:
      1. Cache app shell (index.html, WASM, JS)
      2. Cache embedded fonts
      3. Cache template files
      4. Network-first for API calls (OpenRouter)
      5. Cache-first for static assets
      6. Handle offline gracefully
      7. Update cache on new version

      Cache names:
      - slick-sheet-v1-shell
      - slick-sheet-v1-fonts
      - slick-sheet-v1-templates

      Events to handle:
      - install: precache shell
      - activate: clean old caches
      - fetch: routing strategy

      Also update index.html to register service worker.

  do lint-pass

# Step 4: Async Font Loading
let async-fonts = do:
  session font-loader:
    prompt: |
      Update src/world/fonts.rs with async font loading:

      TDD Requirements (write tests FIRST):
      1. Test: embedded_fonts() returns 2 fonts immediately
      2. Test: fetch_font(url) returns font bytes
      3. Test: FontCache caches fetched fonts
      4. Test: FontCache persists to IndexedDB

      Current: Inter, JetBrains Mono embedded.

      Add async loading for additional fonts:
      - Fetch from CDN (e.g., Google Fonts, Bunny Fonts)
      - Cache in IndexedDB for offline use
      - Update VirtualWorld when fonts load

      FontCache struct:
      - db: IdbDatabase (via web_sys)
      - async get(font_name: &str) -> Option<Vec<u8>>
      - async put(font_name: &str, data: Vec<u8>)

      Methods:
      - async load_additional_fonts(names: &[&str]) -> Vec<Font>
      - is_font_cached(name: &str) -> bool

      Loop until all tests pass.

    do tdd-cycle
    do lint-pass

# Step 5: PWA Manifest
let manifest = do:
  session pwa-specialist:
    prompt: |
      Create manifest.json for PWA:

      Requirements:
      1. App name: "Slick Sheet Studio"
      2. Short name: "SlickSheet"
      3. Theme color and background
      4. Display: standalone
      5. Icons (placeholder, can use SVG)
      6. Start URL
      7. Scope

      Update index.html:
      - Link to manifest
      - Add meta tags for mobile
      - Add apple-touch-icon

      Create simple SVG icon or placeholder PNG.

  do lint-pass

# Step 6: UI Integration
let persistence-ui = do:
  session ui-builder:
    prompt: |
      Create persistence UI components:

      Toolbar buttons:
      1. New - opens template gallery (from Phase 4)
      2. Open - triggers load_project()
      3. Save - triggers save_project()
      4. Save As - triggers save_project_as()
      5. Export PDF - triggers export_pdf()

      Components:
      - Toolbar component with all buttons
      - Keyboard shortcuts (Cmd+S, Cmd+O, Cmd+Shift+S)
      - Unsaved changes indicator (*)
      - Confirm dialog before losing changes

      State:
      - current_file: Option<FileSystemFileHandle>
      - has_unsaved_changes: bool
      - project_name: String

      Integrate into main app layout.

  do lint-pass

# Step 7: Offline Indicator
let offline-ui = do:
  session ui-builder:
    prompt: |
      Create offline status indicator:

      Requirements:
      1. Show "Offline" badge when navigator.onLine is false
      2. Disable AI features when offline
      3. Show "AI unavailable offline" in chat panel
      4. Re-enable when back online
      5. Subtle, non-intrusive UI

      Use online/offline events from window.

  do lint-pass

# Step 8: Final Verification
do:
  session verifier:
    prompt: |
      Verify Phase 5 is COMPLETE:

      Checklist:
      [ ] cargo fmt --check passes
      [ ] cargo clippy -- -D warnings passes
      [ ] cargo test passes (all tests green)
      [ ] Project serialization tests pass
      [ ] File save/load tests pass
      [ ] PDF export tests pass
      [ ] trunk build --release succeeds
      [ ] Service worker registered
      [ ] App works offline (except AI)
      [ ] Save project works (File System Access or download)
      [ ] Load project restores state
      [ ] PDF export downloads file
      [ ] Fonts load (embedded + async)
      [ ] PWA installable
      [ ] Offline indicator shows correctly

      Manual verification:
      1. Open app, create document
      2. Save as .json
      3. Close browser, reopen
      4. Load .json - state restored
      5. Export PDF - file downloads
      6. Go offline (DevTools)
      7. App still works, AI shows disabled
      8. Go online - AI re-enabled

      If ANY check fails:
      1. Identify the failing component
      2. Fix the issue
      3. Re-run ALL checks

      Loop until ALL checks pass.

      Output: "PHASE 5 COMPLETE - PROJECT READY FOR RELEASE" when done.

# =============================================================================
# Exit Criteria
# =============================================================================

# Phase 5 is complete when:
# - Project save/load with JSON format
# - File System Access API or fallback download
# - PDF export working
# - Service worker caches app for offline
# - Async font loading with IndexedDB cache
# - PWA manifest and installable
# - Offline indicator and graceful degradation
# - All tests pass, zero warnings
# - Full manual verification passes
