# Data Module Review

**Location:** `/Users/taala/repos/slick_sheet_studio/src/data/`

**Status:** Well-designed, focused module providing JSON schema and validation for slick sheet documents.

## Executive Summary

The `data` module is a core component that defines and validates the JSON data model for slick sheet documents. It provides:

1. **Schema Definition** - Complete data structures for documents, sections, stats, contact info, and styling
2. **Validation** - Comprehensive validation logic ensuring data integrity
3. **Template Defaults** - Pre-built template data for 10 different document types
4. **Builder Pattern** - Ergonomic, chainable APIs for constructing data programmatically

The module is intentionally minimal and focused - it does NOT handle file I/O, serialization beyond serde, or template rendering. These concerns are delegated to specialized modules.

## Architecture Overview

### Module Structure

```
src/data/
├── mod.rs              - Public API and module exports (24 lines)
├── schema.rs           - Data structures and builders (302 lines)
├── validation.rs       - Validation logic (247 lines)
├── defaults.rs         - Template data generators (354 lines)
└── tests.rs            - Integration tests (187 lines)
```

**Total Lines of Code:** ~1,114 lines

### Module Visibility & Organization

- **`#![allow(dead_code)]`** on schema and defaults - Some builder methods and helpers are not yet used internally, but are exposed for future public API use
- **Validation tests** are embedded in `validation.rs` (116 lines)
- **Default template tests** are embedded in `defaults.rs` (48 lines)
- **Integration tests** in `tests.rs` demonstrate full workflows

## Core Data Structures

### 1. SlickSheetData (Primary Document Model)

**Location:** `schema.rs` lines 13-101

The root data structure representing a complete slick sheet document.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SlickSheetData {
    pub title: String,                              // Required
    pub subtitle: Option<String>,                   // Optional, omitted from JSON if None
    pub body: String,                               // Optional in practice
    pub sections: Vec<Section>,                     // Structured content
    pub metadata: HashMap<String, String>,          // Arbitrary key-value data
    pub features: Vec<String>,                      // Simple feature list
    pub stats: Vec<Stat>,                           // Key metrics/KPIs
    pub contact: Option<ContactInfo>,               // Contact details
    pub style: Option<StyleHints>,                  // Styling preferences
}
```

**Key Features:**

- **Serialization:** Uses serde with `skip_serializing_if = "Option::is_none"` to keep JSON clean (no null values)
- **Default Implementation:** All fields have sensible defaults (empty strings, empty vecs, None for optionals)
- **Builder Pattern:** Fluent API for construction
  - `.new(title)` - Create with title
  - `.with_subtitle()`, `.with_body()`, `.with_section()`, etc.
  - `.with_feature()`, `.with_stat()`, `.with_contact()`, `.with_style()`

**Builder Example:**

```rust
let data = SlickSheetData::new("Product Name")
    .with_subtitle("Amazing Features")
    .with_body("Lorem ipsum...")
    .with_section(Section::list("Features", vec!["Feature 1".into(), "Feature 2".into()]))
    .with_stat(Stat::new("95%", "Uptime").with_color("#e94560"))
    .with_contact(ContactInfo::with_email("sales@example.com"));
```

### 2. Section (Content Sections)

**Location:** `schema.rs` lines 103-184

Represents a structured section within a document. Supports four content types: text, list, table, and quote.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Section {
    pub heading: String,                            // Required
    pub content: String,                            // For text/quote sections
    pub section_type: SectionType,                  // Determines interpretation
    pub items: Option<Vec<String>>,                 // For list sections
    pub rows: Option<Vec<Vec<String>>>,             // For table sections (each row is Vec<String>)
    pub columns: Option<usize>,                     // For table sections
}
```

**Section Type Enum:**

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SectionType {
    #[default]
    Text,    // Serializes as "text"
    List,    // Serializes as "list"
    Table,   // Serializes as "table"
    Quote,   // Serializes as "quote"
}
```

**Builder Methods:**

```rust
Section::text("Heading", "Content text")              // Text section
Section::list("Heading", vec!["Item 1", "Item 2"])   // Bulleted list
Section::table("Heading", vec![vec!["A", "B"]], 2)   // 2-column table
Section::quote("Heading", "Quote content")            // Quote/testimonial
```

**Validation Rules (enforced by schema):**

- All sections must have a heading
- List sections MUST have items (non-empty)
- Table sections MUST have rows AND a column count
- Text and Quote sections don't have additional requirements

### 3. Stat (Key Metrics)

**Location:** `schema.rs` lines 201-230

Represents a key statistic or metric to display.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stat {
    pub value: String,                  // "95%", "$1M", "2x", etc.
    pub label: String,                  // Description of metric
    pub color: Option<String>,          // Hex color for styling
}
```

