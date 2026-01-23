//! Editor module - Split-pane editor with live preview
//!
//! This module provides the main editor interface with:
//! - Split pane layout (code left, preview right, AI chat right)
//! - Content model for structured editing
//! - Debounced auto-preview or manual refresh
//! - Template gallery for quick start
//! - Save/Load/Export functionality
//! - Click-to-edit in preview
//! - AI-powered code generation
//! - Settings for AI configuration
//! - Status bar with online/offline indicator

mod chat_panel;
mod content;
mod edit_modal;
mod image_gallery;
mod image_generator;
mod image_upload;
mod links;
mod settings_modal;
mod state;
mod status_bar;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub use content::Content;
#[allow(unused_imports)]
pub use links::{parse_cmd_url, EditCommand};
pub use state::{EditorState, EditorTab};

use chat_panel::{AiProcessingState, ChatMessage, ChatPanel};
use edit_modal::{get_field_label, get_field_type, EditFieldData, EditModal};
use image_gallery::{copy_to_clipboard, ImageGallery};
use image_generator::ImageGeneratorPanel;
use image_upload::ImageUpload;
use settings_modal::{AiSettings, SettingsModal};
use status_bar::{use_online_status, StatusBar};

use leptos::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::ai::client::ChatMessage as AiChatMessage;
use crate::ai::{OpenRouterClient, OpenRouterConfig};
use crate::images::{ImageCache, ImageMetadata, ImageStore};
use crate::persistence::{pdf_data_url, Project};
use crate::template::TemplateEngine;
use crate::templates::TEMPLATES;
use crate::world::VirtualWorld;

