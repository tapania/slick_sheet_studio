//! Image upload component with drag-drop support
//!
//! Provides a file upload interface for images with:
//! - Drag-and-drop zone
//! - File picker fallback
//! - Format and size validation
//! - Progress indication

use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::images::{is_supported_extension, ImageMetadata, ImageStore, MAX_IMAGE_SIZE};

/// Image upload component with drag-drop support
#[component]
pub fn ImageUpload(
    on_upload: Callback<ImageMetadata>,
    on_error: Callback<String>,
    store: RwSignal<Option<ImageStore>>,
) -> impl IntoView {
    let is_dragging = create_rw_signal(false);
    let is_uploading = create_rw_signal(false);

    // Handle file selection from input
    let on_file_select = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        let input: web_sys::HtmlInputElement = target.unchecked_into();

        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                handle_file(file, store, on_upload, on_error, is_uploading);
            }
        }
    };

    // Handle drag events
    let on_dragover = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        is_dragging.set(true);
    };

    let on_dragleave = move |_ev: web_sys::DragEvent| {
        is_dragging.set(false);
    };

    let on_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        is_dragging.set(false);

        if let Some(data_transfer) = ev.data_transfer() {
            if let Some(files) = data_transfer.files() {
                if let Some(file) = files.get(0) {
                    handle_file(file, store, on_upload, on_error, is_uploading);
                }
            }
        }
    };

    view! {
        <div
            class=move || {
                let base = "image-upload-zone";
                if is_dragging.get() {
                    format!("{} dragging", base)
                } else if is_uploading.get() {
                    format!("{} uploading", base)
                } else {
                    base.to_string()
                }
            }
            on:dragover=on_dragover
            on:dragleave=on_dragleave
            on:drop=on_drop
        >
            {move || {
                if is_uploading.get() {
                    view! {
                        <div class="upload-status">
                            <span class="upload-spinner"></span>
                            <span>"Uploading..."</span>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="upload-content">
                            <div class="upload-icon">"+"</div>
                            <div class="upload-text">"Drop image here or click to upload"</div>
                            <div class="upload-hint">"PNG, JPEG, SVG, GIF, WebP (max 10MB)"</div>
                            <input
                                type="file"
                                accept="image/png,image/jpeg,image/svg+xml,image/gif,image/webp"
                                class="upload-input"
                                on:change=on_file_select
                            />
                        </div>
                    }.into_view()
                }
            }}
        </div>

        <style>
            r#"
            .image-upload-zone {
                border: 2px dashed var(--border);
                border-radius: 8px;
                padding: 2rem;
                text-align: center;
                cursor: pointer;
                transition: all 0.2s;
                position: relative;
                background: var(--bg-tertiary);
            }

            .image-upload-zone:hover {
                border-color: var(--accent);
                background: var(--bg-secondary);
            }

            .image-upload-zone.dragging {
                border-color: var(--accent);
                background: rgba(78, 204, 163, 0.1);
            }

            .image-upload-zone.uploading {
                border-color: var(--text-secondary);
                cursor: wait;
            }

            .upload-content {
                display: flex;
                flex-direction: column;
                align-items: center;
                gap: 0.5rem;
            }

            .upload-icon {
                font-size: 2rem;
                color: var(--accent);
            }

            .upload-text {
                font-size: 0.875rem;
                color: var(--text-primary);
            }

            .upload-hint {
                font-size: 0.75rem;
                color: var(--text-secondary);
            }

            .upload-input {
                position: absolute;
                top: 0;
                left: 0;
                width: 100%;
                height: 100%;
                opacity: 0;
                cursor: pointer;
            }

            .upload-status {
                display: flex;
                align-items: center;
                justify-content: center;
                gap: 0.5rem;
                color: var(--text-secondary);
            }

            .upload-spinner {
                width: 1rem;
                height: 1rem;
                border: 2px solid var(--border);
                border-top-color: var(--accent);
                border-radius: 50%;
                animation: spin 1s linear infinite;
            }

            @keyframes spin {
                to { transform: rotate(360deg); }
            }
            "#
        </style>
    }
}

/// Handle a selected file
fn handle_file(
    file: web_sys::File,
    store: RwSignal<Option<ImageStore>>,
    on_upload: Callback<ImageMetadata>,
    on_error: Callback<String>,
    is_uploading: RwSignal<bool>,
) {
    // Check file extension
    let filename = file.name();
    let extension = filename.rsplit('.').next().unwrap_or("");
    if !is_supported_extension(extension) {
        on_error.call(format!(
            "Unsupported file format: .{}. Use PNG, JPEG, SVG, GIF, or WebP.",
            extension
        ));
        return;
    }

    // Check file size
    let size = file.size() as usize;
    if size > MAX_IMAGE_SIZE {
        on_error.call(format!(
            "File too large: {} MB. Maximum size is 10 MB.",
            size / (1024 * 1024)
        ));
        return;
    }

    is_uploading.set(true);

    // Read file as ArrayBuffer
    let reader = web_sys::FileReader::new().unwrap();
    let reader_clone = reader.clone();
    let filename_clone = filename.clone();

    let onload = Closure::once(Box::new(move |_event: web_sys::Event| {
        if let Ok(result) = reader_clone.result() {
            let array_buffer: js_sys::ArrayBuffer = result.unchecked_into();
            let uint8_array = js_sys::Uint8Array::new(&array_buffer);
            let data = uint8_array.to_vec();

            // Upload to store
            spawn_local(async move {
                if let Some(image_store) = store.get() {
                    match image_store.store_image(filename_clone, data).await {
                        Ok(metadata) => {
                            on_upload.call(metadata);
                        }
                        Err(err) => {
                            on_error.call(err.to_string());
                        }
                    }
                } else {
                    on_error.call("Image store not initialized".to_string());
                }
                is_uploading.set(false);
            });
        } else {
            on_error.call("Failed to read file".to_string());
            is_uploading.set(false);
        }
    }) as Box<dyn FnOnce(_)>);

    let onerror = Closure::once(Box::new(move |_event: web_sys::Event| {
        on_error.call("Failed to read file".to_string());
        is_uploading.set(false);
    }) as Box<dyn FnOnce(_)>);

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onload.forget();
    onerror.forget();

    let _ = reader.read_as_array_buffer(&file);
}
