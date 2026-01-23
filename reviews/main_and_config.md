# Main Application Bootstrap and Configuration Review

## Document Overview
This document details how the Slick Sheet Studio application bootstraps, initializes its components, and handles global configuration through Cargo.toml and Trunk.toml.

## Application Architecture

### Technology Stack

**Language & Framework:**
- **Language**: Rust
- **Web Framework**: Leptos 0.6 with CSR (Client-Side Rendering) feature
- **WASM Target**: wasm32-unknown-unknown via Trunk

**Key Runtime Dependencies:**
- `leptos` (0.6) - Reactive UI framework with CSR feature only
- `wasm-bindgen` (0.2) - JavaScript/Rust FFI bindings
- `web-sys` (0.3) - Low-level browser APIs via bindings
- `js-sys` (0.3) - JavaScript intrinsics and object manipulation
- `typst` (0.12) - Document/markup compiler core
- `typst-pdf` (0.12) - PDF export capability
- `typst-svg` (0.12) - SVG rendering from Typst documents
- `comemo` (0.4) - Caching and memoization infrastructure

---

## Bootstrap Process

### Entry Point: `src/main.rs`

The application follows a minimal WASM bootstrap pattern:

```rust
fn main() {
    // Initialize tracing for WASM
    tracing_wasm::set_as_global_default();

    // Mount the app to the #app element, replacing the loading state
    let document = web_sys::window()
        .expect("window")
        .document()
        .expect("document");
    let app_element = document
        .get_element_by_id("app")
        .expect("app element")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("app should be HtmlElement");

    // Clear the loading content
    app_element.set_inner_html("");

    leptos::mount_to(app_element, editor::Editor);
}
```

**Bootstrap Steps:**

1. **Logging Initialization**
   - Calls `tracing_wasm::set_as_global_default()` to enable browser console logging via the `tracing` crate
   - Allows log output to browser developer console
   - Critical for debugging WASM execution

2. **DOM Element Acquisition**
   - Obtains reference to the browser's window object
   - Retrieves the document from the window
   - Locates the HTML element with id `app`
   - Type-casts the element to `HtmlElement` using `dyn_into::<web_sys::HtmlElement>()`

3. **Loading State Cleanup**
   - Clears the inner HTML of the #app element (removing the loading spinner and "Loading..." message)
   - Prepares the DOM for Leptos component mounting

4. **Component Mounting**
   - Mounts the `editor::Editor` component to the #app element
   - This is the single root component for the entire application
   - Leptos handles all subsequent DOM management and reactivity

---

## HTML Initialization: `index.html`

### Document Structure

The HTML file serves as the WASM bootstrap host:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <!-- Metadata for PWA and mobile compatibility -->
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="Create professional marketing slick sheets with Typst and AI assistance">
    <meta name="theme-color" content="#e94560">
    <meta name="apple-mobile-web-app-capable" content="yes">
    <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
    <meta name="apple-mobile-web-app-title" content="SlickSheet">

    <!-- PWA Manifest and Icons -->
    <link rel="manifest" href="/manifest.json">
    <link rel="apple-touch-icon" href="/assets/icon-192.svg">

    <!-- Trunk Build Directives -->
    <link data-trunk rel="rust" data-wasm-opt="z" />
    <link data-trunk rel="copy-dir" href="assets" />
    <link data-trunk rel="copy-file" href="manifest.json" />
    <link data-trunk rel="copy-file" href="sw.js" />
```

**Key Initialization Features:**

1. **PWA Support**
   - Manifest file linked for installability
   - Apple-specific web app meta tags for iOS home screen support
   - Theme color set to brand primary (`#e94560`)
   - Icons configured for both standard and "maskable" purposes (192x192 and 512x512)

2. **Trunk Build System Integration**
   - `data-trunk rel="rust"` triggers WASM compilation with size optimization (`data-wasm-opt="z"`)
   - `data-trunk rel="copy-dir" href="assets"` copies font and template assets to dist
   - `data-trunk rel="copy-file"` directives include manifest.json and service worker

3. **Styling**
   - CSS custom properties (CSS variables) define the color scheme
   - Dark theme with accent color `#e94560` (magenta-red)
   - Loading spinner animation with rotating border
   - Base styles set margin/padding to 0, box-sizing to border-box

4. **Service Worker Registration**
   - Script registers `/sw.js` for offline support and PWA caching
   - Fires on window load event
   - Logs registration result to console

### Root DOM Element

```html
<div id="app">
    <div class="loading">
        <div class="loading-spinner"></div>
        <p>Loading Slick Sheet Studio...</p>
    </div>
</div>
```

