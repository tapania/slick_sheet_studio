# Template Module Documentation - Slick Sheet Studio

**Module Path:** `/Users/taala/repos/slick_sheet_studio/src/template/`

**Last Updated:** January 15, 2026

---

## Executive Summary

The template module provides a **Handlebars-style template engine** for rendering Typst documents with dynamic data. It consists of four main components:

- **Parser** (`parser.rs`): Converts template strings into an AST (Abstract Syntax Tree)
- **Engine** (`engine.rs`): Renders parsed templates with data, supporting variables, conditionals, and loops
- **Validation** (`validation.rs`): Validates template syntax and checks for unknown variables
- **Tests** (`tests.rs`): Comprehensive integration tests covering all features

The module is designed to bridge the gap between **JSON data** (`SlickSheetData`) and **Typst markup**, allowing AI agents and users to define reusable document templates with placeholders.

---

## Architecture Overview

### Module Structure

```
src/template/
├── mod.rs                 # Module exports and documentation
├── parser.rs              # Handlebars syntax parser → AST
├── engine.rs              # Template rendering engine
├── validation.rs          # Syntax and semantic validation
└── tests.rs               # Integration tests
```

### Data Flow

```
Template String (with {{placeholders}})
    ↓
parse_template()  [Parser]
    ↓
Vec<TemplateNode>  [AST]
    ↓
TemplateEngine::render()  [Engine] + SlickSheetData
    ↓
Rendered Typst Code
```

---

## Module Components

### 1. Parser Module (`parser.rs`)

**Purpose:** Convert Handlebars-style template syntax into an AST for processing.

#### Core Data Structures

```rust
pub enum TemplateNode {
    /// Raw text content (preserved as-is)
    Text(String),

    /// Variable substitution with optional default value
    /// Examples: {{title}}, {{style.primaryColor}}, {{field | default: 'fallback'}}
    Variable {
        path: Vec<String>,           // e.g., ["style", "primaryColor"]
        default: Option<String>,     // Default value if not found
    },

    /// Conditional block (if/else)
    /// Examples: {{#if subtitle}}...{{/if}}, {{#if contact}}...{{else}}...{{/if}}
    Conditional {
        path: Vec<String>,
        then_branch: Vec<TemplateNode>,
        else_branch: Vec<TemplateNode>,
    },

    /// Loop block for arrays
    /// Examples: {{#each features}}...{{/each}}, {{#each sections}}...{{/each}}
    Loop {
        path: Vec<String>,
        body: Vec<TemplateNode>,
    },
}
```

#### Parse Errors

```rust
pub enum ParseError {
    UnclosedTag { tag: String, position: usize },      // {{#if missing closing
    UnexpectedClosingTag { expected: String, found: String },  // Mismatched tags
    InvalidSyntax { message: String, position: usize }, // Malformed syntax
    EmptyVariableName { position: usize },             // {{}} with no name
}
```

#### Public API

```rust
pub fn parse_template(input: &str) -> Result<Vec<TemplateNode>, ParseError>
```
Parses a template string into a list of nodes. This is the entry point for template processing.

```rust
pub fn extract_variables(nodes: &[TemplateNode]) -> HashSet<String>
```
Extracts all variable paths referenced in a template (used for validation and dependency checking).

#### Supported Syntax

| Syntax | Example | Description |
|--------|---------|-------------|
| **Text** | `Hello World` | Raw text passed through unchanged |
| **Variable** | `{{title}}` | Simple variable substitution |
| **Nested Path** | `{{style.primaryColor}}` | Access nested object fields |
| **Default Value** | `{{title \| default: 'Untitled'}}` | Fallback if variable undefined |
| **Conditional** | `{{#if subtitle}}...{{/if}}` | Render block if variable is truthy |
| **If-Else** | `{{#if x}}yes{{else}}no{{/if}}` | Conditional with else branch |
| **Loop** | `{{#each features}}...{{/each}}` | Iterate over array items |
| **Loop Variable** | `{{this}}` | Current item in loop |
| **Loop Index** | `{{@index}}` | Zero-based index in loop |