/// Main Editor component with split pane layout
#[component]
pub fn Editor() -> impl IntoView {
    // Create editor state
    let state = EditorState::new();

    // Get signals from state
    let active_tab = state.active_tab;
    let content_data = state.content_data;
    let template_source = state.template_source;
    let typst_source = state.typst_source;
    let svg_output = state.svg_output;
    let error = state.error;
    let auto_preview = state.auto_preview;

    // Modal states
    let show_template_gallery = create_rw_signal(false);
    let show_settings_modal = create_rw_signal(false);
    let show_edit_modal = create_rw_signal(Option::<EditFieldData>::None);
    let project_name = create_rw_signal("Untitled Project".to_string());
    let status_message = create_rw_signal(Option::<String>::None);
    let last_saved = create_rw_signal(Option::<String>::None);

    // AI settings
    let ai_settings = create_rw_signal(AiSettings::load());
    let has_api_key = create_memo(move |_| ai_settings.get().has_api_key());

    // Online status
    let connection_status = use_online_status();
    let is_online = create_memo(move |_| connection_status.get().is_online());

    // Chat panel state
    let chat_collapsed = create_rw_signal(false);
    let chat_messages = create_rw_signal(Vec::<ChatMessage>::new());
    let processing_state = create_rw_signal(AiProcessingState::Ready);
    let current_iteration = create_rw_signal(0_usize);
    let max_iterations_signal = create_memo(move |_| ai_settings.get().max_iterations as usize);

    // Image state
    let image_store = create_rw_signal(Option::<ImageStore>::None);
    let image_cache = create_rw_signal(ImageCache::new());
    let images_list = create_rw_signal(Vec::<ImageMetadata>::new());

    // Initialize image store on mount
    spawn_local(async move {
        match ImageStore::open().await {
            Ok(store) => {
                // Load existing images
                if let Ok(images) = store.list_images().await {
                    // Pre-load images into cache
                    let mut cache = ImageCache::new();
                    if let Err(e) = cache.preload_all(&store).await {
                        web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&format!(
                            "Failed to preload images: {}",
                            e
                        )));
                    }
                    image_cache.set(cache);
                    images_list.set(images);
                }
                image_store.set(Some(store));
            }
            Err(e) => {
                web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&format!(
                    "Failed to open image store: {}",
                    e
                )));
            }
        }
    });

    // Compile function (with image support)
    let compile = move || {
        let source = typst_source.get();
        let cache = image_cache.get();
        match VirtualWorld::compile_to_svg_with_images(&source, &cache) {
            Ok(svg) => {
                svg_output.set(Some(svg));
                error.set(None);
            }
            Err(errors) => {
                error.set(Some(errors.join("\n")));
            }
        }
    };

    // Initial compile
    compile();

    // Debounce handle
    let debounce_handle = create_rw_signal(Option::<i32>::None);

    // Handle source changes with debounce
    let on_source_change = move |new_source: String| {
        typst_source.set(new_source);

        if auto_preview.get() {
            // Clear existing timeout
            if let Some(handle) = debounce_handle.get() {
                if let Some(window) = web_sys::window() {
                    window.clear_timeout_with_handle(handle);
                }
            }

            // Set new timeout
            if let Some(window) = web_sys::window() {
                let closure =
                    wasm_bindgen::closure::Closure::once(Box::new(compile) as Box<dyn FnOnce()>);
                if let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    300,
                ) {
                    debounce_handle.set(Some(handle));
                }
                closure.forget();
            }
        }
    };

    // Handle template selection - load both template source AND default JSON data
    let on_template_select = Callback::new(move |template_id: String| {
        // Find the template by ID
        if let Some(template) = TEMPLATES.iter().find(|t| t.id == template_id) {
            // Load the raw Typst source
            typst_source.set(template.source.to_string());

            // Load the default JSON data for this template
            let default_data = crate::data::default_data_for_template(&template_id);
            content_data.set(default_data);

            // Reset AI chat when creating new document
            chat_messages.set(Vec::new());
            processing_state.set(AiProcessingState::Ready);

            // Switch to Typst tab to show the loaded template
            active_tab.set(EditorTab::Typst);
        }
        show_template_gallery.set(false);
        compile();
    });

    // Handle settings save
    let on_settings_save = Callback::new(move |settings: AiSettings| {
        ai_settings.set(settings);
        show_settings_modal.set(false);
        status_message.set(Some("Settings saved!".to_string()));
        clear_status_after_delay(status_message);
    });

    // Handle edit modal save
    let on_edit_save = Callback::new(move |(field_id, new_value): (String, String)| {
        // Get current source and update the field
        let source = typst_source.get();
        if let Some(updated_source) = update_field_in_source(&source, &field_id, &new_value) {
            typst_source.set(updated_source);
            compile();
        }
        show_edit_modal.set(None);
    });

    // Handle save
    let on_save = move |_| {
        let project = Project::from_source(project_name.get(), typst_source.get());
        match project.to_json_pretty() {
            Ok(json) => {
                trigger_download(
                    &json,
                    &format!("{}.json", project_name.get()),
                    "application/json",
                );
                let now = get_current_time();
                last_saved.set(Some(now));
                status_message.set(Some("Project saved!".to_string()));
                clear_status_after_delay(status_message);
            }
            Err(e) => {
                status_message.set(Some(format!("Save failed: {}", e)));
            }
        }
    };

    // Handle load
    let on_load = move |_| {
        trigger_file_load(move |content| match Project::from_json(&content) {
            Ok(project) => {
                project_name.set(project.metadata.name);
                typst_source.set(project.source);
                compile();
                status_message.set(Some("Project loaded!".to_string()));
                clear_status_after_delay(status_message);
            }
            Err(e) => {
                status_message.set(Some(format!("Load failed: {}", e)));
            }
        });
    };

    // Handle PDF export
    let on_export_pdf = move |_| {
        let source = typst_source.get();
        match pdf_data_url(&source) {
            Ok(data_url) => {
                trigger_download_url(&data_url, &format!("{}.pdf", project_name.get()));
                status_message.set(Some("PDF exported!".to_string()));
                clear_status_after_delay(status_message);
            }
            Err(e) => {
                status_message.set(Some(format!("Export failed: {}", e)));
            }
        }
    };

    // Handle AI chat send - uses tool-based editing (JSON + Template) with retry
    let on_chat_send = Callback::new(move |prompt: String| {
        // Add user message to history
        chat_messages.update(|msgs| {
            msgs.push(ChatMessage::user(prompt.clone()));
        });

        // Start processing
        processing_state.set(AiProcessingState::Generating);
        current_iteration.set(1);

        // Get settings and current state
        let settings = ai_settings.get();
        let current_data = content_data.get();
        let current_template = template_source.get();
        let max_retries = settings.max_iterations as usize;
        let available_images = images_list.get();
        let current_image_cache = image_cache.get();

        // Validate API key before starting
        if settings.api_key.trim().is_empty() {
            chat_messages.update(|msgs| {
                msgs.push(ChatMessage::error(
                    "No API key configured. Please add your OpenRouter API key in Settings."
                        .to_string(),
                ));
            });
            processing_state.set(AiProcessingState::Failed);
            return;
        }

        // Spawn async task for AI processing
        spawn_local(async move {
            // Log for debugging
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                "Starting AI request (tool-based) with model: {}",
                settings.model
            )));

            let config = OpenRouterConfig::with_key(settings.api_key.clone());
            let client = OpenRouterClient::new(config);

            // Serialize current JSON data
            let current_json =
                serde_json::to_string_pretty(&current_data).unwrap_or_else(|_| "{}".to_string());

            // Build available images info for the prompt
            let images_info = if available_images.is_empty() {
                "No images available. User can upload images in the Images tab.".to_string()
            } else {
                let image_list: Vec<String> = available_images
                    .iter()
                    .map(|img| {
                        let size_str = if img.size < 1024 {
                            format!("{} B", img.size)
                        } else if img.size < 1024 * 1024 {
                            format!("{:.1} KB", img.size as f64 / 1024.0)
                        } else {
                            format!("{:.1} MB", img.size as f64 / (1024.0 * 1024.0))
                        };

                        // Get extension from mime type
                        let ext = crate::images::extension_from_mime_type(&img.mime_type);
                        let full_path = format!("{}.{}", img.id, ext);

                        let mut desc = format!(
                            "  - Path: \"{}\" (filename: {}, {})",
                            full_path, img.filename, size_str
                        );

                        // Add alt description if available
                        if let Some(alt) = &img.alt_description {
                            desc.push_str(&format!("\n    Alt: \"{}\"", alt));
                        }

                        // Add truncated prompt if available
                        if let Some(prompt) = &img.generation_prompt {
                            let truncated = if prompt.len() > 100 {
                                format!("{}...", &prompt[..100])
                            } else {
                                prompt.clone()
                            };
                            desc.push_str(&format!("\n    Prompt: \"{}\"", truncated));
                        }

                        desc
                    })
                    .collect();
                format!("Available images:\n{}", image_list.join("\n"))
            };

            // Build the system prompt for content editing
            let system_prompt = format!(
                r##"You are editing a marketing document. You receive:
1. TEMPLATE - Shows the layout structure (read-only)
2. JSON - The content you will edit
3. AVAILABLE IMAGES - Images the user has uploaded

CRITICAL RULES:

1. USE SPECIFIC DETAILS FROM THE REQUEST
   - If user mentions a name, use that exact name
   - If user mentions a company, use that company name
   - Never substitute generic placeholders

2. FORBIDDEN CHARACTERS (these break the document):
   @ # $ * _ [ ] < > \ {{ }} (in text, not JSON structure)
   - Write "at" instead of @ in emails
   - Write "percent" instead of %

3. MATCH CONTENT TO TEMPLATE STRUCTURE
   The template has these fields:
   - title: Main headline
   - subtitle: Tagline under title
   - body: Description paragraph
   - features: Bullet point list (array of strings)
   - stats: Key metrics shown as big numbers (array of {{value, label}})
   - contact: Footer info ({{email, website}})
   - images: Map of semantic names to image IDs (for adding images)

   If the user asks for something that doesnt fit (like team bios), adapt creatively:
   - For team pages: title=team name, body=team description, features=team member names and roles
   - For events: title=event name, stats=date/time/location
   - Always use what the template provides

4. ADDING IMAGES
   The template supports these image slots:
   - "logo": Small image (60x60pt) in the header next to the title
   - "hero": Full-width banner image below the header

   Add images using the "images" field with the slot name and FULL IMAGE PATH (including extension):
   "images": {{
     "logo": "img_abc123.png",
     "hero": "img_def456.jpg"
   }}

   IMPORTANT: Use the EXACT path from the AVAILABLE IMAGES list, including the file extension (.png, .jpg, etc).
   If user asks to add an image, pick the most appropriate one from the list based on the Alt description.

5. WRITE REAL CONTENT, NOT FILLER
   BAD: "Expertise in scalable solutions and innovative approaches"
   GOOD: "Built the core rendering engine in Rust"

6. OUTPUT FORMAT
   Return ONLY valid JSON. No markdown, no explanation.
   Start with {{ end with }}

{}

EXAMPLE - Adding a logo image to a design:
{{
  "title": "Company Name",
  "subtitle": "Our Amazing Product",
  "body": "Description of what we offer",
  "images": {{
    "logo": "img_abc123.png"
  }}
}}"##,
                images_info
            );

            // Retry loop
            let mut attempt = 0;
            let mut last_error: Option<String> = None;
            let mut last_response: Option<String> = None;

            while attempt < max_retries {
                attempt += 1;
                current_iteration.set(attempt);

                // Build the user prompt - include template, JSON, and images for context
                let user_prompt = if let Some(ref err) = last_error {
                    format!(
                        r#"CURRENT TEMPLATE (read-only, for context):
```
{}
```

CURRENT JSON (edit this):
```json
{}
```

{}

REQUEST: {}

ERROR FROM PREVIOUS ATTEMPT: {}
Previous response was: {}

Fix the error and return ONLY valid JSON (starting with {{ and ending with }}):"#,
                        current_template,
                        current_json,
                        images_info,
                        prompt,
                        err,
                        last_response.as_deref().unwrap_or("N/A")
                    )
                } else {
                    format!(
                        r#"CURRENT TEMPLATE (read-only, for context):
```
{}
```

CURRENT JSON (edit this):
```json
{}
```

{}

REQUEST: {}

Return the updated JSON:"#,
                        current_template, current_json, images_info, prompt
                    )
                };

                // Send to LLM
                let messages = vec![
                    AiChatMessage::system(system_prompt.to_string()),
                    AiChatMessage::user(user_prompt),
                ];

                match client.chat(&settings.model, messages).await {
                    Ok(response) => {
                        last_response = Some(response.clone());

                        // Try to parse the response as JSON
                        let json_str = response.trim();
                        // Remove markdown code fences if present
                        let json_str = json_str
                            .strip_prefix("```json")
                            .or_else(|| json_str.strip_prefix("```"))
                            .unwrap_or(json_str);
                        let json_str = json_str.strip_suffix("```").unwrap_or(json_str).trim();

                        match serde_json::from_str::<crate::data::SlickSheetData>(json_str) {
                            Ok(new_data) => {
                                // Validate the data
                                if let Err(errors) = crate::data::validate_schema(&new_data) {
                                    let error_msg = errors
                                        .iter()
                                        .map(|e| e.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    last_error = Some(format!("Invalid data: {}", error_msg));
                                    chat_messages.update(|msgs| {
                                        msgs.push(ChatMessage::assistant(format!(
                                            "Attempt {}/{}: Validation error, retrying...",
                                            attempt, max_retries
                                        )));
                                    });
                                    continue;
                                }

                                // Render the template with new data
                                match TemplateEngine::render(&current_template, &new_data) {
                                    Ok(rendered_typst) => {
                                        // Try to compile (use image cache for image support)
                                        processing_state.set(AiProcessingState::Compiling);
                                        match VirtualWorld::compile_to_svg_with_images(
                                            &rendered_typst,
                                            &current_image_cache,
                                        ) {
                                            Ok(svg) => {
                                                // Success! Update all the signals
                                                content_data.set(new_data);
                                                typst_source.set(rendered_typst);
                                                svg_output.set(Some(svg));
                                                error.set(None);

                                                // Switch to Content tab so user sees their data
                                                active_tab.set(EditorTab::Content);

                                                processing_state.set(AiProcessingState::Complete);
                                                chat_messages.update(|msgs| {
                                                    msgs.push(ChatMessage::assistant(format!(
                                                        "Done! Updated the content (attempt {}).",
                                                        attempt
                                                    )));
                                                });
                                                // Success - exit retry loop
                                                break;
                                            }
                                            Err(compile_errors) => {
                                                last_error = Some(format!(
                                                    "Compilation failed: {}",
                                                    compile_errors.join(", ")
                                                ));
                                                chat_messages.update(|msgs| {
                                                    msgs.push(ChatMessage::assistant(format!(
                                                        "Attempt {}/{}: Compilation error, retrying...", attempt, max_retries
                                                    )));
                                                });
                                                continue;
                                            }
                                        }
                                    }
                                    Err(render_errors) => {
                                        last_error = Some(format!(
                                            "Template render failed: {}",
                                            render_errors.join(", ")
                                        ));
                                        chat_messages.update(|msgs| {
                                            msgs.push(ChatMessage::assistant(format!(
                                                "Attempt {}/{}: Template error, retrying...",
                                                attempt, max_retries
                                            )));
                                        });
                                        continue;
                                    }
                                }
                            }
                            Err(parse_err) => {
                                // Log the raw response for debugging
                                web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
                                    &format!(
                                        "Failed to parse AI response as JSON: {}\nResponse was: {}",
                                        parse_err, json_str
                                    ),
                                ));
                                last_error = Some(format!("Invalid JSON: {}", parse_err));
                                chat_messages.update(|msgs| {
                                    msgs.push(ChatMessage::assistant(format!(
                                        "Attempt {}/{}: JSON parse error, retrying...",
                                        attempt, max_retries
                                    )));
                                });
                                continue;
                            }
                        }
                    }
                    Err(err) => {
                        // API error - don't retry, just fail
                        chat_messages.update(|msgs| {
                            msgs.push(ChatMessage::error(format!("API Error: {}", err)));
                        });
                        processing_state.set(AiProcessingState::Failed);
                        break;
                    }
                }
            }

            // If we exhausted all retries without success
            if attempt >= max_retries && last_error.is_some() {
                chat_messages.update(|msgs| {
                    msgs.push(ChatMessage::error(format!(
                        "Failed after {} attempts. Last error: {}",
                        max_retries,
                        last_error.unwrap_or_default()
                    )));
                });
                processing_state.set(AiProcessingState::Failed);
            }

            // Reset to ready after a short delay
            if let Some(window) = web_sys::window() {
                let closure = Closure::once(Box::new(move || {
                    processing_state.set(AiProcessingState::Ready);
                }) as Box<dyn FnOnce()>);
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    1000,
                );
                closure.forget();
            }
        });
    });

    // Handle image upload success
    let on_image_upload = Callback::new(move |metadata: ImageMetadata| {
        // Add to images list
        images_list.update(|list| {
            list.insert(0, metadata.clone());
        });

        // Add to cache
        if let Some(store) = image_store.get() {
            let id = metadata.id.clone();
            let mime_type = metadata.mime_type.clone();
            spawn_local(async move {
                if let Ok(data) = store.get_image_data(&id).await {
                    let ext = crate::images::extension_from_mime_type(&mime_type);
                    image_cache.update(|cache| {
                        cache.add(id, data, ext.to_string());
                    });
                }
            });
        }

        status_message.set(Some(format!("Uploaded: {}", metadata.filename)));
        clear_status_after_delay(status_message);
    });

    // Handle image upload error
    let on_image_error = Callback::new(move |err: String| {
        status_message.set(Some(format!("Image error: {}", err)));
        clear_status_after_delay(status_message);
    });

    // Handle image selection (copy ID to clipboard)
    let on_image_select = Callback::new(move |metadata: ImageMetadata| {
        let id = metadata.id.clone();
        copy_to_clipboard(&id);
        status_message.set(Some(format!("Copied image ID: {}", id)));
        clear_status_after_delay(status_message);
    });

    // Handle image deletion
    let on_image_delete = Callback::new(move |id: String| {
        let id_clone = id.clone();
        if let Some(store) = image_store.get() {
            spawn_local(async move {
                if let Err(e) = store.delete_image(&id_clone).await {
                    status_message.set(Some(format!("Delete failed: {}", e)));
                } else {
                    // Remove from list
                    images_list.update(|list| {
                        list.retain(|img| img.id != id_clone);
                    });
                    // Remove from cache
                    image_cache.update(|cache| {
                        cache.clear();
                    });
                    // Reload cache
                    if store.list_images().await.is_ok() {
                        let mut cache = ImageCache::new();
                        let _ = cache.preload_all(&store).await;
                        image_cache.set(cache);
                    }
                    status_message.set(Some("Image deleted".to_string()));
                }
                clear_status_after_delay(status_message);
            });
        }
    });

    // Handle preview click for cmd:// links
    let on_preview_click = move |ev: web_sys::MouseEvent| {
        if let Some(target) = ev.target() {
            if let Some(element) = target.dyn_ref::<web_sys::Element>() {
                // Walk up the DOM to find an <a> element
                let mut current: Option<web_sys::Element> = Some(element.clone());
                while let Some(el) = current {
                    if el.tag_name().to_lowercase() == "a" {
                        if let Some(href) = el.get_attribute("href") {
                            if let Some(cmd) = parse_cmd_url(&href) {
                                ev.prevent_default();
                                // Get current value for the field
                                let source = typst_source.get();
                                let field_id = match &cmd {
                                    EditCommand::Title => "title",
                                    EditCommand::Subtitle => "subtitle",
                                    EditCommand::Body => "body",
                                    EditCommand::Image => "image",
                                    EditCommand::Metadata(key) => {
                                        // Return early with the metadata key
                                        let meta_field_id = format!("meta/{}", key);
                                        let current_value =
                                            extract_field_value(&source, &meta_field_id);
                                        show_edit_modal.set(Some(EditFieldData {
                                            label: get_field_label(&meta_field_id),
                                            value: current_value,
                                            field_type: get_field_type(&meta_field_id),
                                            field_id: meta_field_id,
                                        }));
                                        return;
                                    }
                                };
                                let current_value = extract_field_value(&source, field_id);
                                show_edit_modal.set(Some(EditFieldData {
                                    label: get_field_label(field_id),
                                    value: current_value,
                                    field_type: get_field_type(field_id),
                                    field_id: field_id.to_string(),
                                }));
                                return;
                            }
                        }
                        break;
                    }
                    current = el.parent_element();
                }
            }
        }
    };

    view! {
        <div class="editor-container">
            <header class="toolbar">
                <h1 class="app-title">"Slick Sheet Studio"</h1>
                <div class="toolbar-actions">
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| show_template_gallery.set(true)
                        title="New from template"
                    >
                        "New"
                    </button>
                    <button
                        class="btn btn-secondary"
                        on:click=on_load
                        title="Open project"
                    >
                        "Open"
                    </button>
                    <button
                        class="btn btn-secondary"
                        on:click=on_save
                        title="Save project"
                    >
                        "Save"
                    </button>
                    <button
                        class="btn btn-secondary"
                        on:click=on_export_pdf
                        title="Export as PDF"
                    >
                        "Export PDF"
                    </button>
                    <span class="separator" />
                    <label class="toggle-label">
                        <input
                            type="checkbox"
                            prop:checked=move || auto_preview.get()
                            on:change=move |ev| {
                                let checked = event_target_checked(&ev);
                                auto_preview.set(checked);
                            }
                        />
                        " Auto-preview"
                    </label>
                    <button
                        class="btn btn-primary"
                        on:click=move |_| compile()
                    >
                        "Refresh"
                    </button>
                    <span class="separator" />
                    <button
                        class="btn btn-secondary btn-icon"
                        on:click=move |_| show_settings_modal.set(true)
                        title="AI Settings"
                    >
                        "\u{2699}"
                    </button>
                </div>
            </header>

            <main class="main-content">
                <div class="split-pane">
                    <div class="code-pane">
                        <div class="tab-header">
                            <button
                                class=move || if active_tab.get() == EditorTab::Content { "tab-btn active" } else { "tab-btn" }
                                on:click=move |_| active_tab.set(EditorTab::Content)
                            >
                                "Content"
                            </button>
                            <button
                                class=move || if active_tab.get() == EditorTab::Template { "tab-btn active" } else { "tab-btn" }
                                on:click=move |_| active_tab.set(EditorTab::Template)
                            >
                                "Template"
                            </button>
                            <button
                                class=move || if active_tab.get() == EditorTab::Typst { "tab-btn active" } else { "tab-btn" }
                                on:click=move |_| active_tab.set(EditorTab::Typst)
                            >
                                "Typst"
                            </button>
                            <button
                                class=move || if active_tab.get() == EditorTab::Images { "tab-btn active" } else { "tab-btn" }
                                on:click=move |_| active_tab.set(EditorTab::Images)
                            >
                                "Images"
                            </button>
                        </div>
                        {move || match active_tab.get() {
                            EditorTab::Content => {
                                let json_source = create_rw_signal(
                                    serde_json::to_string_pretty(&content_data.get()).unwrap_or_default()
                                );
                                view! {
                                    <JsonEditor
                                        source=json_source
                                        on_change=move |new_json: String| {
                                            if let Ok(data) = serde_json::from_str::<crate::data::SlickSheetData>(&new_json) {
                                                content_data.set(data);
                                                // Re-render template
                                                let template = template_source.get();
                                                if let Ok(rendered) = crate::template::TemplateEngine::render(&template, &content_data.get()) {
                                                    typst_source.set(rendered);
                                                }
                                            }
                                        }
                                    />
                                }.into_view()
                            }
                            EditorTab::Template => {
                                view! {
                                    <CodeEditor
                                        source=template_source
                                        on_change=move |new_template: String| {
                                            template_source.set(new_template.clone());
                                            // Re-render with current data
                                            if let Ok(rendered) = crate::template::TemplateEngine::render(&new_template, &content_data.get()) {
                                                typst_source.set(rendered);
                                            }
                                        }
                                    />
                                }.into_view()
                            }
                            EditorTab::Typst => {
                                view! {
                                    <CodeEditor
                                        source=typst_source
                                        on_change=on_source_change
                                    />
                                }.into_view()
                            }
                            EditorTab::Images => {
                                view! {
                                    <div class="images-panel">
                                        <ImageUpload
                                            on_upload=on_image_upload
                                            on_error=on_image_error
                                            store=image_store
                                        />
                                        <ImageGeneratorPanel
                                            on_generate=on_image_upload
                                            on_error=on_image_error
                                            store=image_store
                                            api_key=Signal::derive(move || ai_settings.get().api_key)
                                        />
                                        <ImageGallery
                                            images=images_list
                                            image_cache=image_cache
                                            on_select=on_image_select
                                            on_delete=on_image_delete
                                        />
                                    </div>
                                }.into_view()
                            }
                        }}
                    </div>

                    <div class="preview-pane">
                        <div class="pane-header">"Preview (click to edit)"</div>
                        <Preview
                            svg=svg_output.into()
                            error=error.into()
                            on_click=on_preview_click
                        />
                    </div>
                </div>

                // AI Chat Panel
                <ChatPanel
                    collapsed=chat_collapsed
                    has_api_key=has_api_key.into()
                    is_online=is_online.into()
                    messages=chat_messages
                    processing_state=processing_state.into()
                    current_iteration=current_iteration.into()
                    max_iterations=max_iterations_signal.into()
                    on_send=on_chat_send
                />
            </main>

            // Status Bar
            <StatusBar
                connection_status=connection_status.into()
                project_name=project_name.into()
                last_saved=last_saved.into()
                status_message=status_message.into()
            />

            // Template Gallery Modal
            {move || show_template_gallery.get().then(|| view! {
                <TemplateGalleryModal
                    on_select=on_template_select
                    on_close=Callback::new(move |_| show_template_gallery.set(false))
                />
            })}

            // Settings Modal
            {move || show_settings_modal.get().then(|| view! {
                <SettingsModal
                    on_save=on_settings_save
                    on_close=Callback::new(move |_| show_settings_modal.set(false))
                />
            })}

            // Edit Modal
            {move || show_edit_modal.get().map(|field_data| view! {
                <EditModal
                    field_data=field_data
                    on_save=on_edit_save
                    on_close=Callback::new(move |_| show_edit_modal.set(None))
                />
            })}
        </div>

        <style>
            r#"
            .editor-container {
                display: flex;
                flex-direction: column;
                height: 100vh;
                overflow: hidden;
            }

            .toolbar {
                display: flex;
                align-items: center;
                justify-content: space-between;
                padding: 0.75rem 1rem;
                background: var(--bg-secondary);
                border-bottom: 1px solid var(--border);
            }

            .app-title {
                font-size: 1.25rem;
                font-weight: 600;
                color: var(--accent);
            }

            .toolbar-actions {
                display: flex;
                align-items: center;
                gap: 0.5rem;
            }

            .separator {
                width: 1px;
                height: 24px;
                background: var(--border);
                margin: 0 0.5rem;
            }

            .toggle-label {
                display: flex;
                align-items: center;
                gap: 0.25rem;
                font-size: 0.875rem;
                color: var(--text-secondary);
                cursor: pointer;
            }

            .btn {
                padding: 0.5rem 1rem;
                border: none;
                border-radius: 4px;
                cursor: pointer;
                font-size: 0.875rem;
                font-weight: 500;
                transition: all 0.2s;
            }

            .btn-icon {
                padding: 0.5rem;
                font-size: 1.1rem;
            }

            .btn-primary {
                background: var(--accent);
                color: white;
            }

            .btn-primary:hover {
                background: var(--accent-hover);
            }

            .btn-secondary {
                background: var(--bg-tertiary);
                color: var(--text-primary);
                border: 1px solid var(--border);
            }

            .btn-secondary:hover {
                background: var(--border);
            }

            .main-content {
                display: flex;
                flex: 1;
                overflow: hidden;
            }

            .split-pane {
                display: flex;
                flex: 1;
                overflow: hidden;
            }

            .code-pane, .preview-pane {
                flex: 1;
                display: flex;
                flex-direction: column;
                overflow: hidden;
            }

            .code-pane {
                border-right: 1px solid var(--border);
            }

            .pane-header {
                padding: 0.5rem 1rem;
                background: var(--bg-tertiary);
                font-size: 0.75rem;
                font-weight: 600;
                text-transform: uppercase;
                letter-spacing: 0.05em;
                color: var(--text-secondary);
            }

            .tab-header {
                display: flex;
                background: var(--bg-tertiary);
                border-bottom: 1px solid var(--border);
            }

            .tab-btn {
                flex: 1;
                padding: 0.5rem 1rem;
                background: transparent;
                border: none;
                font-size: 0.75rem;
                font-weight: 600;
                text-transform: uppercase;
                letter-spacing: 0.05em;
                color: var(--text-secondary);
                cursor: pointer;
                transition: all 0.2s;
            }

            .tab-btn:hover {
                background: var(--bg-secondary);
                color: var(--text-primary);
            }

            .tab-btn.active {
                background: var(--bg-primary);
                color: var(--accent);
                border-bottom: 2px solid var(--accent);
            }

            /* Images panel */
            .images-panel {
                flex: 1;
                display: flex;
                flex-direction: column;
                overflow: hidden;
                padding: 1rem;
                gap: 1rem;
                background: var(--bg-primary);
            }

            .images-panel > * {
                flex-shrink: 0;
            }

            .images-panel > *:last-child {
                flex: 1;
                overflow-y: auto;
            }

            /* Modal styles */
            .modal-overlay {
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background: rgba(0, 0, 0, 0.7);
                display: flex;
                align-items: center;
                justify-content: center;
                z-index: 1000;
            }

            .modal-content {
                background: var(--bg-secondary);
                border-radius: 8px;
                padding: 1.5rem;
                max-width: 900px;
                max-height: 80vh;
                overflow-y: auto;
                width: 90%;
            }

            .modal-header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                margin-bottom: 1rem;
            }

            .modal-title {
                font-size: 1.25rem;
                font-weight: 600;
            }

            .modal-close {
                background: none;
                border: none;
                color: var(--text-secondary);
                font-size: 1.5rem;
                cursor: pointer;
            }

            .modal-close:hover {
                color: var(--text-primary);
            }

            .template-grid {
                display: grid;
                grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
                gap: 1rem;
            }

            .template-card {
                background: var(--bg-tertiary);
                border-radius: 8px;
                padding: 1rem;
                cursor: pointer;
                transition: all 0.2s;
                border: 2px solid transparent;
            }

            .template-card:hover {
                border-color: var(--accent);
            }

            .template-name {
                font-weight: 600;
                margin-bottom: 0.5rem;
            }

            .template-description {
                font-size: 0.875rem;
                color: var(--text-secondary);
                margin-bottom: 0.5rem;
            }

            .template-category {
                font-size: 0.75rem;
                color: var(--accent);
                text-transform: uppercase;
            }
            "#
        </style>
    }
}

