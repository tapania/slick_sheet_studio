# CLAUDE.md - AI Assistant Guidelines

## Project Overview
Slick Sheet Studio: Rust/WASM app for creating marketing slick sheets with Typst and AI assistance.

## Development Rules

### Test-Driven Development
- **Always write tests first** (Red → Green → Refactor)
- Target 80%+ overall coverage
- Use snapshot tests (`insta`) for SVG/render output
- Run `cargo test` before committing

### Dependency Policy
**Approved crates only.** Do not add new dependencies without explicit approval.

| Category | Allowed |
|----------|---------|
| Core | `typst`, `leptos`, `wasm-bindgen`, `web-sys`, `js-sys` |
| Data | `serde`, `serde_json` |
| Async | `tokio`, `reqwest`, `futures` |
| Testing | `wasm-bindgen-test`, `insta`, `proptest` |
| Utils | `thiserror`, `tracing`, `base64` |

**Forbidden:**
- Crates with <1000 downloads/week
- Unmaintained crates (>12 months no updates)
- Crates with security advisories

### Code Style
- Run `cargo fmt` before committing
- Zero `cargo clippy` warnings
- Prefer std library over external crates
- Keep functions small and testable

### Commands
```bash
cargo test              # Run all tests
cargo fmt               # Format code
cargo clippy            # Lint
trunk serve             # Dev server
trunk build --release   # Production build
```

## Architecture
See `plans/HLPP.md` for full details. Key components:
- `src/world/` - Typst VirtualWorld implementation
- `src/editor/` - Leptos UI components
- `src/ai/` - Agentic AI with visual verification
- `src/persistence/` - Save/load, PWA