#### Implementation Details

The parser is implemented as a **hand-written recursive descent parser** with:

- **Position tracking** for error reporting
- **Lookahead** to detect block tags (`{{#if}}`, `{{#each}}`)
- **Recursive parsing** for nested conditionals and loops
- **Whitespace handling** that preserves meaningful whitespace in text nodes

Key parsing flow:
1. Scan for `{{` markers to identify template tags
2. For each tag, determine type (variable, block, closing)
3. For block tags, recursively parse the body until the matching closing tag
4. Collect all nodes into a flat list (or nested for blocks)

**Test Coverage:** 11 tests covering simple variables, nested paths, defaults, conditionals, loops, and error cases.

---

### 2. Engine Module (`engine.rs`)

**Purpose:** Render parsed templates with actual data from `SlickSheetData`.

#### Core API

```rust
pub struct TemplateEngine;

impl TemplateEngine {
    pub fn render(template: &str, data: &SlickSheetData)
        -> Result<String, Vec<String>>
}
```

The engine:
1. Parses the template
2. Walks the AST, resolving variables from data
3. Accumulates output in a string buffer
4. Returns rendered Typst code or error messages

#### Data Resolution

The engine resolves variables in the following priority order:

**Simple Paths (one level):**
- `title` → `data.title`
- `subtitle` → `data.subtitle`
- `body` → `data.body`
- Other → Check `data.metadata` map

**Nested Paths (two+ levels):**
- `style.primaryColor`, `style.primary_color` → Style color (accepts both camelCase and snake_case)
- `style.accentColor`, `style.accent_color` → Style accent color
- `style.fontFamily`, `style.font_family` → Style font
- `contact.email`, `contact.phone`, `contact.website`, `contact.address` → Contact info fields
- `{array}.length` → Array length for `sections`, `features`, `stats`

**Special Loop Variables:**
- `this` → Current item in loop (as string)
- `@index` → Zero-based loop index (converted to string)

#### Loop Context

```rust
struct LoopContext<'a> {
    item: String,           // Current loop item
    index: usize,           // Zero-based index
    parent: Option<&'a LoopContext<'a>>,  // For nested loops
}
```

Supports nested loops with parent context tracking for future multi-level iterations.

#### Array Resolution

Arrays are converted to `Vec<String>` for iteration:

- **features:** Direct string cloning
- **sections:** Converted to descriptive strings like `"Heading: content"` or `"Heading: [item1, item2]"`
- **stats:** Formatted as `"value: label"` (e.g., `"50%: Growth"`)

#### Truthy Evaluation

For conditional blocks:
- **Empty arrays:** `false`
- **Non-empty arrays:** `true`
- **Optional fields:** `true` if Some and non-empty string
- **Optional objects:** `true` if Some
- **Resolved values:** `true` if non-empty string

#### Example: Full Template Rendering

```rust
let data = SlickSheetData::new("My Product")
    .with_subtitle("Amazing")
    .with_feature("Fast")
    .with_feature("Reliable");

let template = r#"
# {{title}}
{{#if subtitle}}## {{subtitle}}{{/if}}
Features:
{{#each features}}- {{this}}
{{/each}}
"#;

let result = TemplateEngine::render(template, &data)?;
// Output:
// # My Product
// ## Amazing
// Features:
// - Fast
// - Reliable
```

**Test Coverage:** 11 tests covering simple variables, nested variables, defaults, conditionals, loops, contact info, stats, and complex templates.

---

### 3. Validation Module (`validation.rs`)

**Purpose:** Validate template syntax and warn about potentially undefined variables.

#### Validation Errors

```rust
pub enum TemplateValidationError {
    ParseError(String),           // Template parse failed
    UnknownVariable(String),      // Variable not in known list
    EmptyTemplate,                // Template is empty/whitespace
}
```

#### Public API

```rust
pub fn validate_template(template: &str)
    -> Result<Vec<String>, Vec<TemplateValidationError>>
```

Returns:
- `Ok(warnings)` - List of warning messages for unknown variables (still valid)
- `Err(errors)` - Critical errors (parse failures, empty template)

