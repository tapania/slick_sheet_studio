//! AI Settings modal component
//!
//! This component provides a modal for configuring AI settings:
//! - API key for OpenRouter
//! - Model selection
//! - Max iterations for agent loop

use leptos::*;
use wasm_bindgen::JsCast;

/// Available AI models (valid OpenRouter model IDs)
pub const AI_MODELS: &[(&str, &str, &str)] = &[
    (
        "google/gemini-3-flash-preview",
        "Gemini 3 Flash",
        "Fast & Free",
    ),
    (
        "anthropic/claude-3.5-haiku",
        "Claude 3.5 Haiku",
        "Fast & Quality",
    ),
    ("openai/gpt-4o-mini", "GPT-4o Mini", "Balanced"),
    (
        "anthropic/claude-3.5-sonnet",
        "Claude 3.5 Sonnet",
        "Best Quality",
    ),
];

/// Settings stored in localStorage
#[derive(Debug, Clone, Default)]
pub struct AiSettings {
    /// API key for OpenRouter
    pub api_key: String,
    /// Selected model ID
    pub model: String,
    /// Max iterations for agent loop (1-10)
    pub max_iterations: u8,
}

/// Get localStorage if available
fn get_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

impl AiSettings {
    /// Load settings from localStorage
    pub fn load() -> Self {
        let Some(storage) = get_storage() else {
            return Self::default_settings();
        };

        let api_key = storage
            .get_item("slick_ai_api_key")
            .ok()
            .flatten()
            .unwrap_or_default();

        let mut model = storage
            .get_item("slick_ai_model")
            .ok()
            .flatten()
            .unwrap_or_else(|| AI_MODELS[0].0.to_string());

        // Migrate old invalid model IDs to new valid ones
        model = Self::migrate_model_id(&model);

        let max_iterations = storage
            .get_item("slick_ai_max_iterations")
            .ok()
            .flatten()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);

        let settings = Self {
            api_key,
            model,
            max_iterations,
        };

        // Save migrated settings if model was changed
        settings.save();

        settings
    }

    /// Migrate old invalid model IDs to new valid ones
    fn migrate_model_id(model: &str) -> String {
        match model {
            // Old models -> new valid models
            "google/gemini-3-flash" => "google/gemini-3-flash-preview".to_string(),
            "google/gemini-2.0-flash-exp:free" => "google/gemini-3-flash-preview".to_string(),
            "anthropic/claude-4.5-haiku" => "anthropic/claude-3.5-haiku".to_string(),
            "openai/gpt-5.2-mini" => "openai/gpt-4o-mini".to_string(),
            "anthropic/claude-sonnet-4" => "anthropic/claude-3.5-sonnet".to_string(),
            // Keep valid models as-is
            other => other.to_string(),
        }
    }

    /// Save settings to localStorage
    pub fn save(&self) {
        let Some(storage) = get_storage() else {
            return;
        };

        let _ = storage.set_item("slick_ai_api_key", &self.api_key);
        let _ = storage.set_item("slick_ai_model", &self.model);
        let _ = storage.set_item("slick_ai_max_iterations", &self.max_iterations.to_string());
    }

    /// Create default settings
    fn default_settings() -> Self {
        Self {
            api_key: String::new(),
            model: AI_MODELS[0].0.to_string(),
            max_iterations: 3,
        }
    }

    /// Check if API key is configured
    pub fn has_api_key(&self) -> bool {
        !self.api_key.trim().is_empty()
    }
}

