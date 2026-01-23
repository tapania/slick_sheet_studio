# JSON/Typst Separation Architecture Plan

## Overview

This plan describes the architecture for separating content (JSON) from presentation (Typst templates), with AI-powered editing tools that validate changes before accepting them.

## Current State

- `Content` struct in `src/editor/content.rs` - hardcoded structure
- `Content::to_typst()` - generates Typst with embedded content
- Templates in `src/templates/mod.rs` - static Typst code, no data binding
- No separation between data and presentation
- AI edits raw Typst code directly

## Proposed Architecture

### 1. Data Model (JSON)

```
SlickSheetData {
  // Core content fields
  title: string
  subtitle?: string
  body: string

  // Structured sections
  sections: Section[]

  // Key-value metadata
  metadata: Record<string, string>

  // Feature lists
  features: string[]

  // Statistics/metrics
  stats: Stat[]

  // Contact information
  contact?: ContactInfo

  // Styling hints (colors, fonts - interpreted by template)
  style?: StyleHints
}

Section {
  heading: string
  content: string
  type: "text" | "list" | "table" | "quote"
  items?: string[]  // for list type
  rows?: string[][] // for table type
}

Stat {
  value: string
  label: string
  color?: string
}

ContactInfo {
  email?: string
  phone?: string
  website?: string
  address?: string
}

StyleHints {
  primaryColor?: string
  accentColor?: string
  fontFamily?: string
}
```

### 2. Template System

Templates use Handlebars-style placeholders:

```typst
// Template: product-sheet.typ
#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "{{style.fontFamily | default: 'Inter'}}", size: 11pt)

#align(center)[
  #text(size: 24pt, weight: "bold", fill: rgb("{{style.primaryColor | default: '#000000'}}"))[{{title}}]
  {{#if subtitle}}
  #v(0.5em)
  #text(size: 14pt, fill: gray)[{{subtitle}}]
  {{/if}}
]

#v(1em)

{{body}}

{{#each sections}}
== {{heading}}
{{#if (eq type "list")}}
{{#each items}}
- {{this}}
{{/each}}
{{else if (eq type "table")}}
#table(
  columns: {{columns}},
  {{#each rows}}
  {{#each this}}[{{this}}],{{/each}}
  {{/each}}
)
{{else}}
{{content}}
{{/if}}
{{/each}}

{{#if stats}}
#v(1em)
#table(
  columns: {{stats.length}},
  stroke: none,
  align: center,
  {{#each stats}}
  [#text(size: 24pt, weight: "bold", fill: rgb("{{color | default: '#e94560'}}"))[{{value}}]],
  {{/each}}
  {{#each stats}}
  [{{label}}],
  {{/each}}
)
{{/if}}

{{#if contact}}
#v(1em)
#align(center)[
  {{#if contact.email}}Email: {{contact.email}}{{/if}}
  {{#if contact.website}} | {{contact.website}}{{/if}}
]
{{/if}}
```

### 3. File Structure

```
src/
├── data/
│   ├── mod.rs              # Module exports
│   ├── schema.rs           # SlickSheetData struct definitions
│   ├── validation.rs       # JSON schema validation
│   └── defaults.rs         # Default data for templates
├── template/
│   ├── mod.rs              # Module exports
│   ├── engine.rs           # Template rendering engine
│   ├── parser.rs           # Handlebars-style parser
│   ├── validation.rs       # Template syntax validation
│   └── builtin/            # Built-in templates
│       ├── product_sheet.typ
│       ├── event_flyer.typ
│       ├── one_pager.typ
│       └── ...
├── ai/
│   ├── tools/
│   │   ├── mod.rs          # Tool exports
│   │   ├── read_json.rs    # Read JSON content tool
│   │   ├── write_json.rs   # Write JSON content tool
│   │   ├── read_template.rs # Read Typst template tool
│   │   └── write_template.rs # Write Typst template tool
│   └── ...
└── editor/
    └── ...
```

### 4. AI Tools Specification

