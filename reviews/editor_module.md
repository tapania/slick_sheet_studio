# Editor Module Documentation

## Overview

The editor module (`src/editor/`) is the primary user interface for Slick Sheet Studio. It implements a split-pane editor with:
- Left pane: Typst source code editor
- Center pane: Live SVG preview with click-to-edit support
- Right pane: AI chat assistant panel
- Top toolbar: File operations, settings, compilation controls
- Bottom status bar: Connection status, save state, project info

**Key Purpose**: Provide an integrated development environment for creating marketing slick sheets using Typst templates with AI assistance.

---

## Architecture Overview

### Module Structure

```
src/editor/
├── mod.rs              (1023 lines) - Main Editor component, layout orchestration
├── state.rs            (179 lines) - EditorState signal management
├── chat_panel.rs       (456 lines) - AI chat interface component
├── edit_modal.rs       (247 lines) - Click-to-edit modal dialog
├── settings_modal.rs   (364 lines) - AI configuration modal
├── status_bar.rs       (165 lines) - Connection & save status display
├── content.rs          (109 lines) - Content data model
├── links.rs            (37 lines)  - cmd:// URL parsing
└── tests.rs            (190 lines) - Unit tests (23 tests)
```

**Total Lines**: ~2,770 lines of code

### File Purpose Summary

| File | Purpose | Key Responsibility |
|------|---------|-------------------|
| `mod.rs` | Editor orchestration | Main component, layout, event handlers |
| `state.rs` | Reactive state | Signal definitions, initialization |
| `chat_panel.rs` | AI interface | Chat messages, AI status, input/output |
| `edit_modal.rs` | Click-to-edit | Field editing dialogs |
| `settings_modal.rs` | Configuration | AI settings persistence |
| `status_bar.rs` | Status display | Online status, save state, project name |
| `content.rs` | Data model | Typst generation, content serialization |
| `links.rs` | URL parsing | cmd:// protocol handling |

---

## Component Hierarchy

### Main Editor Component
```
Editor (main orchestrator)
├── Header/Toolbar
│   ├── Title
│   ├── Actions (New, Open, Save, Export PDF)
│   ├── Auto-preview toggle
│   ├── Refresh button
│   └── Settings button
├── Main Content Area
│   ├── Split Pane
│   │   ├── Code Pane
│   │   │   ├── Pane Header
│   │   │   └── CodeEditor (textarea)
│   │   └── Preview Pane
│   │       ├── Pane Header
│   │       └── Preview (SVG display)
│   └── ChatPanel
│       ├── Header (collapsible)
│       ├── Status Banner (offline/no-api-key)
│       ├── Progress Indicator
│       ├── Chat History
│       └── Input Area
├── StatusBar
│   ├── Status Message
│   ├── Project Name
│   ├── Last Saved
│   └── Connection Status
└── Modals (conditional rendering)
    ├── TemplateGalleryModal
    ├── SettingsModal
    └── EditModal
```

### Leptos Component Definitions

**Main Components:**

1. **Editor** (lines 46-686)
   - Root component
   - Manages all state via signals
   - Handles compile, chat, save/load operations
   - Conditional modal rendering

2. **CodeEditor** (lines 689-719)
   - Simple textarea wrapper
   - Syntax highlighting support (monospace font, spellcheck disabled)
   - Debounced input handling via parent

3. **TemplateGalleryModal** (lines 722-749)
   - Grid-based template selection
   - Displays template name, description, category
   - Click-to-select callback

4. **Preview** (lines 938-1022)
   - Renders SVG output or error message
   - Click handler for cmd:// link interception
   - Responsive container with box shadow

**Sub-components:**

5. **ChatPanel** (chat_panel.rs, lines 110-456)
   - Collapsible panel (300px when expanded, 40px when collapsed)
   - Status banners for offline/no-api-key
   - Processing progress indicator
   - Message history with type-based styling
   - Textarea input with send button
   - Disabled during processing

6. **SettingsModal** (settings_modal.rs, lines 108-363)
   - API key input (password masked)
   - Model selection dropdown
   - Max iterations slider (1-10)
   - Save/Cancel buttons
   - Keyboard escape support

7. **EditModal** (edit_modal.rs, lines 34-222)
   - Dynamic field type handling (SingleLine, MultiLine, Url)
   - Autofocus on open
   - Enter key to save (single-line), Escape to close
   - Save/Cancel buttons