**Builder Pattern:**

```rust
let stat = Stat::new("95%", "Customer Satisfaction")
    .with_color("#e94560");
```

**Validation Rules:**

- Value must be non-empty
- Label must be non-empty
- Color (if present) must be valid hex format

### 4. ContactInfo

**Location:** `schema.rs` lines 232-260

Contact information for the document.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ContactInfo {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub address: Option<String>,
}
```

**Builder:**

```rust
ContactInfo::with_email("hello@example.com")
```

All fields are optional, and the entire ContactInfo can be omitted from JSON if None.

### 5. StyleHints

**Location:** `schema.rs` lines 262-301

Styling preferences and theme configuration.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StyleHints {
    pub primary_color: Option<String>,      // Hex color
    pub accent_color: Option<String>,       // Hex color
    pub font_family: Option<String>,        // Font name
}
```

**Deserialization Compatibility:**

Supports both camelCase (from JSON) and snake_case (Rust convention):

```rust
#[serde(alias = "primaryColor")]
pub primary_color: Option<String>,
```

**Helper Methods (with safe defaults):**

```rust
style.primary_color_or_default()   // Returns "#e94560" if None
style.accent_color_or_default()    // Returns "#4ecca3" if None
style.font_family_or_default()     // Returns "Inter" if None
```

## Validation Module

**Location:** `validation.rs` (247 lines)

### Validation Engine

```rust
pub fn validate_schema(data: &SlickSheetData) -> Result<(), Vec<ValidationError>>
```

**Design:** Accumulates ALL errors rather than failing on first error, providing comprehensive feedback.

**Validation Error Enum:**

```rust
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ValidationError {
    #[error("Title is required and cannot be empty")]
    EmptyTitle,

    #[error("Section at index {0} is missing a heading")]
    EmptySectionHeading(usize),

    #[error("List section '{0}' has no items")]
    EmptyListItems(String),

    #[error("Table section '{0}' has no rows")]
    EmptyTableRows(String),

    #[error("Table section '{0}' is missing column count")]
    MissingTableColumns(String),

    #[error("Stat at index {0} has empty value")]
    EmptyStatValue(usize),

    #[error("Stat at index {0} has empty label")]
    EmptyStatLabel(usize),

    #[error("Invalid color format: '{0}' (expected hex color like #ffffff)")]
    InvalidColorFormat(String),
}
```

### Validation Rules

1. **Title:** Required, cannot be empty or whitespace-only
2. **Sections:** Each section must have a heading
3. **List Sections:** Must contain at least one item
4. **Table Sections:** Must have rows AND column count
5. **Stats:** Value and label both required; color must be valid hex if present
6. **Style Colors:** Primary and accent colors must be valid hex if present

### Color Validation

```rust
fn is_valid_hex_color(color: &str) -> bool
```

Validates hex colors in two formats:
- 3-digit: `#fff`, `#ABC`
- 6-digit: `#ffffff`, `#abcdef`, `#e94560`

Rejects:
- Missing `#` prefix
- Invalid lengths (not 3 or 6)
- Non-hexadecimal characters

**Example Valid:** `#fff`, `#ffffff`, `#e94560`, `#ABC`, `#abcdef`

