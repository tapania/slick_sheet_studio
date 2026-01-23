//! Edit modal for click-to-edit functionality
//!
//! This component provides a modal dialog for editing fields
//! when users click on editable elements in the preview.

use leptos::*;

/// Types of fields that can be edited
#[derive(Debug, Clone, PartialEq)]
pub enum EditFieldType {
    /// Single line text (title, subtitle)
    SingleLine,
    /// Multi-line text (body)
    MultiLine,
    /// URL input (image)
    Url,
}

/// Data passed to the edit modal
#[derive(Debug, Clone)]
pub struct EditFieldData {
    /// Display label for the field
    pub label: String,
    /// Current value
    pub value: String,
    /// Field type for appropriate input
    pub field_type: EditFieldType,
    /// Field identifier (e.g., "title", "subtitle", "body", "image", "meta/key")
    pub field_id: String,
}

/// Modal component for editing a field value
#[component]
pub fn EditModal(
    field_data: EditFieldData,
    on_save: Callback<(String, String)>,
    on_close: Callback<()>,
) -> impl IntoView {
    let current_value = create_rw_signal(field_data.value.clone());
    let field_id = field_data.field_id.clone();
    let field_id_for_keydown = field_id.clone();
    let is_multiline = field_data.field_type == EditFieldType::MultiLine;

    // Handle save
    let do_save = move |id: String| on_save.call((id, current_value.get()));
    let handle_save = {
        let field_id = field_id.clone();
        move |_| do_save(field_id.clone())
    };

    // Handle key press (Enter to save for single line, Escape to close)
    let on_keydown = move |ev: web_sys::KeyboardEvent| match ev.key().as_str() {
        "Escape" => on_close.call(()),
        "Enter" if !is_multiline => {
            ev.prevent_default();
            do_save(field_id_for_keydown.clone());
        }
        _ => {}
    };

    view! {
        <div class="edit-modal-overlay" on:click=move |_| on_close.call(())>
            <div class="edit-modal-content" on:click=|ev| ev.stop_propagation()>
                <div class="edit-modal-header">
                    <h3 class="edit-modal-title">{format!("Edit {}", field_data.label)}</h3>
                    <button class="edit-modal-close" on:click=move |_| on_close.call(())>
                        {"\u{00D7}"}
                    </button>
                </div>
                <div class="edit-modal-body">
                    {match field_data.field_type {
                        EditFieldType::MultiLine => {
                            view! {
                                <textarea
                                    class="edit-modal-input edit-modal-textarea"
                                    prop:value=move || current_value.get()
                                    on:input=move |ev| {
                                        current_value.set(event_target_value(&ev));
                                    }
                                    on:keydown=on_keydown
                                    autofocus=true
                                />
                            }.into_view()
                        }
                        EditFieldType::Url => {
                            view! {
                                <input
                                    type="url"
                                    class="edit-modal-input"
                                    placeholder="https://example.com/image.png"
                                    prop:value=move || current_value.get()
                                    on:input=move |ev| {
                                        current_value.set(event_target_value(&ev));
                                    }
                                    on:keydown=on_keydown
                                    autofocus=true
                                />
                            }.into_view()
                        }
                        EditFieldType::SingleLine => {
                            view! {
                                <input
                                    type="text"
                                    class="edit-modal-input"
                                    prop:value=move || current_value.get()
                                    on:input=move |ev| {
                                        current_value.set(event_target_value(&ev));
                                    }
                                    on:keydown=on_keydown
                                    autofocus=true
                                />
                            }.into_view()
                        }
                    }}
                </div>
                <div class="edit-modal-footer">
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| on_close.call(())
                    >
                        "Cancel"
                    </button>
                    <button
                        class="btn btn-primary"
                        on:click=handle_save
                    >
                        "Save"
                    </button>
                </div>
            </div>
        </div>

        <style>
            r#"
            .edit-modal-overlay {
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background: rgba(0, 0, 0, 0.7);
                display: flex;
                align-items: center;
                justify-content: center;
                z-index: 1100;
            }

            .edit-modal-content {
                background: var(--bg-secondary);
                border-radius: 8px;
                padding: 0;
                width: 90%;
                max-width: 500px;
                box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
            }

            .edit-modal-header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 1rem 1.5rem;
                border-bottom: 1px solid var(--border);
            }

            .edit-modal-title {
                font-size: 1.125rem;
                font-weight: 600;
                margin: 0;
            }

            .edit-modal-close {
                background: none;
                border: none;
                color: var(--text-secondary);
                font-size: 1.5rem;
                cursor: pointer;
                padding: 0;
                line-height: 1;
            }

            .edit-modal-close:hover {
                color: var(--text-primary);
            }

            .edit-modal-body {
                padding: 1.5rem;
            }

            .edit-modal-input {
                width: 100%;
                padding: 0.75rem;
                background: var(--bg-primary);
                border: 1px solid var(--border);
                border-radius: 4px;
                color: var(--text-primary);
                font-size: 1rem;
                font-family: inherit;
                box-sizing: border-box;
            }

            .edit-modal-input:focus {
                outline: none;
                border-color: var(--accent);
            }

            .edit-modal-textarea {
                min-height: 150px;
                resize: vertical;
                line-height: 1.5;
            }

            .edit-modal-footer {
                display: flex;
                justify-content: flex-end;
                gap: 0.75rem;
                padding: 1rem 1.5rem;
                border-top: 1px solid var(--border);
            }
            "#
        </style>
    }
}

/// Get the field type based on field ID
pub fn get_field_type(field_id: &str) -> EditFieldType {
    match field_id {
        "body" => EditFieldType::MultiLine,
        "image" => EditFieldType::Url,
        _ => EditFieldType::SingleLine,
    }
}

/// Get a display label for a field ID
pub fn get_field_label(field_id: &str) -> String {
    match field_id {
        "title" => "Title".to_string(),
        "subtitle" => "Subtitle".to_string(),
        "body" => "Body Text".to_string(),
        "image" => "Image URL".to_string(),
        s if s.starts_with("meta/") => {
            let key = s.strip_prefix("meta/").unwrap_or(s);
            format!("Metadata: {}", key)
        }
        other => other.to_string(),
    }
}