8. **StatusBar** (status_bar.rs, lines 30-125)
   - Single-line footer display
   - Responsive layout with flexbox
   - Online/offline indicator with emoji
   - Connection status color coding

---

## Signal Patterns & State Management

### EditorState (state.rs)

```rust
pub struct EditorState {
    pub content_data: RwSignal<SlickSheetData>,      // JSON content
    pub template_source: RwSignal<String>,            // Handlebars template
    pub typst_source: RwSignal<String>,               // Compiled Typst code
    pub svg_output: RwSignal<Option<String>>,         // SVG result
    pub error: RwSignal<Option<String>>,              // Compilation error
    pub auto_preview: RwSignal<bool>,                 // Auto-compile toggle
}
```

**Key Pattern: RwSignal<T>**
- Used for state that needs read/write from multiple places
- Allows updates via `.set()` and `.update()`
- Reactive: Components re-render on changes

### Editor Component Signals (mod.rs)

```rust
// Content signals
let typst_source = state.typst_source;           // RwSignal<String>
let svg_output = state.svg_output;               // RwSignal<Option<String>>
let error = state.error;                         // RwSignal<Option<String>>
let auto_preview = state.auto_preview;           // RwSignal<bool>

// UI state signals
let show_template_gallery = create_rw_signal(false);
let show_settings_modal = create_rw_signal(false);
let show_edit_modal = create_rw_signal(Option::<EditFieldData>::None);
let project_name = create_rw_signal("Untitled Project".to_string());
let status_message = create_rw_signal(Option::<String>::None);
let last_saved = create_rw_signal(Option::<String>::None);

// AI signals
let ai_settings = create_rw_signal(AiSettings::load());
let has_api_key = create_memo(move |_| ai_settings.get().has_api_key());

// Connection signals
let connection_status = use_online_status();
let is_online = create_memo(move |_| connection_status.get().is_online());

// Chat signals
let chat_collapsed = create_rw_signal(false);
let chat_messages = create_rw_signal(Vec::<ChatMessage>::new());
let processing_state = create_rw_signal(AiProcessingState::Ready);
let current_iteration = create_rw_signal(0_usize);
let max_iterations_signal = create_memo(move |_| ai_settings.get().max_iterations as usize);
```

**Signal Type Patterns:**

1. **RwSignal<T>** - Mutable state
   - `auto_preview`, `typst_source`, `project_name`
   - Allows both read and write from multiple components

2. **Signal<T>** - Read-only derived signal
   - Used to pass to child components
   - Converted via `.into()` operator

3. **Memo<T>** - Computed/derived signals
   - `has_api_key`: Computed from `ai_settings`
   - `is_online`: Computed from `connection_status`
   - `max_iterations_signal`: Computed from AI settings
   - Only recompute when dependencies change

### Signal Mutation Patterns

```rust
// Read current value
let value = signal.get();

// Set new value
signal.set(new_value);

// Update existing value
signal.update(|v| *v = new_value);
```

---

## Event Handlers & Callbacks

### Key Event Handlers

1. **on_source_change** (lines 101-125)
   - Called on textarea input change
   - Updates `typst_source` signal
   - If auto-preview enabled: debounce (300ms) and compile
   - Uses `web_sys::window().set_timeout()` for debouncing
   - Clears previous timeout to avoid multiple compiles

2. **compile** (lines 81-92)
   - Calls `VirtualWorld::compile_to_svg()`
   - On success: sets `svg_output`, clears `error`
   - On failure: sets `error` message, clears `svg_output`

3. **on_template_select** (lines 128-132)
   - Callback passed to TemplateGalleryModal
   - Sets `typst_source`, closes modal
   - Triggers compile

4. **on_settings_save** (lines 135-140)
   - Persists AI settings to localStorage via `AiSettings::save()`
   - Updates `ai_settings` signal
   - Shows status message with auto-clear (3s)

5. **on_edit_save** (lines 143-151)
   - Receives `(field_id, new_value)` tuple
   - Updates field in source via `update_field_in_source()`
   - Triggers compile
   - Closes edit modal

6. **on_save** (lines 154-172)
   - Creates JSON project file via `Project::from_source()`
   - Triggers download via `trigger_download()`
   - Updates `last_saved` timestamp
   - Shows success/error message

