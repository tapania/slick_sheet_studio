# Templates Directory Analysis - Slick Sheet Studio

## Overview

The `/Users/taala/repos/slick_sheet_studio/src/templates/` directory contains the built-in template system for Slick Sheet Studio. This module provides 10 curated marketing and business templates that users can quickly select from to start creating slick sheets.

**Directory Contents:**
- `mod.rs` - Main templates module (19,966 bytes) containing template definitions, gallery, and category system
- `tests.rs` - Comprehensive test suite (5,123 bytes) for template validation and compilation

## Architecture & Core Concepts

### Template Definition

Templates are defined using Rust structs and static data. Each template is a complete Typst document that can be compiled directly to SVG.

#### Template Structure (`Template` struct)

```rust
#[derive(Debug, Clone)]
pub struct Template {
    pub id: &'static str,              // Unique identifier (e.g., "product-sheet")
    pub name: &'static str,            // Display name (e.g., "Product Sheet")
    pub description: &'static str,     // Short description
    pub category: TemplateCategory,    // Category for organization
    pub preview_svg: Option<&'static str>,  // Optional base64-encoded preview (unused)
    pub source: &'static str,          // Complete Typst source code
}
```

**Field Details:**
- **id**: Used for internal identification and URL routing. Must be unique across all templates.
- **name**: User-facing display name shown in the template gallery UI.
- **description**: One-line description explaining the template's purpose.
- **category**: Enum value categorizing the template for filtering and organization.
- **preview_svg**: Reserved for future use; currently set to `None` for all templates.
- **source**: Raw Typst markup. Users can modify this after selection.

### Template Categories

Templates are organized into 5 distinct categories for easier navigation and filtering:

```rust
pub enum TemplateCategory {
    Marketing,  // Product sheets, one-pagers, marketing materials
    Business,   // Case studies, pricing tables, team profiles
    Event,      // Flyers, invitations, event announcements
    Data,       // Charts, infographics, comparison tables
    Minimal,    // Typography-focused, clean designs
}
```

Each category has a `as_str()` method returning display names: `"Marketing"`, `"Business"`, `"Event"`, `"Data"`, `"Minimal"`.

### Template Gallery

The `TemplateGallery` struct provides a queryable interface for accessing templates:

```rust
pub struct TemplateGallery {
    templates: &'static [Template],
}
```

#### Core Methods

1. **`new()` / `default()`** - Creates a gallery with all built-in templates
   ```rust
   let gallery = TemplateGallery::new();
   ```

2. **`templates()`** - Returns slice of all templates
   ```rust
   let all = gallery.templates();  // &[Template; 10]
   ```

3. **`get(id: &str)`** - Retrieves a single template by ID
   ```rust
   let template = gallery.get("product-sheet");  // Option<&Template>
   ```

4. **`by_category(category: TemplateCategory)`** - Filters templates by category
   ```rust
   let marketing = gallery.by_category(TemplateCategory::Marketing);
   // Returns Vec<&Template> with all marketing templates
   ```

5. **`categories()`** - Gets all unique categories used by templates
   ```rust
   let cats = gallery.categories();  // Vec<TemplateCategory>
   ```

### Static Template Store

All 10 templates are stored in a static constant:

```rust
pub static TEMPLATES: &[Template] = &[
    // 10 Template definitions...
];
```

This is compile-time initialized and read-only. No dynamic template loading is implemented.

## The 10 Built-in Templates

### 1. Product Sheet
- **ID:** `"product-sheet"`
- **Category:** Marketing
- **Description:** Single product showcase with specifications
- **Key Features:**
  - Centered header with product name and tagline
  - Two-column layout for overview and features
  - Specifications table
  - Call-to-action button
- **Placeholder Content:**
  - "Product Name" (main title)
  - "Tagline goes here"
  - Feature list items
  - Contact email

### 2. Event Flyer
- **ID:** `"event-flyer"`
- **Category:** Event
- **Description:** Date, time, and location focused event announcement
- **Key Features:**
  - Full-width colored header bar
  - Three-column grid for Date/Time/Location
  - About event and expectations sections
  - RSVP call-to-action