The #app element contains a loading state that is replaced by the Leptos Editor component during bootstrap.

---

## Build Configuration: `Cargo.toml`

### Package Metadata

```toml
[package]
name = "slick_sheet_studio"
version = "0.1.0"
edition = "2021"
description = "A high-performance web application for creating marketing slick sheets with Typst"
license = "MIT"
```

### Core Dependencies by Category

#### Framework & DOM (Web System)
- `leptos = { version = "0.6", features = ["csr"] }` - CSR-only (no SSR)
- `wasm-bindgen = "0.2"` - JavaScript FFI
- `web-sys = { version = "0.3", features = [...] }` - Browser API bindings (44 features)
- `js-sys = "0.3"` - JavaScript object and function bindings

**web-sys Features Enabled:**
- Console API (logging)
- Document/Element/Node APIs (DOM manipulation)
- HTML element types (textarea, input, div)
- Window object and navigation
- Event system (keyboard, mouse, drag)
- File system access (File, FileList, FileReader, Blob, DataTransfer)
- Storage API (localStorage, etc.)
- Canvas 2D rendering context
- History and Location APIs

#### Document Processing (Typst)
- `typst = "0.12"` - Core Typst markup compiler
- `typst-pdf = "0.12"` - PDF generation
- `typst-svg = "0.12"` - SVG rendering backend
- `comemo = "0.4"` - Caching layer for Typst compilation

#### Data Serialization
- `serde = { version = "1.0", features = ["derive"] }` - Serialization framework
- `serde_json = "1.0"` - JSON codec

#### Async & Networking
- `futures = "0.3"` - Async utilities (futures, streams, combinators)
- `reqwest = { version = "0.12", default-features = false, features = ["json"] }` - HTTP client (JSON only, no default features)
- `gloo-net = "0.6"` - High-level browser networking
- `gloo-timers = "0.3"` - setTimeout/setInterval wrappers

#### Utility Crates
- `thiserror = "1.0"` - Error type derivation
- `tracing = "0.1"` - Structured logging framework
- `tracing-wasm = "0.2"` - Browser console output for tracing
- `base64 = "0.22"` - Base64 encoding/decoding
- `chrono = { version = "0.4", default-features = false, features = ["wasmbind", "clock"] }` - Time handling (WASM-specific)
- `uuid = { version = "1.0", features = ["v4", "js"] }` - UUID generation (v4 and JavaScript object support)

### Development Dependencies

```toml
[dev-dependencies]
wasm-bindgen-test = "0.3"
```

Only test utilities are included in dev dependencies. This is a WASM-specific test runner.

### Release Profile Optimization

```toml
[profile.release]
opt-level = "z"      # Optimize for code size (smallest binary)
lto = true           # Link-Time Optimization enabled
codegen-units = 1    # Single codegen unit for maximum optimization (longer compilation)
```

**Rationale:**
- `opt-level = "z"` produces the smallest WASM binary possible
- `lto = true` enables inter-procedural optimization
- `codegen-units = 1` combines all Rust code into a single compilation unit, allowing better cross-unit optimization at the cost of compile time
- These settings are critical for WASM size (users download the .wasm file)

---

## Trunk Build System: `Trunk.toml`

### Build Configuration

```toml
[build]
target = "index.html"     # HTML file to process
dist = "dist"             # Output directory

[watch]
watch = ["src", "index.html", "assets"]  # Directories to watch for changes

[serve]
address = "127.0.0.1"     # Dev server binds to localhost only
port = 8080               # Standard port for local development
open = false              # Don't auto-open browser
```

**Build Process:**

1. **Target**: Processes `index.html` as the entry point
2. **Output**: Generates optimized WASM and assets in `dist/`
3. **Inline Assets**: The `data-trunk` directives in `index.html` control what gets copied:
   - Rust source compiled to WASM
   - `assets/` directory (fonts, images) copied to dist
   - `manifest.json` for PWA
   - `sw.js` service worker for offline support
4. **Watch Mode**: Monitors changes in src, index.html, and assets directories
5. **Dev Server**: Runs on `127.0.0.1:8080` (localhost only, not network-accessible)

---

## Root Component: `src/editor/Editor`

The `Editor` component is mounted to #app and serves as the application root:

### Component Structure

Located in `src/editor/mod.rs`, the Editor component initializes:

