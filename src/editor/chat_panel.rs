//! AI Chat Panel component
//!
//! This component provides the AI chat interface for:
//! - Sending prompts to the AI
//! - Displaying chat history
//! - Showing progress during AI processing

use leptos::*;

/// Chat message types
#[derive(Debug, Clone, PartialEq)]
pub enum ChatMessageType {
    /// Message from the user
    User,
    /// Message from the AI assistant
    Assistant,
    /// System message (status, progress)
    System,
    /// Error message
    Error,
}

/// A chat message in the history
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// Message type
    pub message_type: ChatMessageType,
    /// Message content
    pub content: String,
}

impl ChatMessage {
    /// Create a user message
    #[allow(dead_code)]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            message_type: ChatMessageType::User,
            content: content.into(),
        }
    }

    /// Create an assistant message
    #[allow(dead_code)]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            message_type: ChatMessageType::Assistant,
            content: content.into(),
        }
    }

    /// Create a system message
    #[allow(dead_code)]
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            message_type: ChatMessageType::System,
            content: content.into(),
        }
    }

    /// Create an error message
    #[allow(dead_code)]
    pub fn error(content: impl Into<String>) -> Self {
        Self {
            message_type: ChatMessageType::Error,
            content: content.into(),
        }
    }
}

/// Current state of the AI processing
#[derive(Debug, Clone, PartialEq, Default)]
pub enum AiProcessingState {
    /// Ready to accept input
    #[default]
    Ready,
    /// Generating code with LLM
    Generating,
    /// Compiling Typst code
    Compiling,
    /// Verifying output visually (planned for visual verification feature)
    #[allow(dead_code)]
    Verifying,
    /// Processing complete
    Complete,
    /// Processing failed
    Failed,
}

impl AiProcessingState {
    /// Get display text for the state
    pub fn display_text(&self) -> &str {
        match self {
            Self::Ready => "Ready",
            Self::Generating => "Generating code...",
            Self::Compiling => "Compiling...",
            Self::Verifying => "Verifying...",
            Self::Complete => "Complete",
            Self::Failed => "Failed",
        }
    }

    /// Check if processing is active
    pub fn is_processing(&self) -> bool {
        matches!(self, Self::Generating | Self::Compiling | Self::Verifying)
    }
}