- **Color Scheme:** Dark background (`#1a1a2e`), accent red (`#e94560`)
- **Placeholder Content:**
  - "EVENT NAME"
  - "January 15, 2024" (sample date)
  - "7:00 PM - 10:00 PM" (sample time)

### 3. One-Pager
- **ID:** `"one-pager"`
- **Category:** Marketing
- **Description:** Executive summary style document
- **Key Features:**
  - Company name header with website
  - Problem/Solution/Metrics format
  - Three-column benefit grid
  - Key metrics table with large numbers
  - Why Choose Us section
- **Design Pattern:** Classic B2B marketing one-pager layout
- **Color Accent:** Red (`#e94560`) for metrics highlights

### 4. Comparison Chart
- **ID:** `"comparison-chart"`
- **Category:** Data
- **Description:** Side-by-side feature comparison
- **Key Features:**
  - Centered title with subtitle
  - Feature comparison table (4 columns: Feature, Us, Competitor A, Competitor B)
  - Summary section highlighting advantages
  - Call-to-action button
- **Table Rows:** 7 feature rows (24/7 Support, Cloud Storage, API Access, etc.)

### 5. Case Study
- **ID:** `"case-study"`
- **Category:** Business
- **Description:** Problem, solution, and results format
- **Key Features:**
  - Case study label and headline
  - Industry/company size metadata
  - Three main sections: The Challenge, Our Solution, The Results
  - Solution phases in 2-column layout
  - Results table with large green numbers (`#4ecca3`)
  - Client testimonial quote
- **Structure:** Classic PSR (Problem-Solution-Results) format

### 6. Team Profile
- **ID:** `"team-profile"`
- **Category:** Business
- **Description:** Staff and team member highlights
- **Key Features:**
  - "Meet Our Team" header
  - 2-column grid of team member cards
  - Cards display: name, role (in red), and bio
  - "Join Our Team" section with careers link
- **Card Design:** Light gray background (`#f5f5f5`) with 1em padding and 4pt radius
- **Sample Members:** Jane Smith (CEO), John Doe (CTO), Emily Chen (Design), Michael Brown (Sales)

### 7. Pricing Table
- **ID:** `"pricing-table"`
- **Category:** Business
- **Description:** Tiered pricing display
- **Key Features:**
  - "Simple Pricing" header
  - Three-column pricing grid (Starter, Professional, Enterprise)
  - Professional plan highlighted with dark background and red accent
  - Feature lists for each plan
  - "Get Started" / "Contact Sales" buttons per tier
- **Sample Pricing:**
  - Starter: $9/month
  - Professional: $29/month (marked "MOST POPULAR")
  - Enterprise: Custom pricing
- **Design:** Professional tier distinguished with dark fill (`#1a1a2e`)

### 8. Newsletter
- **ID:** `"newsletter"`
- **Category:** Marketing
- **Description:** Multi-section content layout
- **Key Features:**
  - Dark header bar with "Monthly Newsletter" and date
  - Two-column layout with `#columns(2)`
  - Sections: Featured Article, Quick Updates, Industry News, Upcoming Events, Contact
  - Industry news items in gray boxes
  - Event list with dates
  - Footer with subscription disclaimer
- **Typst Techniques:** Uses `#colbreak()` for column management

### 9. Infographic
- **ID:** `"infographic"`
- **Category:** Data
- **Description:** Data visualization focused layout
- **Key Features:**
  - Full-width red header (`#e94560`)
  - Three-column stat boxes with large numbers
  - Color-coded stats: green (`#4ecca3`), red (`#e94560`), yellow (`#ffd93d`)
  - Key Findings section with colored progress bars
  - Regional distribution list
  - Bottom line summary box
- **Visual Elements:** Colored horizontal bars for representing percentages
- **Sample Stats:** 78% Growth Rate, 2.5M Active Users, $4.2B Market Size

### 10. Minimal
- **ID:** `"minimal"`
- **Category:** Minimal
- **Description:** Clean, typography-focused design
- **Key Features:**
  - Large title (36pt) with subtitle
  - Clean body text layout with generous spacing
  - Two named sections
  - Horizontal divider line (30% width)
  - Contact information footer in gray