#### Tool: read_json

```rust
/// Read the current JSON content data
/// Returns the full JSON as a formatted string
pub struct ReadJsonTool;

impl ReadJsonTool {
    pub fn execute(&self, state: &EditorState) -> ToolResult {
        let data = state.content_data.get();
        let json = serde_json::to_string_pretty(&data)?;
        ToolResult::Success(json)
    }
}
```

**AI Prompt Format:**
```
Use read_json to get the current content data.

Result:
{
  "title": "Product Name",
  "subtitle": "Tagline goes here",
  "body": "Lorem ipsum...",
  ...
}
```

#### Tool: write_json

```rust
/// Write new JSON content data
/// Validates the JSON before accepting
/// Returns validation errors if invalid
pub struct WriteJsonTool;

impl WriteJsonTool {
    pub fn execute(&self, state: &EditorState, new_json: &str) -> ToolResult {
        // Step 1: Parse JSON
        let data: SlickSheetData = match serde_json::from_str(new_json) {
            Ok(d) => d,
            Err(e) => return ToolResult::Error(format!("JSON parse error: {}", e))
        };

        // Step 2: Validate schema
        if let Err(errors) = validate_schema(&data) {
            return ToolResult::Error(format!("Validation errors:\n{}", errors.join("\n")));
        }

        // Step 3: Test compilation with current template
        let template = state.template_source.get();
        let typst = render_template(&template, &data)?;
        if let Err(errors) = compile_typst(&typst) {
            return ToolResult::Error(format!("Template compilation failed:\n{}", errors.join("\n")));
        }

        // Step 4: Accept the change
        state.content_data.set(data);
        state.refresh_preview();

        ToolResult::Success("JSON updated successfully. Preview refreshed.".to_string())
    }
}
```

**AI Prompt Format:**
```
Use write_json to update the content. Write the COMPLETE JSON object.

Example:
write_json({
  "title": "New Product Name",
  "subtitle": "Updated tagline",
  "body": "New body content...",
  "sections": [...],
  "stats": [...]
})
```

#### Tool: read_template

```rust
/// Read the current Typst template
/// Returns the full template source
pub struct ReadTemplateTool;

impl ReadTemplateTool {
    pub fn execute(&self, state: &EditorState) -> ToolResult {
        let template = state.template_source.get();
        ToolResult::Success(template)
    }
}
```

#### Tool: write_template

```rust
/// Write a new Typst template
/// Validates the template before accepting
/// Returns validation errors if invalid
pub struct WriteTemplateTool;

impl WriteTemplateTool {
    pub fn execute(&self, state: &EditorState, new_template: &str) -> ToolResult {
        // Step 1: Validate template syntax (placeholders, Handlebars constructs)
        if let Err(errors) = validate_template_syntax(new_template) {
            return ToolResult::Error(format!("Template syntax errors:\n{}", errors.join("\n")));
        }

        // Step 2: Test compilation with current data
        let data = state.content_data.get();
        let typst = render_template(new_template, &data)?;
        if let Err(errors) = compile_typst(&typst) {
            return ToolResult::Error(format!("Template compilation failed:\n{}", errors.join("\n")));
        }

        // Step 3: Accept the change
        state.template_source.set(new_template.to_string());
        state.refresh_preview();

        ToolResult::Success("Template updated successfully. Preview refreshed.".to_string())
    }
}
```

