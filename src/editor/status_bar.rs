//! Status bar component
//!
//! This component displays status information at the bottom of the editor:
//! - Online/Offline status
//! - Project name
//! - Last saved timestamp
//! - Current status message

use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Online/Offline status
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ConnectionStatus {
    #[default]
    Online,
    Offline,
}

impl ConnectionStatus {
    /// Check if online
    pub fn is_online(&self) -> bool {
        matches!(self, Self::Online)
    }
}

/// Status bar component
#[component]
pub fn StatusBar(
    /// Current connection status
    connection_status: Signal<ConnectionStatus>,
    /// Project name
    project_name: Signal<String>,
    /// Last saved timestamp (optional)
    last_saved: Signal<Option<String>>,
    /// Current status message (optional)
    status_message: Signal<Option<String>>,
) -> impl IntoView {
    view! {
        <footer class="status-bar-container">
            // Status message (if any)
            {move || status_message.get().map(|msg| view! {
                <span class="status-message">{msg}</span>
                <span class="status-separator">"|"</span>
            })}

            // Project name
            <span class="status-project">
                {move || project_name.get()}
            </span>

            <span class="status-separator">"|"</span>

            // Last saved
            <span class="status-saved">
                {move || {
                    match last_saved.get() {
                        Some(time) => format!("Saved: {}", time),
                        None => "Not saved".to_string(),
                    }
                }}
            </span>

            // Spacer
            <span class="status-spacer" />

            // Connection status indicator
            <span class="status-connection" class:online=move || connection_status.get().is_online()>
                {move || if connection_status.get().is_online() {
                    "\u{1F7E2} Online"
                } else {
                    "\u{1F534} Offline"
                }}
            </span>
        </footer>

        <style>
            r#"
            .status-bar-container {
                display: flex;
                align-items: center;
                padding: 0.5rem 1rem;
                background: var(--bg-tertiary);
                border-top: 1px solid var(--border);
                font-size: 0.75rem;
                color: var(--text-secondary);
            }

            .status-message {
                color: var(--accent);
            }

            .status-separator {
                margin: 0 0.5rem;
                opacity: 0.5;
            }

            .status-project {
                font-weight: 500;
            }

            .status-saved {
                font-style: italic;
            }

            .status-spacer {
                flex: 1;
            }

            .status-connection {
                font-weight: 500;
            }

            .status-connection.online {
                color: var(--success);
            }

            .status-connection:not(.online) {
                color: var(--error);
            }
            "#
        </style>
    }
}

/// Hook to detect online/offline status changes
pub fn use_online_status() -> RwSignal<ConnectionStatus> {
    let status = create_rw_signal(ConnectionStatus::Online);

    let Some(window) = web_sys::window() else {
        return status;
    };

    // Check initial status via navigator.onLine
    let is_online = js_sys::Reflect::get(&window, &"navigator".into())
        .ok()
        .and_then(|nav| js_sys::Reflect::get(&nav, &"onLine".into()).ok())
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    if !is_online {
        status.set(ConnectionStatus::Offline);
    }

    // Set up event listeners for online/offline events
    let online_closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
        status.set(ConnectionStatus::Online);
    }) as Box<dyn Fn(_)>);

    let _ =
        window.add_event_listener_with_callback("online", online_closure.as_ref().unchecked_ref());
    online_closure.forget();

    let offline_closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
        status.set(ConnectionStatus::Offline);
    }) as Box<dyn Fn(_)>);

    let _ = window
        .add_event_listener_with_callback("offline", offline_closure.as_ref().unchecked_ref());
    offline_closure.forget();

    status
}