- **Typography Focus:** Sets `#set par(leading: 0.8em)` for line spacing
- **Design Philosophy:** Whitespace-focused, minimal visual elements

## Template Format: Typst Markup

All templates are written in **Typst**, a modern typesetting language. Here's what's consistent across templates:

### Page Setup (Universal)

```typst
#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "Inter", size: 11pt)
```

**Standard Page Settings:**
- **Width/Height:** Most templates use 8.5" x 11" (US Letter)
- **Margins:** 0.75in standard (some flyers use 0.5in)
- **Font:** "Inter" at 11pt (smallest 10pt for dense layouts like newsletter)

### Common Typst Features Used

1. **Text Formatting:**
   ```typst
   #text(size: 24pt, weight: "bold")[Title]
   #text(fill: gray)[Subtitle]
   #text(fill: rgb("#e94560"))[Accent text]
   ```

2. **Layout Elements:**
   ```typst
   #align(center)[...]         // Horizontal alignment
   #v(1em)                      // Vertical spacing
   #h(1fr)                      // Horizontal spacing
   #line(length: 100%, ...)     // Divider lines
   #rect(fill: ..., ...)        // Box/container
   ```

3. **Grid Layout:**
   ```typst
   #grid(columns: 2, column-gutter: 2em, [Item 1], [Item 2], ...)
   ```

4. **Columns:**
   ```typst
   #columns(2, gutter: 1.5em)[
       Content...
       #colbreak()
       More content...
   ]
   ```

5. **Tables:**
   ```typst
   #table(
       columns: (2fr, 1fr, 1fr),
       stroke: 0.5pt + gray,
       inset: 10pt,
       [Header 1], [Header 2], [Header 3],
       [Row 1 Col 1], [Row 1 Col 2], ...
   )
   ```

### Color Palette

Templates use a consistent color scheme:
- **Primary Accent:** Red `#e94560` (CTAs, highlights, important metrics)
- **Dark Background:** `#1a1a2e` (headers, dark cards)
- **Success Green:** `#4ecca3` (positive metrics, results)
- **Accent Yellow:** `#ffd93d` (data visualization)
- **Light Gray:** `#f5f5f5` (backgrounds, cards)
- **Medium Gray:** `gray` (subtext, borders)

## Placeholder/Variable Pattern

Templates follow a **find-and-replace** model for customization:
1. Users load a template in the editor
2. Typst source code is displayed and editable
3. Users manually search/replace placeholder text
4. Changes compile live in the preview

### Common Placeholder Patterns

**Company/Product Name:**
- "Product Name" - Replace with actual product
- "Company Name" - Replace with company
- "Client XYZ" - Replace with real client name

**Contact Information:**
- "hello at example.com" - Replace with real email
- "sales at example.com" - Replace with sales email
- "www.example.com" - Replace with real website
- "(555) 123-4567" - Replace with real phone

**Lorem Ipsum Fallback:**
- "Lorem ipsum dolor sit amet..." - Use for filler text
- "Tagline goes here" - Descriptive placeholder

**Sample Data:**
- "January 15, 2024" - Date placeholders with year 2024
- "95%", "2x", "$1M+" - Metric placeholders
- Feature lists marked as "- Feature one with benefit"

## Module Integration & Usage

### 1. Editor Module (`src/editor/mod.rs`)

The primary consumer of the templates module:

```rust
use crate::templates::TEMPLATES;

// In TemplateGalleryModal component (line 723):
fn TemplateGalleryModal(on_select: Callback<String>, on_close: Callback<()>) -> impl IntoView {
    view! {
        <div class="template-grid">
            {TEMPLATES.iter().map(|template| {
                let source = template.source.to_string();
                view! {
                    <div class="template-card" on:click=move |_| on_select.call(source.clone())>
                        <div class="template-name">{template.name}</div>
                        <div class="template-description">{template.description}</div>
                        <div class="template-category">{template.category.as_str()}</div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
```

**Flow:**
1. User clicks "Template Gallery" button
2. Modal opens showing all templates in a grid
3. Each card displays: name, description, category
4. Clicking a template passes its full `source` to `on_select` callback
5. Editor loads the template source into the Typst editor pane

### 2. Compilation & Rendering

