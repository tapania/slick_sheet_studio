# Phase 4: Assets & Templates
# Goal: Enable custom images and starter templates

# =============================================================================
# Agent Definitions
# =============================================================================

agent asset-handler:
  model: opus
  prompt: |
    You are a browser file/blob handling specialist.
    You work with File API, Blob URLs, and drag-drop.
    Write tests FIRST (TDD), then implement to pass tests.
    Always run @code-simplifier after writing code.
    Run cargo fmt and cargo clippy before finishing.

agent template-designer:
  model: opus
  prompt: |
    You are a Typst template specialist.
    You create professional document templates.
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
    code is formatted, and features work correctly.
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
# Phase 4 Workflow
# =============================================================================

# Step 1: Image Handling
let image-handler = do:
  session asset-handler:
    prompt: |
      Create src/assets/images.rs with image handling:

      TDD Requirements (write tests in src/assets/tests.rs FIRST):
      1. Test: ImageAsset::from_file(file) creates asset
      2. Test: ImageAsset has valid blob_url after creation
      3. Test: ImageAsset::mime_type() returns correct type
      4. Test: ImageManager tracks multiple images
      5. Test: ImageManager::get(id) returns correct asset
      6. Test: ImageManager::remove(id) revokes blob URL

      ImageAsset struct:
      - id: String (uuid or hash)
      - blob_url: String
      - mime_type: String
      - original_name: String
      - size: usize

      ImageManager struct:
      - assets: HashMap<String, ImageAsset>

      Methods:
      - async add_from_file(file: web_sys::File) -> Result<ImageAsset, AssetError>
      - get(id: &str) -> Option<&ImageAsset>
      - remove(id: &str) -> bool
      - inject_into_world(world: &mut VirtualWorld)

      Use web_sys::Url::create_object_url_with_blob for blob URLs.
      Remember to revoke URLs on remove to prevent memory leaks.

      Loop until all tests pass.

  do tdd-cycle
  do lint-pass

# Step 2: Drag-Drop Component
let drag-drop = do:
  session ui-builder:
    prompt: |
      Create src/assets/drop_zone.rs with drag-drop UI:

      Requirements:
      1. DropZone component for image drag-drop
      2. Visual feedback during drag-over
      3. Accept image files only (png, jpg, gif, webp)
      4. Reject non-image files with error message
      5. Callback on successful drop

      Component signature:
      #[component]
      pub fn DropZone(
          on_drop: Callback<web_sys::File>,
          #[prop(optional)] children: Children,
      ) -> impl IntoView

      Use dragover, dragleave, drop events.
      Prevent default to enable drop.

  do lint-pass

# Step 3: Template Typst Files (parallel)
parallel-do:

  # Agent A: Basic templates
  let templates-basic = do:
    session template-designer:
      prompt: |
        Create Typst templates in assets/templates/:

        Templates to create:
        1. product_sheet.typ - Single product showcase
           - Large product image area
           - Title, subtitle, description
           - Specs/features list
           - CTA button area

        2. event_flyer.typ - Event announcement
           - Event title prominent
           - Date/time/location
           - Description
           - RSVP/register CTA

        3. one_pager.typ - Executive summary
           - Logo/header area
           - Key points (3-5 bullets)
           - Supporting details
           - Contact info footer

        4. comparison_chart.typ - Side-by-side
           - Two or three columns
           - Feature rows
           - Checkmarks/values
           - Recommendation highlight

        5. minimal.typ - Clean typography
           - Simple, elegant layout
           - Focus on text hierarchy
           - Minimal decoration

        Each template should:
        - Use cmd:// links for editable fields
        - Have placeholder content
        - Be valid, compilable Typst
        - Look professional

    do lint-pass

  # Agent B: Advanced templates
  let templates-advanced = do:
    session template-designer:
      prompt: |
        Create Typst templates in assets/templates/:

        Templates to create:
        6. case_study.typ - Problem/solution/results
           - Challenge section
           - Solution description
           - Results with metrics
           - Testimonial quote area

        7. team_profile.typ - Staff highlights
           - Photo placeholder
           - Name and title
           - Bio text
           - Contact/social links

        8. pricing_table.typ - Tiered pricing
           - 2-3 pricing tiers
           - Feature list per tier
           - Price display
           - CTA per tier

        9. newsletter.typ - Multi-section
           - Header with logo
           - Multiple article sections
           - Sidebar content
           - Footer with links

        10. infographic.typ - Data visualization
            - Large stat numbers
            - Progress bars/charts (Typst native)
            - Icon placeholders
            - Flow/process layout

        Each template should:
        - Use cmd:// links for editable fields
        - Have placeholder content
        - Be valid, compilable Typst
        - Look professional

    do lint-pass

