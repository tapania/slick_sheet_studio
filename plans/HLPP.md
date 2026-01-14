# HLPP.md - High Level Project Plan: Slick Sheet Studio (Rust Edition)

## 1. Executive Summary
**Project:** Slick Sheet Studio (Rust/WASM)  
**Vision:** A high-performance, privacy-focused web application for creating marketing slick sheets.  
**Core Architecture:** A monolithic **Pure Rust** application compiled to WebAssembly. It runs entirely in the user's browser, utilizing the `typst` crate directly for zero-latency rendering.  
**AI Integration:** "Bring Your Own Key" (BYOK). The app connects directly to OpenRouter.ai from the client side, storing API keys securely in the browser's Local Storage.

---

## 2. Specification Directory
The project details are split into four primary sub-specifications to be defined via the SpecKit Q&A process.

| Spec File | Scope | Key Questions to Resolve |
| :--- | :--- | :--- |
| **[`specs/01_rust_core.md`](./specs/01_rust_core.md)** | **The "World" & Compiler** | How do we implement the `typst::World` trait in WASM? How do we manage virtual file paths and font loading in memory? |
| **[`specs/02_ui_interaction.md`](./specs/02_ui_interaction.md)** | **Leptos & Editor UX** | Managing state with Leptos Signals. Implementing the "Link Hack" (SVG click interception) in a Rust-native way. |
| **[`specs/03_ai_client.md`](./specs/03_ai_client.md)** | **LLM Integration** | Using `reqwest` (WASM) for OpenRouter API calls. Prompt engineering for Typst code generation. Secure key storage in `localStorage`. |
| **[`specs/04_persistence.md`](./specs/04_persistence.md)** | **Data & Assets** | Implementing "Local-First" saving using the Browser File System Access API. Handling image assets via Object URLs. |

---

## 3. High-Level Architecture
*See `specs/01_rust_core.md` for the deep dive.*

The application is a standalone WASM binary with no mandatory backend dependencies.

1.  **The Stack:**
    * **Framework:** **Leptos** (High-performance, signal-based Rust web framework).
    * **Compiler:** **`typst`** (The core crate, compiled directly into the app).
    * **Network:** **`reqwest`** (WASM-compatible HTTP client).
2.  **The State (In-Memory):**
    * `AppState`: Holds the `content` (Struct/JSON) and `layout_template` (String).
    * `VirtualWorld`: A struct implementing `typst::World` that serves files from memory buffers instead of disk.
3.  **The AI Loop:**
    * User prompts for a layout change.
    * App retrieves Key from LocalStorage.
    * App calls OpenRouter API $\rightarrow$ Receives new Typst code $\rightarrow$ Hot-swaps `layout_template` $\rightarrow$ Re-compiles instantly.

---

## 4. Development Phases

### Phase 1: The Rust Foundation (The "Virtual World")
*Goal: Get Typst compiling a string to SVG inside a web browser.*
* **Sub-tasks:**
    * Initialize Leptos project.
    * Implement a minimal `typst::World` struct.
    * Load default fonts (embedded or fetched on init).
    * **Milestone:** A text box where typing "Hello World" renders a live Typst SVG.

### Phase 2: The Data-Driven Editor
*Goal: Separate content from layout and enable "Click-to-Edit".*
* **Sub-tasks:**
    * Define the `Content` struct (Title, Body, ImageURL).
    * Implement the `render(data, editable)` wrapper in Rust logic.
    * Build the Event Handler: Intercept `cmd://edit/` links in the SVG to open Leptos modals.
    * **Milestone:** Clicking the SVG title opens a Rust-controlled input modal that updates the render.

### Phase 3: The AI & Asset Layer
*Goal: Enable layout generation and custom images.*
* **Sub-tasks:**
    * Build the Settings Modal (Input for OpenRouter API Key).
    * Implement the Chat Interface (send `layout.typ` + prompt to LLM).
    * Implement Image Drag-and-Drop (Create `Blob` $\rightarrow$ `URL.createObjectURL` $\rightarrow$ Inject into Virtual World).
    * **Milestone:** User can drag an image in, and ask AI to "Make the image circular", and it actually happens.

### Phase 4: Local-First Polish
*Goal: Production-ready file handling.*
* **Sub-tasks:**
    * Implement Save/Load `.json` project files.
    * PDF Export (using `typst::export::pdf`).
    * Error handling for AI failures (e.g., invalid Typst code returned).

---

## 5. Next Steps
We will proceed by defining the core logic in **`specs/01_rust_core.md`**.

**Immediate Question:** Do you want to embed the fonts into the WASM binary (simpler, huge download) or fetch them asynchronously at startup (complex `World` implementation, faster initial load)?