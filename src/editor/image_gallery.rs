//! Image gallery component
//!
//! Displays stored images in a grid with:
//! - Thumbnail previews
//! - Filename and size display
//! - Click to copy ID for use in templates
//! - Delete button

use leptos::*;

use crate::images::{ImageCache, ImageMetadata};

/// Image gallery component
#[component]
pub fn ImageGallery(
    /// List of image metadata to display
    images: RwSignal<Vec<ImageMetadata>>,
    /// Image cache for generating thumbnails
    image_cache: RwSignal<ImageCache>,
    /// Called when an image is selected (copies ID to clipboard)
    on_select: Callback<ImageMetadata>,
    /// Called when delete is clicked
    on_delete: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="image-gallery">
            {move || {
                let image_list = images.get();
                if image_list.is_empty() {
                    view! {
                        <div class="gallery-empty">
                            <p>"No images uploaded yet"</p>
                            <p class="gallery-hint">"Upload images to use them in your templates"</p>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="gallery-grid">
                            {image_list.iter().map(|metadata| {
                                let metadata_clone = metadata.clone();
                                let metadata_for_select = metadata.clone();
                                let id_for_delete = metadata.id.clone();
                                let cache = image_cache.get();
                                let thumbnail_data = cache.get(&metadata.id).map(|bytes| {
                                    create_data_url(&metadata.mime_type, bytes.as_slice())
                                });

                                view! {
                                    <div class="gallery-item">
                                        <div class="gallery-thumbnail">
                                            {if let Some(data_url) = thumbnail_data {
                                                view! {
                                                    <img src=data_url alt=metadata_clone.filename.clone() />
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div class="thumbnail-placeholder">
                                                        <span>"?"</span>
                                                    </div>
                                                }.into_view()
                                            }}
                                        </div>
                                        <div class="gallery-info">
                                            <div class="gallery-filename" title=metadata.filename.clone()>
                                                {truncate_filename(&metadata.filename, 20)}
                                            </div>
                                            <div class="gallery-actions">
                                                <button
                                                    class="gallery-copy-btn"
                                                    on:click=move |_| on_select.call(metadata_for_select.clone())
                                                    title="Copy image ID to clipboard"
                                                >
                                                    "Copy ID"
                                                </button>
                                                <span class="gallery-size">{format_size(metadata.size)}</span>
                                            </div>
                                        </div>
                                        <button
                                            class="gallery-delete"
                                            on:click=move |_| on_delete.call(id_for_delete.clone())
                                            title="Delete image"
                                        >
                                            "x"
                                        </button>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_view()
                }
            }}
        </div>

        <style>
            r#"
            .image-gallery {
                padding: 0.5rem;
            }

            .gallery-empty {
                text-align: center;
                padding: 2rem;
                color: var(--text-secondary);
            }

            .gallery-empty p {
                margin: 0.5rem 0;
            }

            .gallery-hint {
                font-size: 0.75rem;
            }

            .gallery-grid {
                display: grid;
                grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
                gap: 0.75rem;
            }

            .gallery-item {
                background: var(--bg-tertiary);
                border-radius: 6px;
                overflow: hidden;
                position: relative;
                transition: all 0.2s;
            }

            .gallery-item:hover {
                box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
            }

            .gallery-thumbnail {
                aspect-ratio: 1;
                overflow: hidden;
                cursor: pointer;
                display: flex;
                align-items: center;
                justify-content: center;
                background: var(--bg-primary);
            }

            .gallery-thumbnail:hover {
                opacity: 0.9;
            }

            .gallery-thumbnail img {
                width: 100%;
                height: 100%;
                object-fit: contain;
            }

            .thumbnail-placeholder {
                width: 100%;
                height: 100%;
                display: flex;
                align-items: center;
                justify-content: center;
                background: var(--bg-secondary);
                color: var(--text-secondary);
                font-size: 2rem;
            }

            .gallery-info {
                padding: 0.5rem;
            }

            .gallery-filename {
                font-size: 0.75rem;
                font-weight: 500;
                color: var(--text-primary);
                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
            }

            .gallery-actions {
                display: flex;
                justify-content: space-between;
                align-items: center;
                margin-top: 0.25rem;
                gap: 0.5rem;
            }

            .gallery-copy-btn {
                padding: 0.25rem 0.5rem;
                font-size: 0.625rem;
                background: var(--accent);
                color: white;
                border: none;
                border-radius: 3px;
                cursor: pointer;
                font-weight: 500;
            }

            .gallery-copy-btn:hover {
                background: var(--accent-hover);
            }

            .gallery-delete {
                position: absolute;
                top: 0.25rem;
                right: 0.25rem;
                width: 1.25rem;
                height: 1.25rem;
                border: none;
                border-radius: 50%;
                background: rgba(0, 0, 0, 0.6);
                color: white;
                font-size: 0.75rem;
                cursor: pointer;
                opacity: 0;
                transition: opacity 0.2s;
                display: flex;
                align-items: center;
                justify-content: center;
            }

            .gallery-item:hover .gallery-delete {
                opacity: 1;
            }

            .gallery-delete:hover {
                background: var(--error);
            }
            "#
        </style>
    }
}

/// Create a data URL for an image
fn create_data_url(mime_type: &str, bytes: &[u8]) -> String {
    let base64 = base64_encode(bytes);
    format!("data:{};base64,{}", mime_type, base64)
}

/// Simple base64 encoding
fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Format file size for display
fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Truncate filename for display
fn truncate_filename(filename: &str, max_len: usize) -> String {
    if filename.len() <= max_len {
        filename.to_string()
    } else {
        // Keep extension visible
        if let Some(dot_pos) = filename.rfind('.') {
            let ext = &filename[dot_pos..];
            let name_len = max_len.saturating_sub(ext.len() + 3);
            if name_len > 0 {
                format!("{}...{}", &filename[..name_len], ext)
            } else {
                format!("{}...", &filename[..max_len.saturating_sub(3)])
            }
        } else {
            format!("{}...", &filename[..max_len.saturating_sub(3)])
        }
    }
}

/// Copy text to clipboard using the Clipboard API
pub fn copy_to_clipboard(text: &str) {
    if let Some(window) = web_sys::window() {
        let navigator = window.navigator();
        let clipboard: web_sys::Clipboard = navigator.clipboard();
        let text = text.to_string();
        spawn_local(async move {
            let promise = clipboard.write_text(&text);
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
        });
    }
}
