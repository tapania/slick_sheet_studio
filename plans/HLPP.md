# HLPP.md - High Level Project Plan: Slick Sheet Studio (Rust Edition)

## 1. Executive Summary
**Project:** Slick Sheet Studio (Rust/WASM)
**Vision:** A high-performance, privacy-focused web application for creating marketing slick sheets.
**Core Architecture:** A monolithic **Pure Rust** application compiled to WebAssembly. It runs entirely in the user's browser, utilizing the `typst` crate directly for zero-latency rendering.
**AI Integration:** "Bring Your Own Key" (BYOK). The app connects directly to OpenRouter.ai from the client side, storing API keys securely in the browser's Local Storage.
**Target Audience:** Developers comfortable with code; Typst editor exposed for power users.
**Development Methodology:** Test-Driven Development (TDD) with high coverage targets.

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
| **Testing** | TDD | Write tests first, >80% coverage target |
| **Dependencies** | Minimal | Only well-maintained, widely-used crates |

---

## 3. Test-Driven Development Strategy

### 3.1 TDD Workflow
1. **Red:** Write a failing test that defines expected behavior
2. **Green:** Write minimal code to make the test pass
3. **Refactor:** Clean up while keeping tests green

### 3.2 Testing Libraries (Approved)

| Library | Purpose | Notes |
|---------|---------|-------|
| `#[test]` | Unit tests | Built-in, no dependency |
| `#[cfg(test)]` | Test modules | Built-in |
| `wasm-bindgen-test` | WASM browser tests | Required for Leptos/WASM |
| `tokio` (test feature) | Async test runtime | Use `#[tokio::test]` |
| `insta` | Snapshot testing | Excellent for SVG/render output comparison |
| `proptest` | Property-based testing | Well-maintained, good for edge cases |

### 3.3 Coverage Targets

| Module | Target | Rationale |
|--------|--------|-----------|
| `world/` (VirtualWorld) | 90% | Core compilation logic, must be rock-solid |
| `ai/` (Agent) | 80% | Mock external APIs, test orchestration |
| `persistence/` | 85% | Data integrity critical |
| `editor/` | 70% | UI components harder to unit test |
| **Overall** | **80%** | Balance between coverage and pragmatism |

### 3.4 Test Organization
```
src/
├── world/
│   ├── mod.rs
│   └── tests.rs          # Unit tests for VirtualWorld
├── ai/
│   ├── mod.rs
│   └── tests.rs          # Agent logic tests (mocked API)
└── ...

tests/
├── integration/
│   ├── compile_test.rs   # End-to-end Typst compilation
│   ├── render_test.rs    # SVG output verification
│   └── agent_test.rs     # Full agent loop (mocked)
└── wasm/
    └── browser_test.rs   # wasm-bindgen-test for DOM interaction
```

---

## 4. Dependency Policy

### 4.1 Approved Dependencies

**Core:**
| Crate | Version | Purpose |
|-------|---------|---------|
| `typst` | latest | Document compiler |
| `leptos` | 0.6+ | Reactive web framework |
| `wasm-bindgen` | latest | JS interop |
| `web-sys` | latest | Web API bindings |
| `js-sys` | latest | JS type bindings |

