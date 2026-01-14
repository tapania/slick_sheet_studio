# HLPP.md - High Level Project Plan: Slick Sheet Studio (Rust Edition)

## 1. Executive Summary
**Project:** Slick Sheet Studio (Rust/WASM)
**Vision:** A high-performance, privacy-focused web application for creating marketing slick sheets.
**Core Architecture:** A monolithic **Pure Rust** application compiled to WebAssembly. It runs entirely in the user's browser, utilizing the `typst` crate directly for zero-latency rendering.
**AI Integration:** "Bring Your Own Key" (BYOK). The app connects directly to OpenRouter.ai from the client side, storing API keys securely in the browser's Local Storage.
**Target Audience:** Developers comfortable with code; Typst editor exposed for power users.

---

## 2. Key Architecture Decisions

| Area | Decision | Details |
|------|----------|---------|
| **Fonts** | Hybrid | Embed 1-2 essential fonts (Inter, JetBrains Mono) in WASM, fetch additional on demand |
| **Deployment** | Static-first | PWA on static hosts (GitHub Pages, Netlify, Vercel), optional backend for future collaboration |
| **PDF Export** | Basic | Direct download from Typst output |
| **AI Loop** | Visual verification | Render → send to vision LLM → verify change matches intent → retry if needed |
| **LLM Support** | Model-aware | Multiple models (Claude, GPT-4, Gemini) with model-specific prompt templates |
| **Templates** | Gallery | 8-10 curated templates (product sheet, event flyer, one-pager, etc.) |
| **Image Storage** | External refs | Project JSON stores URLs/paths, images fetched separately |
| **Agent Iterations** | User configurable | Default 3-5, adjustable in settings |
| **Live Preview** | Adaptive | Debounced (300ms) for light docs, manual refresh for heavy renders |
| **Offline** | Full PWA | Service worker caches app, fonts, templates |

---

## 3. Specification Directory
The project details are split into four primary sub-specifications.

| Spec File | Scope | Key Topics |
| :--- | :--- | :--- |
| **`specs/01_rust_core.md`** | **The "World" & Compiler** | `typst::World` trait in WASM, virtual file paths, hybrid font loading |
| **`specs/02_ui_interaction.md`** | **Leptos & Editor UX** | Signal-based state, split-pane editor, `cmd://edit/` link interception, adaptive preview |
| **`specs/03_ai_client.md`** | **Agentic AI System** | OpenRouter client, visual verification loop, model-specific prompts, retry logic |
| **`specs/04_persistence.md`** | **Data & Assets** | File System Access API, JSON project format, PWA/service worker |

---

## 4. High-Level Architecture

### The Stack
* **Framework:** **Leptos** (High-performance, signal-based Rust web framework)
* **Compiler:** **`typst`** (The core crate, compiled directly into the app)
* **Network:** **`reqwest`** (WASM-compatible HTTP client)
* **Build:** **Trunk** (WASM build tool for Leptos)

### Core Components

#### 4.1 VirtualWorld (Typst Integration)
Implements `typst::World` trait for in-browser compilation:
- In-memory file system for `.typ` sources and assets
- Font loader: embedded defaults + async fetch for extras
- Image handling via Blob URLs from drag-drop

#### 4.2 Agentic AI System
Inspired by Claude Agent SDK:
```
User Request → Generate Typst Code → Compile → Render to Image
    → Vision LLM Verification → (Loop if needed) → Apply Changes
```
- Max iterations: user-configurable (settings)
- Model-specific prompt templates per LLM provider
- Error recovery: compile errors fed back to agent for self-correction

#### 4.3 Editor UX (Leptos)
- **Split view:** Typst code editor + live SVG preview
- **Click-to-edit:** `cmd://edit/{field}` links in SVG → open edit modal
- **Preview mode:** Debounced auto-render OR manual refresh (user toggle)
- **State:** Leptos signals for reactive content/layout updates

#### 4.4 Persistence Layer
- **Project format:** JSON with external image references
- **File API:** Browser File System Access API for save/load
- **PWA:** Service worker for offline capability (excludes AI features)

