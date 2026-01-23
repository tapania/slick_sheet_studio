//! Image generator component for AI-powered image creation
//!
//! Provides a UI for generating images from text prompts using
//! the OpenRouter Gemini model.

use leptos::*;

use crate::ai::client::OpenRouterConfig;
use crate::ai::image_gen::{generate_alt_description, ImageGenerator};
use crate::images::{generate_image_id, ImageMetadata, ImageStore};

/// Image generator component with prompt input
#[component]
pub fn ImageGeneratorPanel(
    /// Called when an image is successfully generated
    on_generate: Callback<ImageMetadata>,
    /// Called when an error occurs
    on_error: Callback<String>,
    /// Image store for saving generated images
    store: RwSignal<Option<ImageStore>>,
    /// API key for OpenRouter
    api_key: Signal<String>,
) -> impl IntoView {
    let prompt = create_rw_signal(String::new());
    let is_generating = create_rw_signal(false);

    let has_api_key = create_memo(move |_| !api_key.get().trim().is_empty());

    let on_submit = move |_| {
        let prompt_text = prompt.get();
        let key = api_key.get();

        // Validate
        if prompt_text.trim().is_empty() {
            on_error.call("Please enter a prompt describing the image you want".to_string());
            return;
        }

        if key.trim().is_empty() {
            on_error.call(
                "No API key configured. Please add your OpenRouter API key in Settings."
                    .to_string(),
            );
            return;
        }

        is_generating.set(true);

        spawn_local(async move {
            let config = OpenRouterConfig::with_key(key);
            let generator = ImageGenerator::new(config.clone());

            match generator.generate(&prompt_text).await {
                Ok((bytes, mime_type)) => {
                    // Generate filename
                    let id = generate_image_id();
                    let ext = crate::images::extension_from_mime_type(&mime_type);
                    let filename = format!("generated_{}.{}", &id[4..12], ext);

                    // Generate alt description using Gemini Flash
                    let alt_desc = generate_alt_description(&config, &prompt_text)
                        .await
                        .unwrap_or_else(|_| "AI-generated image".to_string());

                    if let Some(image_store) = store.get() {
                        // Store with prompt and alt description
                        match image_store
                            .store_generated_image(
                                filename.clone(),
                                bytes,
                                prompt_text.clone(),
                                alt_desc,
                            )
                            .await
                        {
                            Ok(metadata) => {
                                on_generate.call(metadata);
                                prompt.set(String::new()); // Clear prompt on success
                            }
                            Err(e) => {
                                on_error.call(format!("Failed to save image: {}", e));
                            }
                        }
                    } else {
                        on_error.call("Image store not initialized".to_string());
                    }
                }
                Err(e) => {
                    on_error.call(format!("Generation failed: {}", e));
                }
            }

            is_generating.set(false);
        });
    };

    view! {
        <div class="image-generator-panel">
            <div class="generator-header">
                <span class="generator-icon">"AI"</span>
                <span class="generator-title">"Generate Image"</span>
            </div>
            <div class="generator-content">
                <textarea
                    class="generator-prompt"
                    placeholder="Describe the image you want to generate..."
                    prop:value=move || prompt.get()
                    on:input=move |ev| prompt.set(event_target_value(&ev))
                    disabled=move || is_generating.get()
                    rows="3"
                />
                <button
                    class="generator-btn"
                    on:click=on_submit
                    disabled=move || {
                        is_generating.get() || prompt.get().trim().is_empty() || !has_api_key.get()
                    }
                >
                    {move || {
                        if is_generating.get() {
                            "Generating..."
                        } else if !has_api_key.get() {
                            "No API Key"
                        } else {
                            "Generate"
                        }
                    }}
                </button>
                {move || (!has_api_key.get()).then(|| view! {
                    <div class="generator-hint">
                        "Add your OpenRouter API key in Settings to enable image generation"
                    </div>
                })}
            </div>
        </div>

        <style>
            r#"
            .image-generator-panel {
                background: var(--bg-tertiary);
                border-radius: 8px;
                overflow: hidden;
            }

            .generator-header {
                display: flex;
                align-items: center;
                gap: 0.5rem;
                padding: 0.75rem 1rem;
                background: var(--bg-secondary);
                border-bottom: 1px solid var(--border);
            }

            .generator-icon {
                background: linear-gradient(135deg, #667eea, #764ba2);
                color: white;
                padding: 0.25rem 0.5rem;
                border-radius: 4px;
                font-size: 0.625rem;
                font-weight: 700;
            }

            .generator-title {
                font-size: 0.875rem;
                font-weight: 600;
                color: var(--text-primary);
            }

            .generator-content {
                padding: 1rem;
                display: flex;
                flex-direction: column;
                gap: 0.75rem;
            }

            .generator-prompt {
                width: 100%;
                padding: 0.75rem;
                border: 1px solid var(--border);
                border-radius: 6px;
                background: var(--bg-primary);
                color: var(--text-primary);
                font-family: inherit;
                font-size: 0.875rem;
                resize: vertical;
                min-height: 60px;
            }

            .generator-prompt:focus {
                outline: none;
                border-color: var(--accent);
            }

            .generator-prompt:disabled {
                opacity: 0.6;
                cursor: not-allowed;
            }

            .generator-prompt::placeholder {
                color: var(--text-secondary);
            }

            .generator-btn {
                padding: 0.75rem 1rem;
                background: linear-gradient(135deg, #667eea, #764ba2);
                color: white;
                border: none;
                border-radius: 6px;
                font-size: 0.875rem;
                font-weight: 600;
                cursor: pointer;
                transition: all 0.2s;
            }

            .generator-btn:hover:not(:disabled) {
                transform: translateY(-1px);
                box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
            }

            .generator-btn:disabled {
                opacity: 0.6;
                cursor: not-allowed;
                transform: none;
                box-shadow: none;
            }

            .generator-hint {
                font-size: 0.75rem;
                color: var(--text-secondary);
                text-align: center;
            }
            "#
        </style>
    }
}