### 5. Validation Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                     AI Edit Request                              │
│                    (JSON or Template)                            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Parse & Syntax Check                          │
│  - JSON: Valid JSON syntax                                       │
│  - Template: Valid Handlebars + Typst syntax                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │ Syntax Valid?     │
                    └─────────┬─────────┘
                         │ NO        │ YES
                         ▼           ▼
              ┌──────────────┐  ┌─────────────────────────────────┐
              │ Return Error │  │     Schema Validation            │
              │ to AI        │  │  - JSON: Required fields present │
              └──────────────┘  │  - Template: All placeholders    │
                                │    have matching data fields     │
                                └─────────────────────────────────┘
                                              │
                                    ┌─────────┴─────────┐
                                    │ Schema Valid?     │
                                    └─────────┬─────────┘
                                         │ NO        │ YES
                                         ▼           ▼
                              ┌──────────────┐  ┌─────────────────────────────────┐
                              │ Return Error │  │     Compilation Test             │
                              │ to AI        │  │  - Render template with data     │
                              └──────────────┘  │  - Compile resulting Typst       │
                                                │  - Check for Typst errors        │
                                                └─────────────────────────────────┘
                                                              │
                                                    ┌─────────┴─────────┐
                                                    │ Compiles?         │
                                                    └─────────┬─────────┘
                                                         │ NO        │ YES
                                                         ▼           ▼
                                              ┌──────────────┐  ┌─────────────────────────────────┐
                                              │ Return Error │  │     Accept Change                │
                                              │ to AI        │  │  - Update state                  │
                                              └──────────────┘  │  - Refresh preview               │
                                                                │  - Return success                │
                                                                └─────────────────────────────────┘
```

### 6. Template Rendering Engine

The engine renders Handlebars-style templates to Typst:

```rust
pub struct TemplateEngine;