**Serialization & Data:**
| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` | 1.x | Serialization framework |
| `serde_json` | 1.x | JSON support |

**Async & Networking:**
| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.x | Async runtime (test + limited WASM) |
| `reqwest` | 0.11+ | HTTP client (WASM feature) |
| `futures` | 0.3+ | Future utilities |

**Testing (dev-dependencies):**
| Crate | Version | Purpose |
|-------|---------|---------|
| `wasm-bindgen-test` | latest | WASM testing |
| `insta` | latest | Snapshot tests |
| `proptest` | 1.x | Property-based tests |

**Utilities:**
| Crate | Version | Purpose |
|-------|---------|---------|
| `thiserror` | 1.x | Error derive macros |
| `tracing` | 0.1+ | Logging/diagnostics |
| `base64` | 0.21+ | Encoding (if needed) |

### 4.2 Dependency Guidelines
- **No** crates with <1000 downloads/week
- **No** crates unmaintained >12 months
- **No** crates with known security advisories
- **Prefer** Rust standard library when feasible
- **Prefer** crates already in Leptos/Typst dependency tree

---

## 5. Specification Directory
The project details are split into four primary sub-specifications.

| Spec File | Scope | Key Topics |
| :--- | :--- | :--- |
| **`specs/01_rust_core.md`** | **The "World" & Compiler** | `typst::World` trait in WASM, virtual file paths, hybrid font loading |
| **`specs/02_ui_interaction.md`** | **Leptos & Editor UX** | Signal-based state, split-pane editor, `cmd://edit/` link interception, adaptive preview |
| **`specs/03_ai_client.md`** | **Agentic AI System** | OpenRouter client, visual verification loop, model-specific prompts, retry logic |
| **`specs/04_persistence.md`** | **Data & Assets** | File System Access API, JSON project format, PWA/service worker |

---

## 6. High-Level Architecture

### The Stack
* **Framework:** **Leptos** (High-performance, signal-based Rust web framework)
* **Compiler:** **`typst`** (The core crate, compiled directly into the app)
* **Network:** **`reqwest`** (WASM-compatible HTTP client)
* **Build:** **Trunk** (WASM build tool for Leptos)

### Core Components

#### 6.1 VirtualWorld (Typst Integration)
Implements `typst::World` trait for in-browser compilation:
- In-memory file system for `.typ` sources and assets
- Font loader: embedded defaults + async fetch for extras
- Image handling via Blob URLs from drag-drop

#### 6.2 Agentic AI System
Inspired by Claude Agent SDK:
```
User Request → Generate Typst Code → Compile → Render to Image
    → Vision LLM Verification → (Loop if needed) → Apply Changes
```
- Max iterations: user-configurable (settings)
- Model-specific prompt templates per LLM provider
- Error recovery: compile errors fed back to agent for self-correction

#### 6.3 Editor UX (Leptos)
- **Split view:** Typst code editor + live SVG preview
- **Click-to-edit:** `cmd://edit/{field}` links in SVG → open edit modal
- **Preview mode:** Debounced auto-render OR manual refresh (user toggle)
- **State:** Leptos signals for reactive content/layout updates

#### 6.4 Persistence Layer
- **Project format:** JSON with external image references
- **File API:** Browser File System Access API for save/load
- **PWA:** Service worker for offline capability (excludes AI features)

---

## 7. Development Phases (TDD)

### Phase 1: The Rust Foundation (The "Virtual World")
*Goal: Get Typst compiling a string to SVG inside a web browser.*

**TDD Sequence:**
1. Write test: `VirtualWorld` implements required `typst::World` methods
2. Write test: Compile "Hello World" returns valid SVG
3. Write test: Font loading returns embedded fonts
4. Implement minimal code to pass each test
5. Snapshot test: SVG output matches expected baseline

**Deliverables:**
- [ ] Leptos project scaffold with test harness
- [ ] `VirtualWorld` struct with tests
- [ ] Embedded fonts with loading tests
- [ ] Typst → SVG pipeline with snapshot tests
- **Milestone:** All tests green, "Hello World" renders

### Phase 2: The Editor Core
*Goal: Build the split-pane editor with live preview.*

**TDD Sequence:**
1. Write test: Content struct serializes/deserializes correctly
2. Write test: Signal updates trigger re-render
3. Write test: `cmd://` URLs are intercepted
4. Write test: Debounce delays render appropriately

**Deliverables:**
- [ ] Content model with serde tests
- [ ] Editor component with signal tests
- [ ] Link interception with unit tests
- [ ] Adaptive preview with timing tests
- **Milestone:** All tests green, editable preview works

### Phase 3: The Agentic AI Layer
*Goal: Enable AI-driven layout generation with visual verification.*