1. **Core State** (`EditorState`)
   - `typst_source: RwSignal<String>` - Raw Typst markup
   - `svg_output: RwSignal<Option<String>>` - Compiled SVG
   - `error: RwSignal<Option<String>>` - Compilation errors
   - `auto_preview: RwSignal<bool>` - Live preview toggle

2. **Content Data**
   - `content_data: RwSignal<SlickSheetData>` - JSON-structured marketing content
   - `template_source: RwSignal<String>` - Handlebars template with placeholders

3. **UI State Signals**
   - `show_template_gallery` - Template selection modal
   - `show_settings_modal` - AI configuration dialog
   - `show_edit_modal` - Field editing interface
   - `project_name` - Current project name
   - `status_message` - User feedback messages
   - `last_saved` - Save timestamp

4. **AI Integration**
   - `ai_settings` - Persisted API configuration
   - `has_api_key` - Computed memo for API availability
   - `processing_state` - AI agent loop progress
   - `current_iteration` / `max_iterations_signal` - Agent iteration tracking

5. **Network Status**
   - `connection_status` - Online/offline detection
   - `is_online` - Computed from connection status

6. **Chat Interface**
   - `chat_collapsed` - Chat panel visibility
   - `chat_messages` - Message history
   - `processing_state` - Agent execution state

### Initialization Process

1. **EditorState Creation**
   ```rust
   let state = EditorState::new();
   ```
   - Creates default slick sheet with title "Hello World" and body text
   - Initializes with default template containing Handlebars placeholders
   - No SVG output or errors initially

2. **Signal Extraction**
   - Extracts individual signals from state for reactivity
   - Each signal can be passed to child components

3. **Initial Compilation**
   ```rust
   let compile = move || {
       let source = typst_source.get();
       match VirtualWorld::compile_to_svg(&source) {
           Ok(svg) => { svg_output.set(Some(svg)); error.set(None); }
           Err(errors) => { error.set(Some(errors.join("\n"))); }
       }
   };
   compile();
   ```
   - Compiles the default Typst source to SVG immediately
   - Updates either svg_output (success) or error signal
   - Enables immediate preview display on startup

---

## Module Structure

The application is organized into six main modules:

```
src/
├── main.rs              # Bootstrap entry point
├── ai/                  # AI integration (agent loops, OpenRouter API)
├── editor/              # Main UI and state management
│   ├── mod.rs           # Root Editor component
│   ├── state.rs         # EditorState and signals
│   ├── content.rs       # Content model/serialization
│   ├── chat_panel.rs    # AI chat interface
│   ├── edit_modal.rs    # Field editing UI
│   ├── settings_modal.rs # AI settings UI
│   ├── status_bar.rs    # Bottom status bar
│   └── links.rs         # Command URL parsing
├── world/               # Typst VirtualWorld implementation
│   ├── mod.rs           # World trait + compilation
│   └── fonts.rs         # Font loader and FontBook
├── persistence/         # Save/load and PDF export
│   ├── mod.rs
│   ├── project.rs       # Project JSON format
│   └── export.rs        # PDF generation
├── data/                # SlickSheetData JSON structure
│   └── mod.rs
├── template/            # Handlebars template engine
│   └── mod.rs
└── templates/           # Built-in template gallery
    └── mod.rs
```

Each module is independently testable with unit tests in `mod.rs#[cfg(test)]` sections.

---

## Global Configuration & Initialization

### Tracing (Logging)

```rust
tracing_wasm::set_as_global_default();
```

- Initializes the `tracing` crate to output to browser console
- All `info!()`, `debug!()`, `error!()` macros within the app now log to DevTools
- No configuration files needed--enabled globally via default

### Leptos Runtime

```rust
leptos::mount_to(app_element, editor::Editor);
```

- Initializes Leptos CSR runtime (no hydration or SSR)
- Creates the reactive system (signals, effects, memos)
- Mounts the component tree under #app

### Service Worker

Registered in `index.html` script:
```javascript
if ('serviceWorker' in navigator) {
    window.addEventListener('load', () => {
        navigator.serviceWorker.register('/sw.js')
            .then(reg => console.log('SW registered:', reg.scope))
            .catch(err => console.log('SW registration failed:', err));
    });
}
```

- Registers `/sw.js` for offline support after page load
- Enables PWA caching and works offline
- Logs success/failure to console

### Typst World Initialization

The `VirtualWorld` singleton initializes on first use:

```rust
static LIBRARY: OnceLock<LazyHash<Library>> = OnceLock::new();

fn library() -> &'static LazyHash<Library> {
    LIBRARY.get_or_init(|| LazyHash::new(Library::default()))
}
```