/// Code editor component (textarea wrapper)
#[component]
fn CodeEditor(source: RwSignal<String>, on_change: impl Fn(String) + 'static) -> impl IntoView {
    view! {
        <textarea
            class="code-editor"
            prop:value=move || source.get()
            on:input=move |ev| {
                let value = event_target_value(&ev);
                on_change(value);
            }
            spellcheck="false"
        />

        <style>
            r#"
            .code-editor {
                flex: 1;
                padding: 1rem;
                background: var(--bg-primary);
                color: var(--text-primary);
                border: none;
                resize: none;
                font-family: 'JetBrains Mono', 'Fira Code', monospace;
                font-size: 0.875rem;
                line-height: 1.6;
                outline: none;
            }
            "#
        </style>
    }
}

/// JSON editor component (textarea wrapper with JSON styling)
#[component]
fn JsonEditor(source: RwSignal<String>, on_change: impl Fn(String) + 'static) -> impl IntoView {
    view! {
        <textarea
            class="json-editor"
            prop:value=move || source.get()
            on:input=move |ev| {
                let value = event_target_value(&ev);
                on_change(value);
            }
            spellcheck="false"
        />

        <style>
            r#"
            .json-editor {
                flex: 1;
                padding: 1rem;
                background: var(--bg-primary);
                color: #98c379;
                border: none;
                resize: none;
                font-family: 'JetBrains Mono', 'Fira Code', monospace;
                font-size: 0.875rem;
                line-height: 1.6;
                outline: none;
            }
            "#
        </style>
    }
}