**TDD Sequence:**
1. Write test: API client constructs correct request format
2. Write test: Response parsing handles valid/invalid JSON
3. Write test: Agent loop terminates after max iterations
4. Write test: Visual verification mock returns expected result
5. Write test: Error recovery feeds errors back to prompt

**Deliverables:**
- [ ] OpenRouter client with mocked request tests
- [ ] Agent orchestration with state machine tests
- [ ] Prompt templates with output validation tests
- [ ] Verification logic with mock vision responses
- **Milestone:** All tests green, agent loop works with mocks

### Phase 4: Assets & Templates
*Goal: Enable custom images and starter templates.*

**TDD Sequence:**
1. Write test: Image blob creates valid URL
2. Write test: Template loads and compiles without errors
3. Write test: Template gallery lists all templates

**Deliverables:**
- [ ] Image handling with blob tests
- [ ] Template loading with compilation tests
- [ ] Gallery component with listing tests
- **Milestone:** All tests green, templates render correctly

### Phase 5: Persistence & PWA
*Goal: Production-ready file handling and offline support.*

**TDD Sequence:**
1. Write test: Project serializes to valid JSON
2. Write test: Project deserializes and restores state
3. Write test: PDF export produces valid output
4. Write test: Service worker caches required assets

**Deliverables:**
- [ ] Save/load with round-trip tests
- [ ] PDF export with output validation
- [ ] Service worker with cache tests
- **Milestone:** All tests green, offline mode works

---

## 8. File Structure

```
slick_sheet_studio/
├── Cargo.toml
├── Trunk.toml
├── index.html
├── CLAUDE.md                # AI assistant guidelines
├── src/
│   ├── main.rs              # Leptos app entry
│   ├── lib.rs               # Library root (for testing)
│   ├── app.rs               # Root component
│   ├── world/
│   │   ├── mod.rs           # VirtualWorld impl
│   │   ├── fonts.rs         # Font loading
│   │   └── tests.rs         # Unit tests
│   ├── editor/
│   │   ├── mod.rs           # Editor components
│   │   ├── preview.rs       # SVG preview pane
│   │   ├── code.rs          # Code editor pane
│   │   └── tests.rs         # Unit tests
│   ├── ai/
│   │   ├── mod.rs           # Agent orchestration
│   │   ├── client.rs        # OpenRouter client
│   │   ├── prompts.rs       # Model-specific prompts
│   │   ├── verify.rs        # Visual verification
│   │   └── tests.rs         # Unit tests
│   ├── persistence/
│   │   ├── mod.rs           # Save/load logic
│   │   ├── project.rs       # Project struct
│   │   └── tests.rs         # Unit tests
│   └── templates/
│       ├── mod.rs           # Built-in templates
│       └── tests.rs         # Unit tests
├── tests/
│   ├── integration/         # Integration tests
│   └── wasm/                # Browser tests
├── assets/
│   ├── fonts/               # Embedded fonts
│   └── templates/           # Template .typ files
└── sw.js                    # Service worker
```

---

## 9. Verification Plan

| Phase | Automated Tests | Manual Verification |
|-------|-----------------|---------------------|
| Phase 1 | `cargo test`, `wasm-pack test`, snapshot tests | `trunk serve`, type → see SVG |
| Phase 2 | Signal tests, serialization tests | Click SVG → modal opens |
| Phase 3 | Mocked API tests, state machine tests | Real API key → agent works |
| Phase 4 | Blob tests, template compilation tests | Drag image → appears in preview |
| Phase 5 | Round-trip tests, cache tests | Save → close → reload → restore |

**CI Pipeline:**
```
cargo fmt --check
cargo clippy -- -D warnings
cargo test
wasm-pack test --headless --firefox
```

---

## 10. Template Gallery (Planned)

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

## 11. Next Steps
Begin with **Phase 1** - Set up project scaffold with test harness, then TDD the VirtualWorld.