7. **on_load** (lines 175-188)
   - Triggers file picker
   - Parses JSON via `Project::from_json()`
   - Updates `project_name` and `typst_source`
   - Triggers compile
   - Shows success/error message

8. **on_export_pdf** (lines 191-203)
   - Calls `pdf_data_url()` from persistence module
   - Triggers download from data URL
   - Shows success/error message

9. **on_chat_send** (lines 206-306)
   - Complex async callback
   - Adds user message to `chat_messages`
   - Sets `processing_state` to `Generating`
   - Spawns async task with `spawn_local()`
   - Creates OpenRouter client and AgentLoop
   - Handles AgentResult: Success, MaxIterationsReached, Error
   - Updates source, SVG, and messages
   - Resets processing state after 1s delay

10. **on_preview_click** (lines 309-356)
    - Event delegation: walks up DOM to find `<a>` element
    - Parses `href` attribute for `cmd://` protocol
    - Calls `parse_cmd_url()` to extract edit command
    - Prevents default link behavior
    - Extracts current field value from source
    - Opens EditModal with field data

### Callback Pattern

```rust
// Callback<T> - Event handler
let on_chat_send = Callback::new(move |prompt: String| {
    // Handle prompt...
});

// Pass to child
<ChatPanel on_send=on_chat_send />

// Call from child
on_send.call(prompt_value);
```

---

## Integration with Other Modules

### World Module Integration (Typst Compilation)

```rust
use crate::world::VirtualWorld;

// In compile function
match VirtualWorld::compile_to_svg(&source) {
    Ok(svg) => svg_output.set(Some(svg)),
    Err(errors) => error.set(Some(errors.join("\n"))),
}
```

**Purpose**: Compiles Typst code to SVG in WASM

### AI Module Integration (Agent Loop)

```rust
use crate::ai::{AgentConfig, AgentLoop, OpenRouterClient, OpenRouterConfig};

// In on_chat_send
let config = OpenRouterConfig::with_key(settings.api_key.clone());
let client = OpenRouterClient::new(config);
let agent_config = AgentConfig {
    max_iterations: settings.max_iterations as usize,
    model: settings.model.clone(),
    enable_visual_verification: false,
};
let mut agent = AgentLoop::new(client, agent_config);
let result = agent.run(&prompt, Some(&source), compile_fn).await;
```

**Purpose**: AI-powered code generation via agent loop

### Persistence Module Integration

```rust
use crate::persistence::{pdf_data_url, Project};

// Load/Save projects
let project = Project::from_source(project_name.get(), typst_source.get());
let json = project.to_json_pretty()?;
trigger_download(&json, &format!("{}.json", project_name.get()), "application/json");

// PDF export
let data_url = pdf_data_url(&source)?;
trigger_download_url(&data_url, &format!("{}.pdf", project_name.get()));
```

**Purpose**: Project serialization and PDF rendering

### Templates Module Integration

```rust
use crate::templates::TEMPLATES;

// In TemplateGalleryModal
{TEMPLATES.iter().map(|template| {
    // Render template card
}).collect::<Vec<_>>()}
```

**Purpose**: Built-in template gallery

---

## Click-to-Edit System

### Architecture

**Flow**: User click -> cmd:// link interception -> field extraction -> modal edit -> source update

### Implementation

1. **cmd:// URL Scheme** (links.rs)
```rust
pub enum EditCommand {
    Title,                    // cmd://edit/title
    Subtitle,                 // cmd://edit/subtitle
    Body,                     // cmd://edit/body
    Image,                    // cmd://edit/image
    Metadata(String),         // cmd://edit/meta/{key}
}

pub fn parse_cmd_url(url: &str) -> Option<EditCommand>
```

2. **Link Generation** (content.rs)
```rust
// In Content::to_typst()
format!("= #link(\"cmd://edit/title\")[{}]", escape_typst(&self.title))
```

3. **Click Interception** (mod.rs, lines 309-356)
```rust
let on_preview_click = move |ev: web_sys::MouseEvent| {
    // Walk up DOM to find <a> element
    if let Some(href) = el.get_attribute("href") {
        if let Some(cmd) = parse_cmd_url(&href) {
            ev.prevent_default();
            // Extract field value
            let current_value = extract_field_value(&source, field_id);
            // Show edit modal
            show_edit_modal.set(Some(EditFieldData { ... }));
        }
    }
};
```