```rust
pub fn validate_template_with_data(
    template: &str,
    data: &SlickSheetData,
    compile_fn: impl FnOnce(&str) -> Result<String, Vec<String>>,
) -> Result<(), Vec<String>>
```

Full validation including rendering and Typst compilation test.

#### Known Variables

The validator maintains a whitelist of **74 known variables** grouped by category:

**Top-level:**
- `title`, `subtitle`, `body`

**Arrays:**
- `sections`, `features`, `stats`

**Style fields:**
- `style`, `style.primaryColor`, `style.primary_color`
- `style.accentColor`, `style.accent_color`
- `style.fontFamily`, `style.font_family`

**Contact fields:**
- `contact`, `contact.email`, `contact.phone`, `contact.website`, `contact.address`

**Array metadata:**
- `sections.length`, `features.length`, `stats.length`

**Loop variables:**
- `this`, `@index`

**Section fields (in loops):**
- `heading`, `content`, `type`, `items`, `rows`, `columns`

**Stat fields (in loops):**
- `value`, `label`, `color`

#### Validation Strategy

1. **Syntax validation** via `parse_template()`
2. **Variable extraction** via `extract_variables()`
3. **Unknown variable warnings** (not errors - allows metadata keys)
4. **Full render test** (optional) with `validate_template_with_data()`

**Note:** Unknown variables are warnings, not errors, because templates can use custom metadata keys.

**Test Coverage:** 6 tests covering valid templates, empty templates, syntax errors, known variables, unknown variable warnings, and complex templates.

---

## Public API

### Main Exports

```rust
pub use engine::TemplateEngine;
pub use parser::{parse_template, TemplateNode};
pub use validation::{validate_template, TemplateValidationError};
```

### Common Usage Patterns

#### 1. Basic Template Rendering

```rust
use slick_sheet_studio::template::TemplateEngine;
use slick_sheet_studio::data::SlickSheetData;

let data = SlickSheetData::new("My Title");
let template = "# {{title}}";
let result = TemplateEngine::render(template, &data)?;
assert_eq!(result, "# My Title");
```

#### 2. Template Validation

```rust
use slick_sheet_studio::template::validate_template;

let result = validate_template("Hello {{title}}")?;  // Returns warnings vec
```

#### 3. Full Validation (with Typst compile test)

```rust
use slick_sheet_studio::template::validate_template_with_data;

validate_template_with_data(
    template,
    data,
    |rendered| typst_compiler.compile(rendered)
)?;
```

---

## Integration with Other Modules

### 1. Editor Module (`src/editor/state.rs`)

The editor state manages template rendering in the UI:

```rust
pub struct EditorState {
    pub template_source: RwSignal<String>,  // Handlebars template
    pub content_data: RwSignal<SlickSheetData>,  // JSON data
    pub typst_source: RwSignal<String>,    // Rendered result
}

impl EditorState {
    pub fn render_template(&self) {
        let template = self.template_source.get();
        let data = self.content_data.get();

        match TemplateEngine::render(&template, &data) {
            Ok(rendered) => self.typst_source.set(rendered),
            Err(errors) => self.error.set(Some(format!("Rendering error: {}", ...))),
        }
    }
}
```

**Default Template:** Defined in `editor/state.rs` as `DEFAULT_TEMPLATE` - a full product sheet with sections, features, and contact info.

### 2. AI Tools Module (`src/ai/tools/write_template.rs`)

The `WriteTemplateTool` validates templates before accepting them:

```rust
impl WriteTemplateTool {
    pub fn execute(
        new_template: &str,
        current_data: &SlickSheetData,
        compile_fn: impl FnOnce(&str) -> Result<String, Vec<String>>,
    ) -> Result<String, ToolResult> {
        // Step 1: Syntax validation
        validate_template(new_template)?;

        // Step 2: Rendering test
        let rendered = TemplateEngine::render(new_template, current_data)?;

        // Step 3: Typst compilation test
        compile_fn(&rendered)?;

        Ok(new_template.to_string())
    }
}
```

This ensures the AI can only create templates that:
- Parse correctly
- Render with current data
- Compile to valid Typst

