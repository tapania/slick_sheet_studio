//! Editor state management with Leptos signals

use leptos::*;

use crate::data::{default_data_for_template, SlickSheetData};
use crate::template::TemplateEngine;

/// Editor tab enum for the 4-way split
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorTab {
    /// JSON content editor (default)
    #[default]
    Content,
    /// Typst template editor
    Template,
    /// Rendered Typst output (read-only)
    Typst,
    /// Image gallery and upload
    Images,
}

/// Default template with Handlebars placeholders - Modern dark theme
pub const DEFAULT_TEMPLATE: &str = r##"#set page(width: 8.5in, height: 11in, margin: 0.75in, fill: rgb("#0f0f1a"))
#set text(font: "Inter", size: 11pt, fill: rgb("#e8e8e8"))

// Header with optional logo
#rect(width: 100%, fill: rgb("#1a1a2e"), radius: 8pt, inset: 1.2em)[
  #grid(
    columns: (auto, 1fr),
    gutter: 1em,
    align: (horizon, left),
    {{#if images.logo}}
    box(width: 60pt, height: 60pt, clip: true)[#image("{{images.logo}}", width: 100%)],
    {{else}}
    [],
    {{/if}}
    [
      #text(size: 32pt, weight: "bold", fill: white)[{{title}}]
      {{#if subtitle}}
      #v(0.3em)
      #text(size: 14pt, fill: rgb("#e94560"))[{{subtitle}}]
      {{/if}}
    ]
  )
]

#v(1em)

{{#if images.hero}}
#align(center)[
  #box(width: 100%, clip: true, radius: 8pt)[
    #image("{{images.hero}}", width: 100%)
  ]
]
#v(1em)
{{/if}}

{{#if body}}
#text(fill: rgb("#b8b8b8"))[{{body}}]
#v(1em)
{{/if}}

{{#if features}}
#text(size: 14pt, weight: "bold", fill: rgb("#e94560"))[FEATURES]
#v(0.5em)
{{#each features}}
- {{this}}
{{/each}}
#v(1em)
{{/if}}

{{#if stats}}
#v(0.5em)
#table(
  columns: {{stats.length}},
  stroke: none,
  inset: 12pt,
  align: center,
  {{#each stats}}
  [#text(size: 28pt, weight: "bold", fill: rgb("#4ecca3"))[{{this.value}}]],
  {{/each}}
  {{#each stats}}
  [#text(size: 10pt, fill: rgb("#888888"))[{{this.label}}]],
  {{/each}}
)
#v(1em)
{{/if}}

{{#if contact}}
#line(length: 100%, stroke: 0.5pt + rgb("#333344"))
#v(0.5em)
#align(center)[
  #text(size: 10pt, fill: rgb("#666677"))[
    {{#if contact.email}}{{contact.email}}{{/if}}
    {{#if contact.website}} | {{contact.website}}{{/if}}
  ]
]
{{/if}}
"##;

/// Editor state with reactive signals
pub struct EditorState {
    /// Active editor tab
    pub active_tab: RwSignal<EditorTab>,
    /// JSON content data
    pub content_data: RwSignal<SlickSheetData>,
    /// Typst template source with Handlebars placeholders
    pub template_source: RwSignal<String>,
    /// Raw Typst source code (rendered from template + data)
    pub typst_source: RwSignal<String>,
    /// Compiled SVG output
    pub svg_output: RwSignal<Option<String>>,
    /// Compilation error message
    pub error: RwSignal<Option<String>>,
    /// Auto-preview enabled
    pub auto_preview: RwSignal<bool>,
}

impl EditorState {
    /// Create a new editor state with default content
    pub fn new() -> Self {
        // Initialize with default data
        let data = SlickSheetData::new("Hello World").with_body("Welcome to Slick Sheet Studio!");

        // Render the default template with the data
        let typst_source = TemplateEngine::render(DEFAULT_TEMPLATE, &data)
            .unwrap_or_else(|_| "// Template render error".to_string());

        Self {
            active_tab: create_rw_signal(EditorTab::default()),
            content_data: create_rw_signal(data),
            template_source: create_rw_signal(DEFAULT_TEMPLATE.to_string()),
            typst_source: create_rw_signal(typst_source),
            svg_output: create_rw_signal(None),
            error: create_rw_signal(None),
            auto_preview: create_rw_signal(true),
        }
    }

    /// Create editor state with custom source
    #[allow(dead_code)]
    pub fn with_source(source: String) -> Self {
        let data = SlickSheetData::new("Custom Document");

        Self {
            active_tab: create_rw_signal(EditorTab::Typst),
            content_data: create_rw_signal(data),
            template_source: create_rw_signal(DEFAULT_TEMPLATE.to_string()),
            typst_source: create_rw_signal(source),
            svg_output: create_rw_signal(None),
            error: create_rw_signal(None),
            auto_preview: create_rw_signal(true),
        }
    }

    /// Create editor state from a template ID
    #[allow(dead_code)]
    pub fn from_template(template_id: &str) -> Self {
        let data = default_data_for_template(template_id);

        // Render the template with the data
        let typst_source =
            TemplateEngine::render(DEFAULT_TEMPLATE, &data).unwrap_or_else(|_| String::new());

        Self {
            active_tab: create_rw_signal(EditorTab::Content),
            content_data: create_rw_signal(data),
            template_source: create_rw_signal(DEFAULT_TEMPLATE.to_string()),
            typst_source: create_rw_signal(typst_source),
            svg_output: create_rw_signal(None),
            error: create_rw_signal(None),
            auto_preview: create_rw_signal(true),
        }
    }

    /// Render the template with current data and update typst_source
    #[allow(dead_code)]
    pub fn render_template(&self) {
        let template = self.template_source.get();
        let data = self.content_data.get();

        match TemplateEngine::render(&template, &data) {
            Ok(rendered) => {
                self.typst_source.set(rendered);
                self.error.set(None);
            }
            Err(errors) => {
                self.error.set(Some(format!(
                    "Template rendering error:\n{}",
                    errors.join("\n")
                )));
            }
        }
    }

    /// Update the JSON content data
    #[allow(dead_code)]
    pub fn set_content_data(&self, data: SlickSheetData) {
        self.content_data.set(data);
        self.render_template();
    }

    /// Update the template source
    #[allow(dead_code)]
    pub fn set_template_source(&self, template: String) {
        self.template_source.set(template);
        self.render_template();
    }

    /// Get JSON representation of current content
    #[allow(dead_code)]
    pub fn get_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.content_data.get())
            .map_err(|e| format!("Failed to serialize: {}", e))
    }

    /// Update content from JSON string
    #[allow(dead_code)]
    pub fn set_json(&self, json: &str) -> Result<(), String> {
        let data: SlickSheetData =
            serde_json::from_str(json).map_err(|e| format!("Failed to parse JSON: {}", e))?;
        self.set_content_data(data);
        Ok(())
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}