/// Settings modal component
#[component]
pub fn SettingsModal(on_save: Callback<AiSettings>, on_close: Callback<()>) -> impl IntoView {
    // Load current settings
    let initial_settings = AiSettings::load();

    // Local state
    let api_key = create_rw_signal(initial_settings.api_key);
    let model = create_rw_signal(initial_settings.model);
    let max_iterations = create_rw_signal(initial_settings.max_iterations);

    // Handle save
    let handle_save = move |_| {
        let settings = AiSettings {
            api_key: api_key.get(),
            model: model.get(),
            max_iterations: max_iterations.get(),
        };
        settings.save();
        on_save.call(settings);
    };

    // Handle escape key
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            on_close.call(());
        }
    };

    view! {
        <div
            class="settings-modal-overlay"
            on:click=move |_| on_close.call(())
            on:keydown=on_keydown
        >
            <div class="settings-modal-content" on:click=|ev| ev.stop_propagation()>
                <div class="settings-modal-header">
                    <h3 class="settings-modal-title">"AI Settings"</h3>
                    <button class="settings-modal-close" on:click=move |_| on_close.call(())>
                        {"\u{00D7}"}
                    </button>
                </div>

                <div class="settings-modal-body">
                    // API Key
                    <div class="settings-field">
                        <label class="settings-label">"API Key (OpenRouter)"</label>
                        <input
                            type="password"
                            class="settings-input"
                            placeholder="sk-or-..."
                            prop:value=move || api_key.get()
                            on:input=move |ev| {
                                api_key.set(event_target_value(&ev));
                            }
                        />
                        <a
                            class="settings-help-link"
                            href="https://openrouter.ai/keys"
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            "Get an API key"
                        </a>
                    </div>

                    // Model selector
                    <div class="settings-field">
                        <label class="settings-label">"Model"</label>
                        <select
                            class="settings-select"
                            on:change=move |ev| {
                                let target = ev.target().unwrap();
                                let select = target.dyn_ref::<web_sys::HtmlSelectElement>().unwrap();
                                model.set(select.value());
                            }
                        >
                            {AI_MODELS.iter().map(|(id, name, desc)| {
                                let selected = model.get() == *id;
                                view! {
                                    <option value=*id selected=selected>
                                        {format!("{} ({})", name, desc)}
                                    </option>
                                }
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>

                    // Max iterations
                    <div class="settings-field">
                        <label class="settings-label">
                            {move || format!("Max Iterations: {}", max_iterations.get())}
                        </label>
                        <input
                            type="range"
                            class="settings-range"
                            min="1"
                            max="10"
                            prop:value=move || max_iterations.get().to_string()
                            on:input=move |ev| {
                                if let Ok(val) = event_target_value(&ev).parse::<u8>() {
                                    max_iterations.set(val);
                                }
                            }
                        />
                        <div class="settings-range-labels">
                            <span>"1"</span>
                            <span>"10"</span>
                        </div>
                    </div>
                </div>

                <div class="settings-modal-footer">
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
            .settings-modal-overlay {
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

            .settings-modal-content {
                background: var(--bg-secondary);
                border-radius: 8px;
                width: 90%;
                max-width: 450px;
                box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
            }

            .settings-modal-header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 1rem 1.5rem;
                border-bottom: 1px solid var(--border);
            }

            .settings-modal-title {
                font-size: 1.125rem;
                font-weight: 600;
                margin: 0;
            }

            .settings-modal-close {
                background: none;
                border: none;
                color: var(--text-secondary);
                font-size: 1.5rem;
                cursor: pointer;
                padding: 0;
                line-height: 1;
            }

            .settings-modal-close:hover {
                color: var(--text-primary);
            }

            .settings-modal-body {
                padding: 1.5rem;
            }

            .settings-field {
                margin-bottom: 1.25rem;
            }

            .settings-field:last-child {
                margin-bottom: 0;
            }

            .settings-label {
                display: block;
                font-size: 0.875rem;
                font-weight: 500;
                margin-bottom: 0.5rem;
                color: var(--text-primary);
            }

            .settings-input,
            .settings-select {
                width: 100%;
                padding: 0.75rem;
                background: var(--bg-primary);
                border: 1px solid var(--border);
                border-radius: 4px;
                color: var(--text-primary);
                font-size: 0.9rem;
                font-family: inherit;
                box-sizing: border-box;
            }

            .settings-input:focus,
            .settings-select:focus {
                outline: none;
                border-color: var(--accent);
            }

            .settings-select {
                cursor: pointer;
            }

            .settings-help-link {
                display: inline-block;
                margin-top: 0.5rem;
                font-size: 0.8rem;
                color: var(--accent);
                text-decoration: none;
            }

            .settings-help-link:hover {
                text-decoration: underline;
            }

            .settings-range {
                width: 100%;
                margin: 0.5rem 0;
            }

            .settings-range-labels {
                display: flex;
                justify-content: space-between;
                font-size: 0.75rem;
                color: var(--text-secondary);
            }

            .settings-modal-footer {
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