### 3. Data Module (`src/data/`)

Templates operate on the `SlickSheetData` structure:

```rust
pub struct SlickSheetData {
    pub title: String,
    pub subtitle: Option<String>,
    pub body: String,
    pub style: Option<StyleHints>,
    pub contact: Option<ContactInfo>,
    pub sections: Vec<Section>,
    pub features: Vec<String>,
    pub stats: Vec<Stat>,
    pub metadata: HashMap<String, String>,
}
```

Each field is a potential template variable.

---

## Default Template

The `DEFAULT_TEMPLATE` used by the editor:

```typst
#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "{{style.fontFamily | default: 'Inter'}}", size: 11pt)

= #link("cmd://edit/title")[{{title}}]

{{#if subtitle}}
_#link("cmd://edit/subtitle")[{{subtitle}}]_
{{/if}}

#link("cmd://edit/body")[{{body}}]

{{#if features}}
== Features
{{#each features}}
- {{this}}
{{/each}}
{{/if}}

{{#if sections}}
{{#each sections}}
== {{this}}
{{/each}}
{{/if}}

{{#if stats}}
#v(1em)
#table(
  columns: {{stats.length}},
  stroke: none,
  align: center,
  {{#each stats}}[#text(size: 24pt, weight: "bold", fill: rgb("{{style.primaryColor | default: '#e94560'}}"))[{{this}}]],{{/each}}
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

**Features:**
- 8.5" x 11" page with 0.75" margins (standard letter)
- Title as level-1 heading with edit link
- Conditional subtitle (italicized)
- Body content
- Feature list (if present)
- Sections (if present)
- Stats table with dynamic column count (if present)
- Contact footer (if present)
- Style variables with defaults (Inter font, #e94560 primary color)

---

## Test Coverage

### Test Files

1. **`parser.rs` - inline tests:** 11 tests
   - Simple text parsing
   - Simple and nested variables
   - Variables with defaults
   - If/if-else blocks
   - Each loops
   - Mixed content
   - Error cases (unclosed tags)

2. **`engine.rs` - inline tests:** 11 tests
   - Simple text rendering
   - Simple and nested variables
   - Variables with defaults
   - Conditionals (true/false/else)
   - Loops
   - Contact info rendering
   - Stats length
   - Complex templates

3. **`validation.rs` - inline tests:** 6 tests
   - Valid templates
   - Empty templates
   - Syntax errors
   - Known variables
   - Unknown variable warnings
   - Complex templates with all features

4. **`tests.rs` - integration tests:** 11 tests
   - Full product sheet template
   - Conditional rendering with/without data
   - Loop rendering
   - Nested conditionals
   - Default values
   - Stats rendering
   - Empty arrays
   - Parser preserves Typst syntax
   - Complex sections template
   - Whitespace preservation

5. **`write_template.rs` - integration tests:** 6 tests
   - Valid templates
   - Templates with conditionals
   - Templates with loops
   - Invalid syntax detection
   - Empty template rejection
   - Complex template validation

**Total Test Count:** ~45 tests across the module and integrations

**Coverage Assessment:** The module has comprehensive coverage of:
- All parsing syntax variants
- All rendering operations
- Variable resolution paths
- Conditional evaluation
- Loop iteration
- Error handling
- Integration with data structures
- Integration with editor
- Integration with AI tools

---

## Performance Considerations

### Parsing
- **Hand-written recursive descent parser** (not regex-based)
- Single pass over input string
- Position tracking for errors
- **O(n)** complexity where n = template length

### Rendering
- AST traversal (already parsed)
- String concatenation via mutable buffer (efficient)
- Variable resolution: O(1) for simple paths, O(depth) for nested
- Array handling: O(array_size) for each loop
- **Overall: O(n * m)** where n = template nodes, m = loop iterations

### Memory
- AST nodes are small enums (4-5 variants)
- String allocations minimal (use `&str` internally)
- Loop context uses references (no deep copying)

### Caching Opportunities
- Could cache parsed AST if rendering same template multiple times
- Currently re-parsed on each `render()` call
- Not critical since templates are typically small

---

## Error Handling

The module uses two error types:

### `ParseError`
- Returned from `parse_template()`
- Contains position info for debugging
- All variants implement `Display` and `Error` traits

### `TemplateValidationError`
- Returned from `validate_template()`
- Variants for parse errors, unknown variables, empty templates
- Implements `From<ParseError>`

### Rendering Errors
- Returned as `Vec<String>` from `TemplateEngine::render()`
- Multiple errors can be collected (though currently just parse errors)
- Empty vec means success

---

## Design Patterns

### 1. Context-Aware Rendering
The `LoopContext` struct enables:
- Access to current item via `this`
- Access to loop index via `@index`
- Support for nested loops (parent context tracking)

### 2. Graceful Degradation
Variables with missing data:
- Use default value if provided: `{{field | default: 'fallback'}}`
- Use empty string if no default
- Conditional blocks treat missing as `false`

### 3. Typst Preservation
Raw text is passed through unchanged:
- Preserves all Typst syntax (`#set`, `#text`, etc.)
- Template engine is document-agnostic
- Could be used with other markup languages