/// Template Gallery Modal component
#[component]
fn TemplateGalleryModal(on_select: Callback<String>, on_close: Callback<()>) -> impl IntoView {
    view! {
        <div class="modal-overlay" on:click=move |_| on_close.call(())>
            <div class="modal-content" on:click=|ev| ev.stop_propagation()>
                <div class="modal-header">
                    <h2 class="modal-title">"Choose a Template"</h2>
                    <button class="modal-close" on:click=move |_| on_close.call(())>"×"</button>
                </div>
                <div class="template-grid">
                    {TEMPLATES.iter().map(|template| {
                        let template_id = template.id.to_string();
                        view! {
                            <div
                                class="template-card"
                                on:click=move |_| on_select.call(template_id.clone())
                            >
                                <div class="template-name">{template.name}</div>
                                <div class="template-description">{template.description}</div>
                                <div class="template-category">{template.category.as_str()}</div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}

/// Trigger a file download in the browser
fn trigger_download(content: &str, filename: &str, _mime_type: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            // Create blob URL using JsValue array
            let array = js_sys::Array::new();
            array.push(&content.into());

            if let Ok(blob) = web_sys::Blob::new_with_str_sequence(&array) {
                if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                    // Create download link
                    if let Ok(link) = document.create_element("a") {
                        let _ = link.set_attribute("href", &url);
                        let _ = link.set_attribute("download", filename);
                        if let Some(body) = document.body() {
                            let _ = body.append_child(&link);
                            if let Some(el) = link.dyn_ref::<web_sys::HtmlElement>() {
                                el.click();
                            }
                            let _ = body.remove_child(&link);
                        }
                        let _ = web_sys::Url::revoke_object_url(&url);
                    }
                }
            }
        }
    }
}

/// Trigger a file download from a data URL
fn trigger_download_url(data_url: &str, filename: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(link) = document.create_element("a") {
                let _ = link.set_attribute("href", data_url);
                let _ = link.set_attribute("download", filename);
                if let Some(body) = document.body() {
                    let _ = body.append_child(&link);
                    if let Some(el) = link.dyn_ref::<web_sys::HtmlElement>() {
                        el.click();
                    }
                    let _ = body.remove_child(&link);
                }
            }
        }
    }
}

