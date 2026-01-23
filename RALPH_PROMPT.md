# Ralph Loop Prompt

Execute the implementation plan at `plans/IMPLEMENTATION_PLAN.md`.

## Instructions

1. Read the plan and reviews in `reviews/` to understand current state
2. Determine which phase/task to work on next based on what's actually implemented
3. Complete tasks in parallel track order within each phase
4. After each task, verify it works end-to-end (UI to API where applicable)
5. For AI features, test using `.env.testing` API key
6. Mark progress by updating the plan's acceptance criteria checkboxes

## Key Constraints

- Run `cargo test` after code changes
- Run `cargo clippy -- -D warnings` before considering a task complete
- Use CLI tool for AI testing when available (Phase 2+)
- Each phase must be fully testable before moving to next

## Context Files

- `plans/IMPLEMENTATION_PLAN.md` - The implementation plan
- `reviews/final_verdict.md` - Current state assessment
- `CLAUDE.md` - Project guidelines and tooling
- `.env.testing` - API keys for AI integration tests