### 4. Type-Safe Variable Resolution
- `Vec<String>` paths instead of string keys
- Type checking at parse time
- Clear error messages with positions

---

## Known Limitations and Future Improvements

### Current Limitations

1. **No advanced filters:** Only `default` filter implemented
   - Could add: `uppercase`, `lowercase`, `capitalize`, `date` formatting

2. **Limited arithmetic:** Cannot do math operations
   - Could add: `{{count - 1}}`, `{{price * qty}}`

3. **No nested loops with custom items:** Loops only iterate arrays
   - Could support: Object/dict iteration with `{{@key}}` and `{{@value}}`

4. **No template partials:** Cannot include one template in another
   - Could add: `{{> partial_name}}`

5. **Array filtering:** Cannot filter/transform arrays
   - Could add: `{{#each features where condition}}`

6. **Comment syntax:** No way to add comments
   - Could add: `{{! comment text }}`

### Potential Improvements

1. **Performance:** Cache parsed templates when used repeatedly
2. **Error recovery:** Continue rendering after non-critical errors
3. **Debugging:** Add line/column tracking for template errors
4. **Validation:** More precise unknown variable warnings (suggest alternatives)
5. **Documentation:** Auto-generate variable reference from data schema

---

## Quick Reference

### Template Syntax Cheat Sheet

```typst
{{title}}                              # Variable substitution
{{style.primaryColor}}                 # Nested path
{{name | default: 'Guest'}}           # Default value

{{#if condition}}                      # Conditional block
  content here
{{/if}}

{{#if condition}}                      # If-else block
  then content
{{else}}
  else content
{{/if}}

{{#each array}}                        # Loop array
  {{this}}  {{@index}}                # Current item, index
{{/each}}

{{#each features}}                     # Real example: feature list
- {{this}}
{{/each}}
```

### Variable Reference

**Always Available:**
- `title`, `subtitle`, `body`
- `sections`, `features`, `stats`, `contact`, `style`
- `sections.length`, `features.length`, `stats.length`

**In Loops:**
- `this` - current item
- `@index` - zero-based position

**Contact Info:**
- `contact.email`, `contact.phone`, `contact.website`, `contact.address`

**Style:**
- `style.primaryColor` / `style.primary_color`
- `style.accentColor` / `style.accent_color`
- `style.fontFamily` / `style.font_family`

**Custom:**
- Any key in `data.metadata` map

---

## Summary

The template module provides a **lightweight, type-safe, and well-tested** template engine that:

1. **Separates concerns:** Templates (formatting) vs. Data (content)
2. **Enables AI-driven customization:** AI agents can write/modify templates
3. **Validates thoroughly:** Syntax checking and test rendering
4. **Integrates seamlessly:** Works with editor, AI tools, and data model
5. **Preserves Typst:** Raw Typst syntax passes through unchanged
6. **Has comprehensive tests:** 45+ tests covering all features

The module is production-ready with clear error messages, efficient rendering, and good test coverage.
