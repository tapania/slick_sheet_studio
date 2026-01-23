# Ralph Loop Prompt: JSON/Typst Separation

Copy and run:

```
/ralph-loop:ralph-loop "Implement JSON/Typst Separation per plans/JSON_PLAN.md

Read plans/JSON_PLAN.md for complete architecture.
Read CLAUDE.md for tooling and dev rules.

## Summary
Separate content (JSON) from presentation (Typst templates). Create AI tools (read_json, write_json, read_template, write_template) that validate all changes before accepting. AI writes complete files, not diffs.

## Phases
1. Data Model: src/data/ (schema.rs, validation.rs, defaults.rs)
2. Template Engine: src/template/ (parser.rs, engine.rs, validation.rs, builtin/*.typ)
3. AI Tools: src/ai/tools/ (read_json.rs, write_json.rs, read_template.rs, write_template.rs)
4. Editor Integration: Update state.rs, mod.rs to use template engine
5. AI Agent: Wire tools into AgentLoop, update prompts

## Per-Phase
- cargo fmt && cargo clippy -- -D warnings
- cargo test (all pass)
- Add tests for new code

## Success Criteria
- JSON/template separation working
- All 4 AI tools implemented with validation
- Click-to-edit modifies JSON
- AI chat uses tools
- Invalid edits rejected with clear errors
- All tests pass, zero clippy warnings
- trunk serve works

Output <promise>COMPLETE</promise> when done." --max-iterations 100 --completion-promise "COMPLETE"
```