4. **Field Extraction** (mod.rs, lines 875-893)
```rust
fn extract_field_value(source: &str, field_id: &str) -> String {
    let pattern = format!("cmd://edit/{}", field_id);
    // Search for line containing pattern
    // Extract content between ][ and next ]
}
```

5. **Field Update** (mod.rs, lines 896-935)
```rust
fn update_field_in_source(source: &str, field_id: &str, new_value: &str) -> Option<String> {
    let pattern = format!("cmd://edit/{}", field_id);
    // Find line with pattern
    // Replace content between ][ and ]
    // Return modified source
}
```

### Limitations
- Simple string-based extraction (not full AST parsing)
- Works with templated Typst patterns but fragile with custom layouts
- Only handles standard field structure

---

## AI Settings System

### Storage (settings_modal.rs)

```rust
pub struct AiSettings {
    pub api_key: String,
    pub model: String,
    pub max_iterations: u8,
}

impl AiSettings {
    pub fn load() -> Self {
        // Load from localStorage: "slick_ai_api_key", "slick_ai_model", "slick_ai_max_iterations"
    }

    pub fn save(&self) {
        // Persist to localStorage
    }
}
```

### Available Models

```rust
pub const AI_MODELS: &[(&str, &str, &str)] = &[
    ("google/gemini-3-flash", "Gemini 3 Flash", "Fast & Cost-Effective"),
    ("anthropic/claude-4.5-haiku", "Claude 4.5 Haiku", "Balanced Quality"),
    ("openai/gpt-5.2-mini", "GPT-5.2 Mini", "Alternative"),
    ("anthropic/claude-sonnet-4", "Claude Sonnet 4", "Best Quality"),
];
```

### Settings Modal Features
- Password-masked API key input
- Model dropdown selector
- Max iterations slider (1-10)
- Escape key to close
- Persistent storage via localStorage
- Link to OpenRouter key creation

---

## Chat Panel System

### Processing States (chat_panel.rs)

```rust
pub enum AiProcessingState {
    Ready,                  // Idle
    Generating,             // LLM call in progress
    Compiling,              // Typst compilation
    Verifying,              // Visual verification (planned)
    Complete,               // Success
    Failed,                 // Error
}

impl AiProcessingState {
    pub fn is_processing(&self) -> bool {
        matches!(self, Self::Generating | Self::Compiling | Self::Verifying)
    }
}
```

### Message Types

```rust
pub enum ChatMessageType {
    User,       // User prompt
    Assistant,  // AI response
    System,     // Status/progress
    Error,      // Error message
}

pub struct ChatMessage {
    pub message_type: ChatMessageType,
    pub content: String,
}
```

### Chat Panel Features
- Collapsible (300px -> 40px)
- Status banners: offline warning, missing API key warning
- Processing progress with iteration counter
- Message history with type-based styling
- Textarea input (Shift+Enter for newline, Enter to send)
- Disabled state during processing
- Auto-scroll via flex layout

---

## File Operations

### Save (Project Export)
```
trigger_download()
  -> Create Blob from JSON
  -> Generate object URL
  -> Create invisible <a> element
  -> Set href + download attribute
  -> Click to trigger save dialog
  -> Revoke object URL
```

### Load (Project Import)
```
trigger_file_load()
  -> Create <input type="file">
  -> Listen for change event
  -> Read file as text via FileReader
  -> Parse JSON via Project::from_json()
  -> Update editor state
```

### PDF Export
```
pdf_data_url()
  -> Compile Typst to PDF (via persistence module)
  -> Create base64 data URL
  -> trigger_download_url()
  -> Same as save flow with data URL
```

---

## Styling System

### CSS Architecture
- Scoped styles within Leptos view! macros
- CSS custom properties (CSS variables):
  - `--bg-primary`, `--bg-secondary`, `--bg-tertiary`
  - `--text-primary`, `--text-secondary`
  - `--border`, `--accent`, `--accent-hover`
  - `--error`, `--warning`, `--success`
- Flexbox-based responsive layout
- Z-index layering: 0 (normal), 1000 (modals), 1100 (edit modal)