/// Chat panel component
#[component]
pub fn ChatPanel(
    /// Whether the panel is collapsed
    collapsed: RwSignal<bool>,
    /// Whether API key is configured
    has_api_key: Signal<bool>,
    /// Whether the app is online
    is_online: Signal<bool>,
    /// Chat message history
    messages: RwSignal<Vec<ChatMessage>>,
    /// Current processing state
    processing_state: Signal<AiProcessingState>,
    /// Current iteration (1-based)
    current_iteration: Signal<usize>,
    /// Max iterations
    max_iterations: Signal<usize>,
    /// Callback when user sends a message
    on_send: Callback<String>,
) -> impl IntoView {
    // Local state for the input
    let input_text = create_rw_signal(String::new());

    // Shared send logic
    let try_send = move || {
        let text = input_text.get();
        if !text.trim().is_empty() && !processing_state.get().is_processing() {
            on_send.call(text);
            input_text.set(String::new());
        }
    };

    // Handle send button click
    let handle_send = move |_| try_send();

    // Handle enter key (Shift+Enter for newline)
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() {
            ev.prevent_default();
            try_send();
        }
    };

    // Toggle collapsed state
    let toggle_collapse = move |_| {
        collapsed.update(|c| *c = !*c);
    };

    view! {
        <div class="chat-panel" class:collapsed=move || collapsed.get()>
            <div class="chat-panel-header" on:click=toggle_collapse>
                <span class="chat-panel-title">"AI Assistant"</span>
                <button class="chat-panel-toggle">
                    {move || if collapsed.get() { "\u{25C0}" } else { "\u{25B6}" }}
                </button>
            </div>

            {move || (!collapsed.get()).then(|| view! {
                <div class="chat-panel-body">
                    // Status banner
                    {move || {
                        if !is_online.get() {
                            Some(view! {
                                <div class="chat-status-banner chat-status-offline">
                                    "AI unavailable offline"
                                </div>
                            })
                        } else if !has_api_key.get() {
                            Some(view! {
                                <div class="chat-status-banner chat-status-warning">
                                    "Configure API key in Settings"
                                </div>
                            })
                        } else {
                            None
                        }
                    }}

                    // Processing indicator
                    {move || {
                        let state = processing_state.get();
                        if state.is_processing() {
                            let display_text = state.display_text().to_string();
                            Some(view! {
                                <div class="chat-progress">
                                    <div class="chat-progress-text">
                                        {display_text}
                                    </div>
                                    <div class="chat-progress-iteration">
                                        {format!("Iteration {}/{}", current_iteration.get(), max_iterations.get())}
                                    </div>
                                </div>
                            })
                        } else {
                            None
                        }
                    }}

                    // Chat history
                    <div class="chat-history">
                        {move || messages.get().iter().map(|msg| {
                            let class_name = match msg.message_type {
                                ChatMessageType::User => "chat-message chat-message-user",
                                ChatMessageType::Assistant => "chat-message chat-message-assistant",
                                ChatMessageType::System => "chat-message chat-message-system",
                                ChatMessageType::Error => "chat-message chat-message-error",
                            };
                            let content = msg.content.clone();
                            view! {
                                <div class=class_name>
                                    {content}
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>

                    // Input area
                    <div class="chat-input-area">
                        <textarea
                            class="chat-input"
                            placeholder=move || {
                                if !is_online.get() {
                                    "AI unavailable offline..."
                                } else if !has_api_key.get() {
                                    "Configure API key first..."
                                } else if processing_state.get().is_processing() {
                                    "Processing..."
                                } else {
                                    "Describe what you want to change..."
                                }
                            }
                            prop:value=move || input_text.get()
                            on:input=move |ev| {
                                input_text.set(event_target_value(&ev));
                            }
                            on:keydown=on_keydown
                            disabled=move || {
                                !is_online.get() || !has_api_key.get() || processing_state.get().is_processing()
                            }
                        />
                        <button
                            class="chat-send-btn"
                            on:click=handle_send
                            disabled=move || {
                                let text = input_text.get();
                                text.trim().is_empty()
                                    || !is_online.get()
                                    || !has_api_key.get()
                                    || processing_state.get().is_processing()
                            }
                        >
                            "Send"
                        </button>
                    </div>
                </div>
            })}
        </div>

        <style>
            r#"
            .chat-panel {
                width: 300px;
                min-width: 300px;
                background: var(--bg-secondary);
                border-left: 1px solid var(--border);
                display: flex;
                flex-direction: column;
                transition: width 0.2s, min-width 0.2s;
            }

            .chat-panel.collapsed {
                width: 40px;
                min-width: 40px;
            }

            .chat-panel-header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: 0.75rem;
                background: var(--bg-tertiary);
                border-bottom: 1px solid var(--border);
                cursor: pointer;
            }

            .chat-panel.collapsed .chat-panel-header {
                writing-mode: vertical-rl;
                text-orientation: mixed;
                padding: 0.75rem 0.5rem;
            }

            .chat-panel-title {
                font-size: 0.875rem;
                font-weight: 600;
            }

            .chat-panel-toggle {
                background: none;
                border: none;
                color: var(--text-secondary);
                cursor: pointer;
                padding: 0;
                font-size: 0.75rem;
            }

            .chat-panel.collapsed .chat-panel-toggle {
                margin-top: 0.5rem;
            }

            .chat-panel-body {
                flex: 1;
                display: flex;
                flex-direction: column;
                overflow: hidden;
            }

            .chat-status-banner {
                padding: 0.5rem;
                text-align: center;
                font-size: 0.75rem;
                font-weight: 500;
            }

            .chat-status-offline {
                background: var(--error);
                color: white;
            }

            .chat-status-warning {
                background: var(--warning);
                color: #1a1a2e;
            }

            .chat-progress {
                padding: 0.75rem;
                background: var(--bg-tertiary);
                border-bottom: 1px solid var(--border);
            }

            .chat-progress-text {
                font-size: 0.875rem;
                color: var(--accent);
                margin-bottom: 0.25rem;
            }

            .chat-progress-iteration {
                font-size: 0.75rem;
                color: var(--text-secondary);
            }

            .chat-history {
                flex: 1;
                overflow-y: auto;
                padding: 0.75rem;
                display: flex;
                flex-direction: column;
                gap: 0.5rem;
            }

            .chat-message {
                padding: 0.5rem 0.75rem;
                border-radius: 8px;
                font-size: 0.8rem;
                line-height: 1.4;
                max-width: 90%;
                word-wrap: break-word;
            }

            .chat-message-user {
                background: var(--accent);
                color: white;
                align-self: flex-end;
            }

            .chat-message-assistant {
                background: var(--bg-tertiary);
                color: var(--text-primary);
                align-self: flex-start;
            }

            .chat-message-system {
                background: transparent;
                color: var(--text-secondary);
                font-size: 0.75rem;
                font-style: italic;
                align-self: center;
                text-align: center;
            }

            .chat-message-error {
                background: var(--error);
                color: white;
                align-self: flex-start;
            }

            .chat-input-area {
                display: flex;
                gap: 0.5rem;
                padding: 0.75rem;
                border-top: 1px solid var(--border);
            }

            .chat-input {
                flex: 1;
                padding: 0.5rem;
                background: var(--bg-primary);
                border: 1px solid var(--border);
                border-radius: 4px;
                color: var(--text-primary);
                font-size: 0.8rem;
                font-family: inherit;
                resize: none;
                min-height: 60px;
            }

            .chat-input:focus {
                outline: none;
                border-color: var(--accent);
            }

            .chat-input:disabled {
                opacity: 0.5;
                cursor: not-allowed;
            }

            .chat-send-btn {
                padding: 0.5rem 1rem;
                background: var(--accent);
                color: white;
                border: none;
                border-radius: 4px;
                cursor: pointer;
                font-size: 0.8rem;
                font-weight: 500;
                align-self: flex-end;
            }

            .chat-send-btn:hover:not(:disabled) {
                background: var(--accent-hover);
            }

            .chat-send-btn:disabled {
                opacity: 0.5;
                cursor: not-allowed;
            }
            "#
        </style>
    }
}