---

## 5. Development Phases

### Phase 1: The Rust Foundation (The "Virtual World")
*Goal: Get Typst compiling a string to SVG inside a web browser.*
* Initialize Leptos project with Trunk build
* Implement minimal `VirtualWorld` struct implementing `typst::World`
* Embed 2 default fonts (Inter, JetBrains Mono)
* Basic Typst → SVG render pipeline
* **Milestone:** Text input renders live to SVG

### Phase 2: The Editor Core
*Goal: Build the split-pane editor with live preview.*
* Build split-pane editor (textarea or lightweight code editor)
* Implement content struct with Leptos signals
* Add `cmd://` link interception for click-to-edit
* Adaptive preview (debounced + manual toggle)
* **Milestone:** Editable document with live preview

### Phase 3: The Agentic AI Layer
*Goal: Enable AI-driven layout generation with visual verification.*
* OpenRouter client (`reqwest` WASM)
* Settings modal: API key storage (localStorage), model selection, max iterations
* Implement agentic loop:
  - Prompt construction with current Typst code
  - Code generation → compile → render to PNG
  - Visual verification via vision-capable model
  - Retry logic with error context
* Model-specific prompt templates (Claude, GPT-4, Gemini)
* **Milestone:** "Make the title red" works end-to-end with verification

### Phase 4: Assets & Templates
*Goal: Enable custom images and starter templates.*
* Image drag-drop → Blob URL injection into VirtualWorld
* Template gallery UI (8-10 templates)
* Template selection on new project
* **Milestone:** User can start from template, add custom images

### Phase 5: Persistence & PWA
*Goal: Production-ready file handling and offline support.*
* Project save/load (JSON + File System Access API)
* PDF export via `typst::export::pdf`
* Service worker for offline mode
* Async font fetching for non-embedded fonts
* **Milestone:** Full offline-capable PWA

---

## 6. File Structure

```
slick_sheet_studio/
├── Cargo.toml
├── Trunk.toml
├── index.html
├── src/
│   ├── main.rs              # Leptos app entry
│   ├── app.rs               # Root component
│   ├── world/
│   │   ├── mod.rs           # VirtualWorld impl
│   │   └── fonts.rs         # Font loading
│   ├── editor/
│   │   ├── mod.rs           # Editor components
│   │   ├── preview.rs       # SVG preview pane
│   │   └── code.rs          # Code editor pane
│   ├── ai/
│   │   ├── mod.rs           # Agent orchestration
│   │   ├── client.rs        # OpenRouter client
│   │   ├── prompts/         # Model-specific prompts
│   │   └── verify.rs        # Visual verification
│   ├── persistence/
│   │   ├── mod.rs           # Save/load logic
│   │   └── project.rs       # Project struct
│   └── templates/
│       └── mod.rs           # Built-in templates
├── assets/
│   ├── fonts/               # Embedded fonts
│   └── templates/           # Template .typ files
└── sw.js                    # Service worker
```

---

## 7. Verification Plan

| Phase | Test |
|-------|------|
| Phase 1 | Run `trunk serve`, type in editor, see SVG update |
| Phase 2 | Click text in SVG, modal opens, edit persists |
| Phase 3 | Enter API key, send prompt, observe agent loop in console, see result applied |
| Phase 4 | Drag image into app, see it appear in preview |
| Phase 5 | Save project, close tab, reopen, load project, verify state restored |

---

## 8. Template Gallery (Planned)

1. **Product Sheet** - Single product showcase with specs
2. **Event Flyer** - Date/time/location focused
3. **One-Pager** - Executive summary style
4. **Comparison Chart** - Side-by-side feature comparison
5. **Case Study** - Problem/solution/results format
6. **Team Profile** - Staff/team member highlights
7. **Pricing Table** - Tiered pricing display
8. **Newsletter** - Multi-section content layout
9. **Infographic** - Data visualization focused
10. **Minimal** - Clean, typography-focused design

---

## 9. Next Steps
Begin with **Phase 1** - Initialize the Leptos project and implement the VirtualWorld.