After template selection, the source code flows through:

```rust
// In Editor component (line 81-92):
let compile = move || {
    let source = typst_source.get();
    match VirtualWorld::compile_to_svg(&source) {
        Ok(svg) => {
            svg_output.set(Some(svg));
            error.set(None);
        }
        Err(errors) => {
            error.set(Some(errors.join("\n")));
        }
    }
};
```

The template source (Typst) is compiled to SVG by `VirtualWorld::compile_to_svg()` which uses the Typst language compiler.

## Testing

The test suite (`tests.rs`) validates:

### 1. Template Definition Tests
- All required fields are present (id, name, description, source)
- Template category variants exist (5 total)
- Category display names work (`as_str()`)

### 2. Gallery Query Tests
- Gallery contains exactly 10 templates
- `get(id)` retrieves correct template
- `get(id)` returns None for unknown IDs
- `by_category()` filters correctly
- All template IDs are unique
- `categories()` returns all 5 categories

### 3. Compilation Tests
- **CRITICAL:** All 10 templates must compile to valid SVG
- Test iterates through `TEMPLATES` array and calls `VirtualWorld::compile_to_svg()`
- Individual template compilation tested for: product-sheet, event-flyer, one-pager
- SVG output validation (contains "<svg" tag)

### 4. Content Structure Tests
- All templates have `#set page(...)` directive
- At least one template sets fonts with `#set text`

**Test Count:** 18 total tests (100% passing)

## File Structure Summary

```
/Users/taala/repos/slick_sheet_studio/src/templates/
├── mod.rs                  # 19,966 bytes
│   ├── TemplateCategory enum (5 variants)
│   ├── Template struct (6 fields)
│   ├── TEMPLATES: &[Template; 10] (all template definitions)
│   └── TemplateGallery struct + impl (query interface)
│
└── tests.rs               # 5,123 bytes
    ├── Definition tests (5 tests)
    ├── Gallery tests (5 tests)
    ├── Compilation tests (4 tests)
    └── Content structure tests (2 tests)
```

## Design Patterns & Best Practices

### 1. Static Initialization
Templates are defined as compile-time constants. This:
- Prevents runtime overhead
- Ensures templates can't be corrupted
- Allows embedding into WASM binary

### 2. Gallery Pattern
The `TemplateGallery` struct wraps the static array and provides:
- Convenient `.get()` lookup
- `.by_category()` filtering
- `.categories()` enumeration

This separates "storage" (TEMPLATES array) from "access patterns" (gallery methods).

### 3. Raw String Literals
Templates use raw string literals `r##"..."##` to avoid escaping Typst syntax:
```rust
source: r##"#set page(...) ... "##
```
This allows direct Typst code without escape sequences.

### 4. Zero Customization Data
Templates don't use a variable/placeholder system. They rely on:
- Copy-paste user editing
- Search-and-replace workflow
- Live preview for validation

This is simpler than a template engine but less flexible.

## Limitations & Future Enhancements

### Current Limitations
1. **No Variable System:** No handlebars/mustache-style `{{variable}}` interpolation
2. **No Dynamic Templates:** Can't add custom templates at runtime
3. **No Preview Images:** `preview_svg` field unused (all `None`)
4. **No Localization:** All text in English
5. **No Template Versioning:** No way to update templates after deployment

### Possible Future Work
1. Implement `preview_svg` with base64-encoded preview images
2. Add template customization API (variable injection)
3. Support user-created/uploaded templates
4. Add template metadata (target audience, use cases)
5. Implement template recommendations based on user actions
6. Add template categories filtering in editor UI

## Key Takeaways

1. **Complete Template System:** 10 production-ready templates covering marketing, business, events, data, and minimal designs
2. **Typst-Based:** All templates are valid Typst documents with consistent styling
3. **Gallery Pattern:** Clean API for querying and filtering templates
4. **Well-Tested:** 18 tests ensure template validity and compilation
5. **Editor Integration:** Seamlessly integrated into the editor UI for quick starts
6. **Simple Model:** No complex variable system; relies on user editing of Typst source
7. **Color Consistent:** Uses a defined palette (red `#e94560`, dark `#1a1a2e`, etc.) across all templates
