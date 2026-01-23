#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
use slick_sheet_studio::editor;

#[cfg(target_arch = "wasm32")]
fn main() {
    // Initialize tracing for WASM
    tracing_wasm::set_as_global_default();

    // Mount the app to the #app element, replacing the loading state
    let document = web_sys::window()
        .expect("window")
        .document()
        .expect("document");
    let app_element = document
        .get_element_by_id("app")
        .expect("app element")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("app should be HtmlElement");

    // Clear the loading content
    app_element.set_inner_html("");

    leptos::mount_to(app_element, editor::Editor);
}

/// Stub main for non-WASM builds (used during testing)
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("This binary is intended for WASM. Use slick-cli for native builds.");
}