/// Trigger a file load dialog
fn trigger_file_load(on_load: impl Fn(String) + 'static) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(input) = document.create_element("input") {
                let _ = input.set_attribute("type", "file");
                let _ = input.set_attribute("accept", ".json");

                let on_load = std::rc::Rc::new(on_load);
                let input_ref = input.clone();

                let closure =
                    wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                        if let Some(input) = input_ref.dyn_ref::<web_sys::HtmlInputElement>() {
                            if let Some(files) = input.files() {
                                if let Some(file) = files.get(0) {
                                    let reader = web_sys::FileReader::new().unwrap();
                                    let on_load = on_load.clone();

                                    let reader_ref = reader.clone();
                                    let onload = wasm_bindgen::closure::Closure::wrap(Box::new(
                                        move |_: web_sys::Event| {
                                            if let Ok(result) = reader_ref.result() {
                                                if let Some(text) = result.as_string() {
                                                    on_load(text);
                                                }
                                            }
                                        },
                                    )
                                        as Box<dyn Fn(_)>);

                                    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                                    onload.forget();

                                    let _ = reader.read_as_text(&file);
                                }
                            }
                        }
                    }) as Box<dyn Fn(_)>);

                let input_el = input.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
                input_el.set_onchange(Some(closure.as_ref().unchecked_ref()));
                closure.forget();

                if let Some(el) = input.dyn_ref::<web_sys::HtmlElement>() {
                    el.click();
                }
            }
        }
    }
}