- **Lazy Initialization**: The Typst standard library is only compiled when first needed
- **Thread-Safe**: `OnceLock` ensures single initialization even in concurrent contexts
- **Memoization**: `LazyHash` caches library contents for performance
- **Fonts**: Loaded separately via `FontLoader::new()` in each VirtualWorld instance

---

## PWA Configuration: `manifest.json`

```json
{
  "name": "Slick Sheet Studio",
  "short_name": "SlickSheet",
  "description": "Create professional marketing slick sheets with Typst and AI assistance",
  "start_url": "/",
  "scope": "/",
  "display": "standalone",
  "background_color": "#1a1a2e",
  "theme_color": "#e94560",
  "orientation": "portrait-primary",
  "icons": [...],
  "categories": ["productivity", "design"],
  "lang": "en-US"
}
```

**PWA Features:**
- **display: "standalone"** - Runs without browser chrome on mobile/desktop
- **start_url: "/"** - Opens at root when launched from home screen
- **scope: "/"** - Service worker controls entire site
- **Icons** - Dual-format (192x192 and 512x512) SVG with "maskable" purpose for adaptive icons
- **Categories** - Classified as productivity and design tool

---

## Dependency Policy & Constraints

### Approved Dependencies
All dependencies in Cargo.toml are pre-approved according to project guidelines:

| Crate | Version | Category | Notes |
|-------|---------|----------|-------|
| leptos | 0.6 | Framework | CSR only, no SSR |
| typst | 0.12 | Core | Document compilation |
| serde | 1.0 | Data | Serialization |
| reqwest | 0.12 | Networking | JSON only, minimal features |
| chrono | 0.4 | Time | WASM-compatible |
| uuid | 1.0 | Utility | v4 and JS support |
| tracing | 0.1 | Logging | Structured logging |
| base64 | 0.22 | Encoding | Base64 codec |

### Constraints Enforced

1. **No new dependencies** may be added without explicit approval
2. **Must not use**:
   - Crates with <1000 downloads/week
   - Unmaintained crates (>12 months without updates)
   - Crates with active security advisories
3. **Prefer stdlib** over external crates where feasible
4. **Minimize WASM binary size** - all optimizations target small code footprint

---

## Build & Deployment Pipeline

### Development Build
```bash
# Set PATH to use rustup (not Homebrew)
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH"

# Run dev server with hot reload
trunk serve
```

### Production Build
```bash
# Compile with maximum optimizations (CSR only, no server code)
trunk build --release
```

**Output**: Optimized WASM in `dist/` with:
- Size optimization (`opt-level = "z"`)
- Link-time optimization
- Single codegen unit
- Typical binary: ~2-4MB (varies with Typst features)

### Testing
```bash
cargo test                              # Run all 95 tests
cargo clippy -- -D warnings             # Lint enforcement
cargo fmt                               # Format enforcement
```

---

## Known Initialization Patterns

### Leptos Signal Patterns

The application follows these reactive patterns:

1. **RwSignal<T>** - For mutable state that needs both read and write access
   - `typst_source`, `content_data`, `template_source` - content that changes frequently
   - `show_edit_modal`, `chat_messages` - UI state that multiple handlers modify

2. **Signal<T>** - For derived/computed values
   - Created via `create_memo(move |_| expression)`
   - `is_online`, `has_api_key` - computed from other signals

3. **create_rw_signal()** - Used at component root for top-level state
   - All state in Editor component initializes with `create_rw_signal()`
   - Passed down to child components via props

4. **move ||** closures - Event handlers and effects capture reactive values
   - Event handlers use `move || { }` to capture signal references
   - Leptos automatically tracks signal dependencies

### Debouncing Pattern

The Editor implements debounced compilation on source changes:
```rust
let debounce_handle = create_rw_signal(Option::<i32>::None);
```

- Stores a `setTimeout` handle ID
- On each keystroke, clears the old handle and sets a new timeout
- Delays compilation 500ms after user stops typing

---

## Summary

The Slick Sheet Studio bootstrap process is clean and minimal:

1. **HTML serves WASM** - index.html contains Trunk build directives and PWA metadata
2. **Rust entry point** - main.rs initializes logging and mounts the Editor component
3. **Single root component** - Editor manages all state via Leptos signals
4. **Lazy initialization** - Typst library and font loader initialize on first compilation
5. **Service worker** - Enables offline support and PWA installation
6. **Optimized build** - Release profile maximizes compression and minimizes binary size

The architecture is straightforward: pure CSR (client-side rendering) with no server code, suitable for static hosting on CDN or cloud storage services.
