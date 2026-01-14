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

agent browser-tester:
  model: opus
  prompt: |
    You are a browser automation testing specialist.
    You use @dev-browser to verify PWA behavior, offline mode,
    file operations, and service worker functionality.
    Always run @code-simplifier after writing code.

agent verifier:
  model: opus
  prompt: |
    You are a strict build and test verifier.
    You ensure all tests pass, clippy has zero warnings,
    code is formatted, and offline mode works correctly.
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

block pwa-test:
  session: browser-tester
    prompt: |
      Use @dev-browser to test PWA functionality:
      1. Check service worker registration
      2. Verify caching behavior
      3. Test offline mode
      4. Verify PWA installability
  session: browser-tester
    prompt: "Take screenshots and validate PWA requirements"

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

# Step 8: Browser Testing - File Operations
let browser-file-tests = do:
  session browser-tester:
    prompt: |
      Start trunk serve in background, then use @dev-browser to test:

      File Operation Test Scenarios:
      1. Navigate to http://localhost:8080
      2. Create a new document (type in editor)
      3. Verify unsaved changes indicator (*) appears
      4. Click Save button
      5. Verify file picker dialog appears (or download starts)
      6. Save file as "test-project.json"
      7. Verify unsaved indicator disappears
      8. Make changes to document
      9. Use Cmd+S keyboard shortcut
      10. Verify save works
      11. Click New button
      12. Verify confirm dialog appears (unsaved changes)
      13. Cancel, then Save, then New
      14. Click Open button
      15. Load the "test-project.json"
      16. Verify document state restored
      17. Take screenshots at each step

      Use @dev-browser for all browser interactions.
      Loop until all file operation tests pass.

  do browser-test

# Step 9: Browser Testing - PDF Export
let browser-pdf-tests = do:
  session browser-tester:
    prompt: |
      Use @dev-browser to test PDF export:

      PDF Export Test Scenarios:
      1. Navigate to http://localhost:8080
      2. Create or load a document
      3. Click Export PDF button
      4. Verify download starts
      5. Check downloaded file is valid PDF
      6. Take screenshot of export process

      Use @dev-browser for all browser interactions.
      Loop until PDF export tests pass.

  do browser-test

# Step 10: Browser Testing - PWA & Offline
let browser-pwa-tests = do:
  session browser-tester:
    prompt: |
      Use @dev-browser to test PWA and offline functionality:

      PWA Test Scenarios:
      1. Navigate to http://localhost:8080
      2. Open DevTools → Application → Service Workers
      3. Verify service worker is registered and active
      4. Check cached resources in Cache Storage
      5. Verify manifest.json is detected
      6. Check PWA install prompt availability
      7. Take screenshot of service worker status

      Offline Test Scenarios:
      1. With app loaded, go to DevTools → Network
      2. Enable "Offline" mode
      3. Refresh the page
      4. Verify app loads from cache
      5. Verify offline indicator appears
      6. Verify AI chat shows "unavailable offline"
      7. Try to type in editor - should work
      8. Try to send AI prompt - should show offline error
      9. Disable "Offline" mode
      10. Verify online status restored
      11. Verify AI features re-enabled
      12. Take screenshots of offline and online states

      Font Caching Test:
      1. Clear IndexedDB
      2. Reload app
      3. Verify embedded fonts work immediately
      4. Request additional font
      5. Verify font loads and caches
      6. Go offline
      7. Verify cached font still works

      Use @dev-browser for all browser interactions.
      Loop until all PWA and offline tests pass.

  do pwa-test

# Step 11: Full End-to-End Test
let e2e-test = do:
  session browser-tester:
    prompt: |
      Use @dev-browser for complete end-to-end workflow test:

      Full Workflow:
      1. Navigate to http://localhost:8080
      2. Click New → Select template from gallery
      3. Edit content in editor
      4. Use AI to modify layout (if online and API key set)
      5. Add an image via drag-drop
      6. Save project as JSON
      7. Export as PDF
      8. Close and reopen browser
      9. Load saved project
      10. Verify all state restored correctly
      11. Go offline
      12. Make edits
      13. Save project (should work offline)
      14. Go online
      15. Use AI features again
      16. Take final screenshot

      This is the complete user journey test.
      Loop until full workflow passes.

  do browser-test

# Step 12: Final Verification
do:
  session verifier:
    prompt: |
      Verify Phase 5 is COMPLETE:

      Automated Checks:
      [ ] cargo fmt --check passes
      [ ] cargo clippy -- -D warnings passes
      [ ] cargo test passes (all tests green)
      [ ] Project serialization tests pass
      [ ] File save/load tests pass
      [ ] PDF export tests pass
      [ ] trunk build --release succeeds

      Browser Verification (use @dev-browser):
      [ ] Navigate to http://localhost:8080
      [ ] Service worker registered and active
      [ ] Manifest.json detected
      [ ] PWA installable
      [ ] Save project works (FSA or download)
      [ ] Load project restores state
      [ ] Save As creates new file
      [ ] Keyboard shortcuts work (Cmd+S, Cmd+O)
      [ ] Unsaved changes indicator works
      [ ] PDF export downloads valid file
      [ ] Fonts load (embedded + async cached)
      [ ] Offline indicator shows when offline
      [ ] App works offline (except AI)
      [ ] AI disabled offline, re-enabled online
      [ ] Take final screenshots for documentation

      Full E2E Workflow (use @dev-browser):
      [ ] New → Template → Edit → AI → Save → Export → Reload → Load
      [ ] Offline editing and saving works
      [ ] Complete user journey successful

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
# - All unit tests pass, zero warnings
# - All @dev-browser tests pass:
#   - File operations
#   - PDF export
#   - PWA/Service worker
#   - Offline mode
#   - Full E2E workflow
# - Project ready for production release