/// Clear status message after a delay
fn clear_status_after_delay(status: RwSignal<Option<String>>) {
    if let Some(window) = web_sys::window() {
        let closure = wasm_bindgen::closure::Closure::once(Box::new(move || {
            status.set(None);
        }) as Box<dyn FnOnce()>);

        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            3000,
        );
        closure.forget();
    }
}

/// Get current time as a formatted string
fn get_current_time() -> String {
    js_sys::Date::new_0()
        .to_locale_time_string("en-US")
        .as_string()
        .unwrap_or_else(|| "just now".to_string())
}

/// Extract a field value from Typst source (simplified extraction)
fn extract_field_value(source: &str, field_id: &str) -> String {
    // This is a simplified extraction - in a real app, you'd parse the Typst AST
    // Look for link patterns with cmd://edit/{field_id}
    // Typst syntax: #link("url")[content]
    let pattern = format!("cmd://edit/{}", field_id);

    for line in source.lines() {
        if line.contains(&pattern) {
            // Try to extract content between )[ and ]
            // For Typst syntax: #link("cmd://edit/title")[Hello World]
            if let Some(start) = line.find(")[") {
                let after_bracket = &line[start + 2..];
                if let Some(end) = after_bracket.find(']') {
                    return after_bracket[..end].to_string();
                }
            }
        }
    }

    String::new()
}

