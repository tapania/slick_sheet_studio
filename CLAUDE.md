# CLAUDE.md - AI Assistant Guidelines

## Project Overview
Slick Sheet Studio: Rust/WASM app for creating marketing slick sheets with Typst and AI assistance.

## Tooling

### Rust/WASM
```bash
# CRITICAL: Use rustup toolchain, NOT Homebrew's rust
# Homebrew rust doesn't have wasm32-unknown-unknown target
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH"

# Verify correct rustc is used (should show ~/.rustup path, NOT /opt/homebrew)
which rustc        # Must be: ~/.rustup/toolchains/stable-aarch64-apple-darwin/bin/rustc

# Versions
rustc --version    # 1.92.0+
trunk --version    # 0.21.14
```

**Run this PATH export before ANY cargo/trunk command.** Without it, WASM builds fail with "can't find crate for `core`".

### Build Commands
```bash
# Always set PATH first (or add to shell profile)
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH"

cargo test              # Run all tests (95 tests)
cargo fmt               # Format code
cargo clippy -- -D warnings  # Lint (zero warnings required)
trunk serve             # Dev server (http://localhost:8080)
trunk build --release   # Production build
```

### Timeouts
Rust compilation is slow. Use extended timeouts:
- `cargo build`: 10 minutes (600000ms)
- `cargo install`: 10 minutes
- `trunk build --release`: 10 minutes
- `wasm-pack build`: 5 minutes

### Browser Testing (@dev-browser)
Start the dev-browser server first:
```bash
cd ~/.claude/plugins/cache/dev-browser-marketplace/dev-browser/*/skills/dev-browser
./server.sh &
# Wait for "Ready" message
```

Run test scripts:
```bash
cd ~/.claude/plugins/cache/dev-browser-marketplace/dev-browser/*/skills/dev-browser
npx tsx <<'EOF'
import { connect, waitForPageLoad } from "@/client.js";

const client = await connect();
const page = await client.page("test");
await page.goto("http://localhost:8080");
await waitForPageLoad(page);
await page.screenshot({ path: "tmp/screenshot.png" });
console.log({ title: await page.title() });
await client.disconnect();
EOF
```

### API Testing (.env.testing)
```bash
# Copy template and add your key
cp .env.testing.template .env.testing
# Edit .env.testing with your OPENROUTER_API_KEY

# Load before running integration tests
source .env.testing
```

Test models (cost-optimized):
- `google/gemini-2.0-flash-exp:free` - default (free)
- `anthropic/claude-3.5-haiku` - fast & quality
- `openai/gpt-4o-mini` - alternative

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

## Architecture
See `plans/HLPP.md` for full details. Key components:
- `src/world/` - Typst VirtualWorld implementation
- `src/editor/` - Leptos UI components
- `src/ai/` - Agentic AI with visual verification
- `src/persistence/` - Save/load, PWA

## Known Issues & Solutions

### SVG Link Rendering (SOLVED)
**Problem:** `typst_svg::svg()` does NOT render `#link()` elements as clickable `<a>` tags. The typst-svg crate explicitly skips `FrameItem::Link` items (see line 199 in typst-svg source: "TODO: SVGs could contain links, couldn't they?").

**Solution:** Post-process SVG to add link overlays. In `src/world/mod.rs`:
1. After compiling, iterate `page.frame.items()` to find `FrameItem::Link(Destination::Url(url), size)`
2. Collect link positions (x, y, width, height) and URLs
3. Inject transparent clickable `<a><rect/></a>` elements at end of SVG before `</svg>`

```rust
// Pattern for extracting links from Frame
for (pos, item) in frame.items() {
    match item {
        FrameItem::Link(Destination::Url(url), size) => {
            // Collect link info
        }
        FrameItem::Group(group) => {
            // Recursively extract from nested groups
        }
        _ => {}
    }
}
```

### Leptos Signal Patterns
- Use `RwSignal<T>` for state that needs read/write from multiple places
- Use `Signal<T>` for derived/computed values
- Use `Callback<T>` for event handlers passed to child components
- Always use `move ||` closures in view macros for reactive updates

### Clippy Patterns to Watch
- Collapse nested `if let` into outer `match` patterns
- Use `matches!()` macro for simple boolean match expressions
- Avoid `#[allow(dead_code)]` unless truly necessary for public API

## Plans
- `plans/HLPP.md` - High-level project plan
- `plans/UI-plan.md` - UI implementation specification
- `plans/JSON_PLAN.md` - JSON/Typst separation architecture