# Step 4: Template Registry
let template-registry = do:
  session template-designer:
    prompt: |
      Create src/templates/mod.rs with template registry:

      TDD Requirements (write tests FIRST):
      1. Test: Template::load(name) returns template content
      2. Test: TemplateRegistry::list() returns all 10 templates
      3. Test: each template compiles without errors
      4. Test: TemplateInfo has name, description, preview

      Template struct:
      - id: &'static str
      - name: &'static str
      - description: &'static str
      - content: &'static str (embedded with include_str!)
      - category: TemplateCategory

      TemplateCategory enum:
      - Marketing
      - Sales
      - Internal
      - Creative

      TemplateRegistry:
      - list() -> Vec<&Template>
      - get(id: &str) -> Option<&Template>
      - by_category(cat: TemplateCategory) -> Vec<&Template>

      Embed templates using include_str! macro.

      Loop until all tests pass.

  do tdd-cycle
  do lint-pass

# Step 5: Template Gallery UI
let gallery-ui = do:
  session ui-builder:
    prompt: |
      Create src/templates/gallery.rs with gallery UI:

      Requirements:
      1. TemplateGallery component
      2. Grid of template cards
      3. Each card shows: name, description, preview thumbnail
      4. Click to select template
      5. Filter by category (optional)
      6. "Blank" option for empty start

      Component signature:
      #[component]
      pub fn TemplateGallery(
          on_select: Callback<&'static Template>,
      ) -> impl IntoView

      For previews: render mini SVG of each template.
      Can be done lazily or on hover.

  do lint-pass

# Step 6: New Project Flow
let new-project = do:
  session ui-builder:
    prompt: |
      Create new project flow in src/app.rs:

      Requirements:
      1. "New Project" button in toolbar
      2. Opens TemplateGallery modal
      3. On template select:
         a. Load template content
         b. Replace editor content
         c. Inject any template images into VirtualWorld
         d. Close modal
      4. Confirm dialog if existing content will be lost

      Update app state management to handle:
      - current_template: Option<&'static Template>
      - has_unsaved_changes: bool

  do lint-pass

# Step 7: Integration - Images in Editor
let image-integration = do:
  session asset-handler:
    prompt: |
      Integrate images with editor:

      In src/editor/mod.rs:
      1. Add DropZone around preview area
      2. On image drop:
         a. Add to ImageManager
         b. Inject blob URL into VirtualWorld
         c. Insert image reference into Typst code
      3. Show image list in sidebar (optional)
      4. Allow removing images

      In VirtualWorld:
      - Method to add image file: add_image(id: &str, url: &str)
      - Method to resolve image in Typst: #image("blob:...")

  do lint-pass

# Step 8: Final Verification
do:
  session verifier:
    prompt: |
      Verify Phase 4 is COMPLETE:

      Checklist:
      [ ] cargo fmt --check passes
      [ ] cargo clippy -- -D warnings passes
      [ ] cargo test passes (all tests green)
      [ ] ImageAsset/ImageManager tests pass
      [ ] Template loading tests pass
      [ ] All 10 templates compile without errors
      [ ] trunk serve runs without errors
      [ ] Drag-drop image onto preview works
      [ ] Image appears in rendered output
      [ ] Template gallery displays all templates
      [ ] Selecting template updates editor
      [ ] New project flow works end-to-end

      If ANY check fails:
      1. Identify the failing component
      2. Fix the issue
      3. Re-run ALL checks

      Loop until ALL checks pass.

      Output: "PHASE 4 COMPLETE" only when everything passes.

# =============================================================================
# Exit Criteria
# =============================================================================

# Phase 4 is complete when:
# - Image drag-drop creates blob URLs
# - Images inject into VirtualWorld
# - Images render in Typst output
# - 10 template files created and valid
# - Template gallery UI works
# - Template selection loads into editor
# - New project flow with template selection
# - All tests pass, zero warnings
