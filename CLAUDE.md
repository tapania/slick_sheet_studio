# CLAUDE.md - AI Assistant Guidelines

## Project Overview
Slick Sheet Studio: Rust/WASM app for creating marketing slick sheets with Typst and AI assistance.

## Tooling

### Rust/WASM
```bash
# Ensure cargo bin is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Versions
rustc --version    # 1.89.0+
trunk --version    # 0.21.14
wasm-pack --version # 0.13.1
```

### Build Commands
```bash
cargo test              # Run all tests
cargo fmt               # Format code
cargo clippy            # Lint
trunk serve             # Dev server (http://localhost:8080)
trunk build --release   # Production build
wasm-pack test --headless --firefox  # WASM tests
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
- `google/gemini-3-flash` - default
- `anthropic/claude-4.5-haiku` - fallback
- `openai/gpt-5.2-mini` - alternative

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