### Key Style Classes
```css
.editor-container          /* Root flex container */
.toolbar                   /* Top action bar */
.main-content              /* Split pane area */
.split-pane                /* Left/right panes */
.code-pane, .preview-pane  /* Individual panes */
.chat-panel                /* AI panel (collapsible) */
.modal-overlay             /* Dark background */
.modal-content             /* Dialog box */
.btn, .btn-primary, .btn-secondary  /* Buttons */
```

---

## Status Bar Features

### Online Status Detection (status_bar.rs, lines 128-164)

```rust
pub fn use_online_status() -> RwSignal<ConnectionStatus> {
    // Check navigator.onLine
    // Listen for "online" and "offline" events
    // Update signal on status change
}
```

**Implementation**:
- Uses `js_sys::Reflect` to check `navigator.onLine`
- Sets up event listeners for browser online/offline events
- Updates signal on status change
- Green/red indicator emoji display

### Status Bar Display
```
[Status Message] | Project Name | Saved: HH:MM:SS | [Online/Offline]
```

---

## Tests (tests.rs)

### Test Coverage: 23 tests across 3 categories

#### 1. Content Model Tests (9 tests)
- `test_content_new_creates_default` - Default values
- `test_content_serializes_to_json` - JSON serialization
- `test_content_deserializes_from_json` - JSON deserialization
- `test_content_roundtrip` - Serialize -> deserialize
- `test_content_to_typst_generates_valid_markup` - Typst generation
- `test_content_to_typst_includes_subtitle` - Optional field inclusion
- `test_content_to_typst_includes_metadata` - Metadata rendering
- `test_content_to_typst_escapes_special_chars` - Escaping verification
- (Additional metadata test)

#### 2. Link Parsing Tests (8 tests)
- `test_parse_cmd_url_title` - cmd://edit/title
- `test_parse_cmd_url_subtitle` - cmd://edit/subtitle
- `test_parse_cmd_url_body` - cmd://edit/body
- `test_parse_cmd_url_image` - cmd://edit/image
- `test_parse_cmd_url_metadata` - cmd://edit/meta/key
- `test_parse_cmd_url_https_returns_none` - Non-cmd URLs
- `test_parse_cmd_url_invalid_cmd_returns_none` - Invalid format
- `test_parse_cmd_url_empty_returns_none` - Empty string

#### 3. EditCommand Tests (6 tests)
- `test_edit_command_covers_all_fields` - Enum completeness
- `test_edit_command_clone` - Clone implementation
- `test_edit_command_debug` - Debug output
- (Additional derive trait tests)

### Test Quality
- **Coverage Focus**: Data model and URL parsing (core logic)
- **Limitations**: No component tests (Leptos component tests require browser context)
- **Best Practices**: Tests use standard assertions and are isolated

---

## Content Model (content.rs)

### Content Structure

```rust
pub struct Content {
    pub title: String,
    pub subtitle: Option<String>,
    pub body: String,
    pub image_url: Option<String>,
    pub metadata: HashMap<String, String>,
}
```

### Typst Code Generation

The `to_typst()` method generates Typst code with:
```
#set page(...)          // Page setup
#set text(...)          // Font setup
= #link("cmd://edit/title")[Title]
_#link("cmd://edit/subtitle")[Subtitle]_
#link("cmd://edit/body")[Body]
#link("cmd://edit/image")[#image(...)]
== Details             // Metadata section
- *Key*: #link("cmd://edit/meta/key")[Value]
```

### Escaping

Special Typst characters are escaped:
- `\` -> `\\`
- `#` -> `\#`
- `*` -> `\*`
- `_` -> `\_`
- `[`, `]` -> `\[`, `\]`
- `"` -> `\"`

---

## Performance Considerations

### 1. Debounced Compilation
```rust
if auto_preview.get() {
    // Clear existing timeout
    if let Some(handle) = debounce_handle.get() {
        window.clear_timeout_with_handle(handle);
    }
    // Set new timeout (300ms)
    window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        300,
    )
}
```
**Benefit**: Prevents excessive compilation during rapid typing

### 2. Signal Memoization
```rust
let is_online = create_memo(move |_| connection_status.get().is_online());
```
**Benefit**: Only recomputes when dependency changes