/// Update a field value in Typst source (simplified update)
fn update_field_in_source(source: &str, field_id: &str, new_value: &str) -> Option<String> {
    let pattern = format!("cmd://edit/{}", field_id);
    let mut result = String::new();
    let mut modified = false;

    for line in source.lines() {
        if line.contains(&pattern) {
            // Replace the content between ][ and the next ]
            // Note: Typst uses #link() so we search for #link or link (to handle both cases)
            let link_pattern = format!("#link(\"{}\")", pattern);
            if let Some(link_start) = line.find(&link_pattern) {
                let before_link = &line[..link_start];
                let link_part = &line[link_start..];

                if let Some(bracket_pos) = link_part.find(")[") {
                    let after_bracket = &link_part[bracket_pos + 2..];
                    if let Some(end_bracket) = after_bracket.find(']') {
                        let after_end = &after_bracket[end_bracket..];
                        let new_line = format!(
                            "{}#link(\"{}\")[{}{}",
                            before_link, pattern, new_value, after_end
                        );
                        result.push_str(&new_line);
                        result.push('\n');
                        modified = true;
                        continue;
                    }
                }
            }
        }
        result.push_str(line);
        result.push('\n');
    }

    if modified {
        // Remove trailing newline
        result.pop();
        Some(result)
    } else {
        None
    }
}

