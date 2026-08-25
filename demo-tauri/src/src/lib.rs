//! Thin Tauri 2 shell: loads the Trunk-built rangular Leptos demo SPA. No invoke commands.

use tauri::Manager;

/// Start the desktop webview and block until the process exits.
///
/// # Panics
///
/// Panics if the Tauri runtime fails to start.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Close the window = exit the process (no orphan after Trunk dies).
            if let Some(win) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        app_handle.exit(0);
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running rangular-demo-tauri");
}