impl TemplateEngine {
    /// Render a template with data
    pub fn render(template: &str, data: &SlickSheetData) -> Result<String, Vec<String>> {
        let mut output = template.to_string();
        let mut errors = Vec::new();

        // Replace simple placeholders: {{field}}
        output = self.replace_simple_placeholders(&output, data, &mut errors);

        // Process conditionals: {{#if field}}...{{/if}}
        output = self.process_conditionals(&output, data, &mut errors);

        // Process loops: {{#each items}}...{{/each}}
        output = self.process_loops(&output, data, &mut errors);

        // Process helpers: {{field | default: 'value'}}
        output = self.process_helpers(&output, data, &mut errors);

        if errors.is_empty() {
            Ok(output)
        } else {
            Err(errors)
        }
    }
}
```

### 7. Default JSON for Each Template

Each built-in template has associated default JSON data:

```rust
pub fn default_data_for_template(template_id: &str) -> SlickSheetData {
    match template_id {
        "product-sheet" => SlickSheetData {
            title: "Product Name".to_string(),
            subtitle: Some("Tagline goes here".to_string()),
            body: "Overview of your amazing product...".to_string(),
            sections: vec![
                Section {
                    heading: "Key Features".to_string(),
                    type_: SectionType::List,
                    items: Some(vec![
                        "Feature one with benefit".to_string(),
                        "Feature two with benefit".to_string(),
                    ]),
                    ..Default::default()
                },
                Section {
                    heading: "Specifications".to_string(),
                    type_: SectionType::Table,
                    rows: Some(vec![
                        vec!["Dimension".to_string(), "Value".to_string()],
                        vec!["Weight".to_string(), "Value".to_string()],
                    ]),
                    ..Default::default()
                },
            ],
            contact: Some(ContactInfo {
                email: Some("sales@example.com".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        },
        // ... other templates
        _ => SlickSheetData::default()
    }
}
```

### 8. AI System Prompt

The AI receives this system prompt explaining the tools:

```
You are an AI assistant helping users create and edit marketing slick sheets.

## Available Tools

### read_json
Read the current content data as JSON.
Use this to understand what content exists before making changes.

### write_json
Update the content data with new JSON.
IMPORTANT: Always write the COMPLETE JSON object, not partial updates.
The system validates your JSON before accepting it.

Example:
write_json({
  "title": "My Product",
  "subtitle": "Best in class",
  "body": "Description here...",
  "sections": [...],
  "features": [...],
  "stats": [...],
  "contact": {...}
})

### read_template
Read the current Typst template.
Use this to understand the document structure before modifying it.

### write_template
Update the Typst template.
IMPORTANT: Always write the COMPLETE template, not partial updates.
The system validates and test-compiles your template before accepting it.

Templates use Handlebars-style placeholders:
- {{field}} - Simple value substitution
- {{#if field}}...{{/if}} - Conditional sections
- {{#each items}}...{{/each}} - Loop over arrays
- {{field | default: 'value'}} - Default values

## Workflow

1. First, use read_json and read_template to understand the current state
2. Make your changes by calling write_json or write_template
3. If the system returns validation errors, fix them and try again
4. Changes are only applied when validation passes

## Best Practices

- For content changes (text, data), modify the JSON
- For layout/styling changes, modify the template
- Always write complete files, never diffs or patches
- Test compile errors will guide you to fix issues
```

### 9. Implementation Phases

#### Phase 1: Data Model (2-3 hours)
- [ ] Create `src/data/schema.rs` with `SlickSheetData` and related structs
- [ ] Implement JSON serialization/deserialization
- [ ] Add validation in `src/data/validation.rs`
- [ ] Create default data generators in `src/data/defaults.rs`
- [ ] Write unit tests for data model

#### Phase 2: Template Engine (3-4 hours)
- [ ] Create `src/template/parser.rs` for Handlebars-style parsing
- [ ] Implement `src/template/engine.rs` renderer
- [ ] Add template validation in `src/template/validation.rs`
- [ ] Convert existing templates to use placeholders
- [ ] Write unit tests for template engine

#### Phase 3: AI Tools (2-3 hours)
- [ ] Create `src/ai/tools/read_json.rs`
- [ ] Create `src/ai/tools/write_json.rs`
- [ ] Create `src/ai/tools/read_template.rs`
- [ ] Create `src/ai/tools/write_template.rs`
- [ ] Integrate tools with AgentLoop
- [ ] Write unit tests for tools

#### Phase 4: Editor Integration (2-3 hours)
- [ ] Update `EditorState` to hold both JSON and template
- [ ] Modify preview rendering to use template engine
- [ ] Update click-to-edit to modify JSON fields
- [ ] Update AI chat to use new tool system
- [ ] Ensure validation feedback shows in UI

#### Phase 5: Testing & Polish (1-2 hours)
- [ ] E2E tests for JSON editing
- [ ] E2E tests for template editing
- [ ] E2E tests for AI tool usage
- [ ] Error message UX improvements
- [ ] Documentation updates

### 10. Success Criteria

- [ ] JSON and Typst template are stored separately
- [ ] Template engine renders templates with JSON data
- [ ] AI can read and write JSON via tools
- [ ] AI can read and write templates via tools
- [ ] All edits are validated before acceptance
- [ ] Invalid edits return clear error messages
- [ ] AI writes complete files (not diffs)
- [ ] All 95+ existing tests still pass
- [ ] New tests for data model, template engine, and tools
- [ ] Zero clippy warnings
- [ ] trunk serve works with new architecture
- [ ] E2E tests pass for edit workflows

### 11. Migration Strategy

To maintain backward compatibility during migration:

1. Keep `Content` struct temporarily
2. Add new `SlickSheetData` alongside
3. Implement conversion functions between them
4. Gradually migrate components to new system
5. Remove old `Content` when migration complete

### 12. Example Workflow

**User says:** "Make the title red and add a features section"

**AI response:**
```
Let me read the current data and template first.

[read_json]
{
  "title": "Product Name",
  "body": "..."
}

[read_template]
#set page(...)
...

I'll update both to add the features section with a red title.

[write_json]
{
  "title": "Product Name",
  "body": "...",
  "features": [
    "Feature one",
    "Feature two"
  ],
  "style": {
    "primaryColor": "#ff0000"
  }
}

[write_template]
#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "Inter", size: 11pt)

#align(center)[
  #text(size: 24pt, weight: "bold", fill: rgb("{{style.primaryColor | default: '#000000'}}"))[{{title}}]
]

{{#if features}}
== Features
{{#each features}}
- {{this}}
{{/each}}
{{/if}}

...

Done! I've updated the title color to red and added a features section.
```