/// Preview component for SVG output
#[component]
fn Preview(
    svg: Signal<Option<String>>,
    error: Signal<Option<String>>,
    on_click: impl Fn(web_sys::MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <div class="preview-content" on:click=on_click>
            {move || {
                if let Some(err) = error.get() {
                    view! {
                        <div class="error-display">
                            <strong>"Compilation Error:"</strong>
                            <pre>{err}</pre>
                        </div>
                    }.into_view()
                } else if let Some(svg_content) = svg.get() {
                    view! {
                        <div class="svg-container" inner_html=svg_content />
                    }.into_view()
                } else {
                    view! {
                        <div class="loading-preview">"Compiling..."</div>
                    }.into_view()
                }
            }}
        </div>

        <style>
            r#"
            .preview-content {
                flex: 1;
                overflow: auto;
                background: white;
                padding: 1rem;
                cursor: pointer;
            }

            .svg-container {
                display: flex;
                justify-content: center;
            }

            .svg-container svg {
                max-width: 100%;
                height: auto;
                box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
            }

            .svg-container a {
                cursor: pointer;
                text-decoration: none;
            }

            .svg-container a:hover {
                opacity: 0.8;
            }

            .error-display {
                padding: 1rem;
                background: #fff0f0;
                border: 1px solid var(--error);
                border-radius: 4px;
                color: #333;
            }

            .error-display pre {
                margin-top: 0.5rem;
                white-space: pre-wrap;
                font-family: monospace;
                font-size: 0.875rem;
                color: var(--error);
            }

            .loading-preview {
                display: flex;
                align-items: center;
                justify-content: center;
                height: 100%;
                color: var(--text-secondary);
            }
            "#
        </style>
    }
}