**Example Invalid:** `fff` (no #), `#ff` (wrong length), `#gggggg` (invalid chars)

### Test Coverage (validation.rs)

The module includes 14 test cases:

1. `test_valid_data` - Minimal valid data passes
2. `test_empty_title` - Empty title fails
3. `test_empty_section_heading` - Empty section heading fails
4. `test_empty_list_items` - List with no items fails
5. `test_empty_stat_value` - Stat with empty value fails
6-11. `test_invalid_hex_colors` - Various invalid color formats
12-14. `test_valid_hex_colors` - Various valid color formats

## Defaults Module

**Location:** `defaults.rs` (354 lines)

### Purpose

Provides template data for 10 common document types, used when creating new documents from templates.

### Main API

```rust
pub fn default_data_for_template(template_id: &str) -> SlickSheetData
```

**Supported Template IDs:**

| ID | Purpose | Key Fields |
|---|---|---|
| `product-sheet` | Product marketing material | Features table, specifications |
| `event-flyer` | Event promotion | "What to Expect" list |
| `one-pager` | Company overview | Problem/Solution/Why Us sections, 3 stats |
| `comparison-chart` | Competitive comparison | Feature comparison table |
| `case-study` | Customer success story | Challenge/Solution/Testimonial, 3 stats |
| `team-profile` | Team introduction | Individual team member sections |
| `pricing-table` | Pricing plans | Plans and features table |
| `newsletter` | Monthly newsletter | Featured article, updates, events |
| `infographic` | Statistics display | Multiple stats, market trends |
| `minimal` | Bare minimum | 2 text sections, contact |
| *(unknown)* | Fallback | Returns minimal template |

### Template Examples

**Product Sheet Template:**

```rust
SlickSheetData::new("Product Name")
    .with_subtitle("Tagline goes here")
    .with_body("Lorem ipsum...")
    .with_section(Section::list(
        "Key Features",
        vec!["Feature one...", "Feature two...", "Feature three...", "Feature four..."],
    ))
    .with_section(Section::table(
        "Specifications",
        vec![
            vec!["Dimension", "Value here"],
            vec!["Weight", "Value here"],
            vec!["Material", "Value here"],
            vec!["Warranty", "Value here"],
        ],
        2,
    ))
    .with_contact(ContactInfo::with_email("sales@example.com"))
    .with_style(StyleHints { primary_color: Some("#e94560".into()), ..Default::default() })
```

**One-Pager Template:**

```rust
SlickSheetData::new("Company Name")
    .with_section(Section::text("The Problem", "..."))
    .with_section(Section::text("Our Solution", "..."))
    .with_section(Section::list("Why Choose Us", vec![...]))
    .with_stat(Stat::new("95%", "Customer Satisfaction").with_color("#e94560"))
    .with_stat(Stat::new("2x", "Faster Results").with_color("#e94560"))
    .with_stat(Stat::new("$1M+", "Savings Generated").with_color("#e94560"))
    .with_contact(ContactInfo::with_email("hello@example.com"))
```

**Case Study Template:**

Uses all four section types and stats with accent color:

```rust
SlickSheetData::new("Client Success Story")
    .with_subtitle("Industry: Technology | Company Size: Enterprise")
    .with_section(Section::list("The Challenge", vec![...]))
    .with_section(Section::text("Our Solution", "..."))
    .with_section(Section::quote("Testimonial", "Quote text..."))
    .with_stat(Stat::new("40%", "Cost Reduction").with_color("#4ecca3"))
    .with_stat(Stat::new("60%", "Efficiency Gain").with_color("#4ecca3"))
    .with_stat(Stat::new("3x", "ROI").with_color("#4ecca3"))
```

### Test Coverage (defaults.rs)

4 test cases:

1. `test_default_product_sheet` - Product sheet template has required fields
2. `test_default_event_flyer` - Event flyer template name is correct
3. `test_unknown_template_returns_minimal` - Unknown template IDs fall back to minimal
4. `test_all_templates_have_defaults` - All 10 templates have non-empty titles

## Integration with Other Modules

### 1. Editor Module (`src/editor/state.rs`)

**Usage:**
- Initializes `SlickSheetData` as reactive state via `create_rw_signal()`
- Uses `default_data_for_template()` to populate initial content
- Stores data in `EditorState::content_data: RwSignal<SlickSheetData>`

**Example:**
```rust
pub struct EditorState {
    pub content_data: RwSignal<SlickSheetData>,
    // ...
}

impl EditorState {
    pub fn new() -> Self {
        let data = SlickSheetData::new("Hello World")
            .with_body("Welcome to Slick Sheet Studio!");
        Self {
            content_data: create_rw_signal(data),
            // ...
        }
    }
}
```

### 2. Template Engine (`src/template/engine.rs`)

**Usage:**
- Renders templates with `SlickSheetData` as context
- Accesses all data fields: `title`, `subtitle`, `body`, `sections`, `features`, `stats`, `contact`, `style`
- Supports nested path resolution: `style.primaryColor`, `contact.email`, `stats.length`

**Key Functions:**
```rust
pub fn render(template: &str, data: &SlickSheetData) -> Result<String, Vec<String>>
```

Resolves template variables by:
1. Simple paths: `title`, `subtitle`, `body` (direct field access)
2. Nested paths: `style.primaryColor`, `contact.email`, `stats.length`
3. Metadata: Falls back to `data.metadata` HashMap
4. Loop contexts: Special variables like `this` and `@index`

### 3. AI Tools (`src/ai/tools/`)

#### ReadJsonTool
```rust
pub fn execute(data: &SlickSheetData) -> ToolResult
```
- Serializes `SlickSheetData` to pretty JSON
- Used by AI to inspect current content

#### WriteJsonTool
```rust
pub fn execute(
    new_json: &str,
    current_template: &str,
    compile_fn: impl FnOnce(&str) -> Result<String, Vec<String>>,
) -> Result<SlickSheetData, ToolResult>
```
- Parses JSON into `SlickSheetData`
- Calls `validate_schema()` to check correctness
- Tests template compilation with new data
- Returns validated data or detailed errors

**Validation Pipeline:**
1. JSON syntax validation (parse)
2. Schema validation (`validate_schema()`)
3. Template rendering (test)
4. Typst compilation (test)

### 4. File References

**Modules that import from `data`:**

- `src/main.rs` - Module declaration
- `src/editor/state.rs` - Initial data setup
- `src/template/engine.rs` - Template rendering
- `src/template/validation.rs` - Template validation
- `src/ai/tools/read_json.rs` - Read current data
- `src/ai/tools/write_json.rs` - Write and validate new data
- `src/ai/tools/write_template.rs` - Template writing
- `src/ai/tools/tests.rs` - AI tool tests

## Public API Surface

### Exports from `mod.rs`

```rust
// Core structures
pub use schema::{Section, SectionType, SlickSheetData};

// Extended API (not yet used internally)
pub use schema::{ContactInfo, Stat, StyleHints};
pub use validation::{validate_schema, ValidationError};

// Generators
pub use defaults::default_data_for_template;
```

All types implement:
- `Debug` - For diagnostics
- `Clone` - For reactive state
- `Serialize`/`Deserialize` - For JSON I/O
- `PartialEq` - For testing

## JSON Serialization Examples

### Minimal Document

```json
{
  "title": "Just Title"
}
```

Result of `SlickSheetData::new("Just Title")` - only required field is serialized.

### Complete Document

```json
{
  "title": "Product Launch",
  "subtitle": "Revolutionary New Product",
  "body": "Introducing our latest innovation...",
  "sections": [
    {
      "heading": "Features",
      "content": "",
      "type": "list",
      "items": ["Feature 1", "Feature 2"]
    },
    {
      "heading": "Specs",
      "content": "",
      "type": "table",
      "rows": [["Size", "Large"]],
      "columns": 2
    }
  ],
  "metadata": {},
  "features": [],
  "stats": [
    {
      "value": "99%",
      "label": "Uptime",
      "color": "#e94560"
    }
  ],
  "contact": {
    "email": "sales@example.com"
  },
  "style": {
    "primaryColor": "#e94560"
  }
}
```

### Deserialization Flexibility

The schema supports:
- **Optional fields omitted:** `{ "title": "Just Title" }` deserializes correctly
- **Extra fields ignored:** Unknown JSON fields don't cause errors
- **camelCase/snake_case:** Style hints support both `primaryColor` (JSON) and `primary_color` (Rust)

## Testing

### Test Coverage Summary

**Total Tests:** ~35 test cases across module

**Breakdown:**

| File | Tests | Focus |
|------|-------|-------|
| `tests.rs` | 14 | Integration: serialization, builders, types, validation |
| `validation.rs` | 14 | Validation rules and color format checking |
| `defaults.rs` | 4 | Template defaults |
| Embedded | 3 | Other validations in schema |

### Key Test Examples

**Serialization Round-Trip:**

```rust
#[test]
fn test_slick_sheet_data_serialization() {
    let data = SlickSheetData::new("Test Title")
        .with_subtitle("Test Subtitle")
        .with_body("Test body content");

    let json = serde_json::to_string(&data).expect("Should serialize");
    let deserialized: SlickSheetData = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(data, deserialized);
}
```

**Validation with Multiple Errors:**

```rust
#[test]
fn test_validation_multiple_errors() {
    let data = SlickSheetData {
        title: "".to_string(),                    // Empty title error
        sections: vec![Section {
            heading: "".to_string(),              // Empty heading error
            section_type: SectionType::List,
            items: Some(vec![]),                  // Empty list error
            ..Default::default()
        }],
        stats: vec![Stat {
            value: "".to_string(),                // Empty value error
            label: "".to_string(),                // Empty label error
            color: Some("invalid".to_string()),   // Invalid color error
            ..Default::default()
        }],
        ..Default::default()
    };

    let result = validate_schema(&data);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.len() >= 5);
}
```

## Design Patterns & Best Practices

### 1. Builder Pattern

All main types implement builder methods for ergonomic construction:

```rust
SlickSheetData::new("title")
    .with_subtitle("sub")
    .with_body("body")
    .with_section(...)
    .with_stat(...)
```

**Benefit:** Readable, chainable API; optional fields can be skipped.

### 2. Default Trait Implementation

```rust
#[derive(Default)]
pub struct SlickSheetData { ... }
```

**Benefit:** Sensible defaults for all fields; `..Default::default()` pattern for partial updates.

### 3. Validation Accumulation

Validation doesn't fail on first error; it collects all errors:

```rust
pub fn validate_schema(data: &SlickSheetData) -> Result<(), Vec<ValidationError>>
```

**Benefit:** Users get comprehensive feedback about all problems, not just first one.

### 4. Optional Field Handling

Uses `Option<T>` with `skip_serializing_if = "Option::is_none"`:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub subtitle: Option<String>,
```

**Benefit:** JSON stays clean (no null values); deserialization is forgiving (missing fields OK).

### 5. Semantic Type Naming

`SectionType` enum makes intent clear:

```rust
pub enum SectionType { Text, List, Table, Quote }
```

**Benefit:** Not using strings or integers; type-safe section handling.

## Known Limitations & Future Considerations

### Current Limitations

1. **No Schema Versioning:** No version field in `SlickSheetData`. Breaking changes would require migration logic.
2. **Limited Color Validation:** Only validates format (hex); doesn't check if colors are readable/accessible.
3. **No Rich Text:** Body and content fields are plain strings; no markdown/formatting support.
4. **Metadata HashMap:** Untyped key-value store; could lead to runtime errors if keys are misspelled.
5. **No Constraints on List Length:** No max/min size constraints for lists, tables, or stats.

### Future Enhancements

1. **Version Field:** Add `version: u32` to enable migrations.
2. **Markdown Support:** Support markdown in body/content fields with optional rendering.
3. **Schema Constraints:** Add min/max validators for lists and stats.
4. **Accessibility:** Add `aalt_text` field to stats for screen readers.
5. **Advanced Styling:** Expand `StyleHints` with more CSS properties or theming options.

## Dependencies

The module relies on minimal external crates:

- **serde** / **serde_json** - JSON serialization/deserialization
- **thiserror** - Error types (`#[derive(Error)]`)
- **std::collections::HashMap** - Metadata storage

**No async code, no external APIs, no file I/O** - module is purely data-focused.

## Code Quality Observations

### Strengths

1. **Focused Responsibility:** Module does one thing well - define and validate data schema
2. **Comprehensive Tests:** Good test coverage including edge cases (empty title, invalid colors, multiple errors)
3. **Builder Pattern:** Ergonomic API for construction
4. **Clear Documentation:** Inline doc comments on all public types
5. **Type Safety:** Uses enums, Options, and custom types rather than strings/integers
6. **Error Handling:** Detailed error enum with context (section index, field name)

### Areas for Improvement

1. **Embedding Tests:** Schema module has `#[allow(dead_code)]` - some builder methods unused internally. Could remove or use them.
2. **Metadata Usage:** Unclear if metadata HashMap is actively used; could add documentation.
3. **Error Message Consistency:** Some errors include field names, others use indices. Could standardize.
4. **Serde Flexibility:** `alias = "primaryColor"` works but could be more explicit about JSON naming strategy.

## Conclusion

The `data` module is a well-designed, focused component that successfully encodes the domain model for slick sheet documents. It provides:

- **Clear Schema:** Seven well-defined types covering all content needs
- **Strong Validation:** Comprehensive validation with detailed error feedback
- **Practical Defaults:** Ten ready-to-use templates
- **Type Safety:** Leverages Rust's type system for correctness
- **Minimal Dependencies:** Pure data layer with no I/O or external APIs

The module serves as the "single source of truth" for data structure in the application, with clear contracts for serialization, validation, and template rendering. Integration with other modules (editor, template engine, AI tools) is straightforward because the types are simple, well-documented, and have a clear API surface.
