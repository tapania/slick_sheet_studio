# Persistence Module Analysis

The persistence module in `/Users/taala/repos/slick_sheet_studio/src/persistence/` is a lean, focused implementation for saving, loading, and exporting Slick Sheet projects. Here's the detailed breakdown:

## Module Structure

The module consists of 4 files:
1. **mod.rs** - Module declaration and public API
2. **project.rs** - Project data structures and JSON serialization
3. **export.rs** - PDF generation functionality
4. **tests.rs** - Comprehensive test suite

## Key Components

### 1. Project Data Structure (project.rs)

The module defines two main data structures:

**ProjectMetadata:**
```rust
pub struct ProjectMetadata {
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub created_at: Option<String>,      // ISO 8601 format
    pub modified_at: Option<String>,     // ISO 8601 format
}
```

**Project:**
```rust
pub struct Project {
    pub metadata: ProjectMetadata,
    pub source: String,  // Typst source code
}
```

### 2. Serialization Methods

The Project struct provides:
- `to_json_pretty()` - Pretty-printed JSON format
- `to_json()` - Compact JSON format
- `from_json(&str)` - Deserialize from JSON string
- `touch()` - Update the modified_at timestamp

### 3. PDF Export (export.rs)

Two main export functions:
- `pdf_bytes_from_source(source: &str) -> Result<Vec<u8>, String>` - Converts Typst source to PDF bytes
- `pdf_data_url(source: &str) -> Result<String, String>` - Creates a data URL for direct download

The export pipeline:
1. Create a VirtualWorld from Typst source
2. Compile using `typst::compile()`
3. Generate PDF using `typst_pdf::pdf()`
4. Encode to base64 for data URL

### 4. Storage Mechanisms

**Current Implementation (File-Based Downloads):**
- Projects saved as `.json` files via browser download
- PDF exports via data URLs
- No persistent storage - files must be manually saved/loaded

**Separate: AI Settings (localStorage)**
Located in `src/editor/settings_modal.rs`, not in persistence module:
- `slick_ai_api_key` - OpenRouter API key
- `slick_ai_model` - Selected model (Gemini, Claude, etc.)
- `slick_ai_max_iterations` - Max AI iterations (1-10, default 3)

## Integration Points

The persistence module integrates with:

1. **Editor Module** (src/editor/mod.rs):
   - Line 41: `use crate::persistence::{pdf_data_url, Project};`
   - Save: Creates Project from name and source, serializes to JSON, triggers download
   - Load: File picker reads JSON, deserializes to Project, updates editor state
   - Export PDF: Uses `pdf_data_url()` to generate data URL

2. **VirtualWorld** (src/world/):
   - Used in `pdf_bytes_from_source()` for compilation
   - Provides Typst compilation context

3. **File Download Utilities** (editor/mod.rs):
   - `trigger_download()` - Creates blob and triggers browser download
   - `trigger_download_url()` - Downloads from data URL
   - `trigger_file_load()` - Opens file picker for loading

## Data Formats

**JSON Project Format:**
```json
{
  "metadata": {
    "name": "My Project",
    "description": "Optional description",
    "version": "1.0.0",
    "created_at": "2024-01-15T12:00:00Z",
    "modified_at": "2024-01-15T13:00:00Z"
  },
  "source": "= Hello World\n\nTypst content here..."
}
```

**Default Typst Template:**
```typst
#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "Inter", size: 11pt)

= Hello World

Welcome to Slick Sheet Studio!

Edit this document to create your slick sheet.
```

## Test Coverage

The test suite (`tests.rs`) includes 13 tests:

**Project Serialization Tests (7 tests):**
- `test_project_new_has_defaults` - New projects have required fields
- `test_project_with_name` - Named project creation
- `test_project_serializes_to_json` - JSON serialization
- `test_project_deserializes_from_json` - JSON deserialization
- `test_project_roundtrip` - Serialize -> Deserialize -> Compare
- `test_project_to_json_pretty` - Pretty formatting works
- `test_project_from_json` - From JSON convenience method
- `test_project_from_invalid_json` - Error handling

**Project Metadata Tests (2 tests):**
- `test_metadata_default` - Default values check
- `test_metadata_custom_name` - Custom metadata creation

**PDF Export Tests (3 tests):**
- `test_pdf_export_from_source` - Basic PDF generation
- `test_pdf_export_with_invalid_source` - Error handling for invalid Typst
- `test_pdf_export_with_complex_document` - Complex document compilation

All tests are unit tests that don't require browser APIs or WASM runtime.

## Dependencies

The module uses:
- `serde`, `serde_json` - Serialization/deserialization
- `typst`, `typst-pdf` - Compilation and PDF generation
- `base64` - Data URL encoding
- `chrono` - Timestamp generation (ISO 8601 format)

## Architecture Limitations

1. **No Client-Side Persistence:** Files must be manually saved/loaded. No IndexedDB or localStorage for projects (only AI settings use localStorage)

2. **No PWA Features:** Despite CLAUDE.md mentioning PWA/service worker in persistence roadmap, currently not implemented

3. **Simple Metadata Only:** Projects store metadata (name, description, timestamps) but no thumbnails, tags, or rich project properties

4. **No Validation:** JSON deserialization doesn't validate Typst syntax or content structure

5. **Download-Based Workflow:** Users must manage files themselves - no project history or auto-save

## Future Plans (from CLAUDE.md)

According to the project guidelines, the persistence module should support:
- [ ] File System Access API for better save/load
- [ ] PWA/service worker support
- [ ] Project versioning/history
- [ ] Cloud storage integration (potential)

## Public API Summary

```rust
// Project creation
Project::new() -> Project
Project::with_name(name) -> Project
Project::from_source(name, source) -> Project

// Serialization
project.to_json_pretty() -> Result<String>
project.to_json() -> Result<String>
Project::from_json(&str) -> Result<Project>

// Metadata
project.touch() -> void  // Updates modified_at

// Export
pdf_bytes_from_source(source: &str) -> Result<Vec<u8>>
pdf_data_url(source: &str) -> Result<String>
```

## Code Quality

- All 13 tests pass
- No dead code warnings (uses `#![allow(dead_code)]` appropriately)
- Proper error handling with Result types
- Clear separation of concerns (serialization, export, tests)
- Good test documentation and assertions