### 3. Conditional Rendering
```rust
{move || show_template_gallery.get().then(|| view! { ... })}
```
**Benefit**: Modals only render when needed

### 4. Local Storage Caching
```rust
let ai_settings = create_rw_signal(AiSettings::load());
```
**Benefit**: Settings persist across sessions without API calls

---

## Known Limitations & TODOs

1. **Simplified Field Extraction**
   - Uses string matching, not AST parsing
   - Fragile with non-standard layouts
   - Could be improved with proper Typst AST parsing

2. **Visual Verification (Planned)**
   - `AiProcessingState::Verifying` state exists but unused
   - Agent config has `enable_visual_verification: false`
   - Planned feature for visual output validation

3. **Component Testing**
   - No Leptos component tests (browser context required)
   - Snapshot tests not implemented
   - Could benefit from integration tests

4. **Modal Z-index Management**
   - Edit modal (1100) above settings modal (1100)
   - Could cause layering issues if both open
   - Should use stack-based z-index system

5. **File Loading Error Handling**
   - Limited error messages for corrupted files
   - FileReader errors not fully handled
   - Could improve UX with detailed error reporting

---

## Integration Examples

### Adding a New Editable Field

1. **Add to Content Model** (content.rs)
   ```rust
   pub struct Content {
       pub new_field: String,
       // ...
   }
   ```

2. **Generate Edit Link** (content.rs, to_typst)
   ```rust
   parts.push(format!(
       "#link(\"cmd://edit/new_field\")[{}]",
       escape_typst(&self.new_field)
   ));
   ```

3. **Add EditCommand Variant** (links.rs)
   ```rust
   pub enum EditCommand {
       NewField,  // New variant
       // ...
   }
   ```

4. **Parse the URL** (links.rs, parse_cmd_url)
   ```rust
   match parts.as_slice() {
       ["edit", "new_field"] => Some(EditCommand::NewField),
       // ...
   }
   ```

5. **Handle Field Label** (edit_modal.rs, get_field_label)
   ```rust
   pub fn get_field_label(field_id: &str) -> String {
       match field_id {
           "new_field" => "New Field".to_string(),
           // ...
       }
   }
   ```

6. **Handle Field Type** (edit_modal.rs, get_field_type)
   ```rust
   pub fn get_field_type(field_id: &str) -> EditFieldType {
       match field_id {
           "new_field" => EditFieldType::SingleLine,  // or MultiLine/Url
           // ...
       }
   }
   ```

7. **Extract Value** (mod.rs, extract_field_value)
   ```rust
   let new_field_pattern = "cmd://edit/new_field";
   // Add to pattern matching
   ```

8. **Update Value** (mod.rs, update_field_in_source)
   ```rust
   // Add new_field handling to replacement logic
   ```

---

## Security Considerations

1. **localStorage Exposure**
   - API keys stored in plaintext in localStorage
   - Vulnerable on shared devices
   - Consider encrypted storage or session-only keys

2. **XSS via SVG**
   - SVG content inserted via `inner_html`
   - Typst-svg output is trusted, but worth auditing
   - Could sanitize SVG before display

3. **File Input Validation**
   - JSON parsing errors handled but limited validation
   - No file size limits
   - Could add validation for malicious files

4. **cmd:// Protocol**
   - Custom protocol for internal links
   - Safe from external URLs (only works with relative links)
   - Good security by design

---

## Summary

The editor module is a well-structured Leptos application that demonstrates:
- **Clean component hierarchy**: Nested components with clear responsibilities
- **Reactive state management**: Proper use of signals, memos, and callbacks
- **Event-driven architecture**: Robust event handling with debouncing and error recovery
- **Integration patterns**: Clean APIs to world, AI, and persistence modules
- **User experience**: Click-to-edit, auto-preview, AI assistance, offline support
- **Test coverage**: Focused tests on core logic (data and parsing)

**Strengths**:
- Comprehensive feature set for a single module
- Good separation of concerns
- Leptos signal patterns follow best practices
- Extensive documentation in code
- Smooth user experience with debouncing and status feedback

**Areas for Improvement**:
- Add component integration tests
- Implement visual verification for AI
- Use AST-based field extraction instead of string matching
- Add file size limits and validation
- Secure API key storage

This module serves as the bridge between user input and the backend compilation/AI systems, making it critical for the application's functionality and user experience.
