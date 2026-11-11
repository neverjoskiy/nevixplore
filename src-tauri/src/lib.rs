//! NeviXplore - a fast, beautiful Windows file manager.
//!
//! Layer rule (see `docs/ARCHITECTURE.md`): UI -> commands -> services -> OS.
//! This crate holds the Tauri shell and thin command adapters only. Anything
//! testable lives in `crates/*`, which carry no Tauri dependency.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;
mod prefs;

use std::sync::Mutex;

use tauri::{Manager, RunEvent};

/// Application state shared by every command.
///
/// Everything here is loaded once at startup; commands only read it. Settings
/// writes go through [`prefs::Prefs::save`], which is the single writer.
pub struct AppState {
    pub prefs: Mutex<prefs::Prefs>,
    pub paths: prefs::Paths,
}

pub fn run() {
    init_tracing();

    let paths = prefs::Paths::resolve();
    let prefs = prefs::Prefs::load(&paths);
    // Persist immediately so the very first run leaves a readable, editable
    // settings file rather than silently defaulting forever.
    if let Err(err) = prefs.save(&paths) {
        tracing::warn!(%err, "could not write initial settings");
    }
    let start_dir = paths.start_dir();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            prefs: Mutex::new(prefs),
            paths,
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::set_theme,
            commands::pick_folder,
        ])
        .setup(move |app| {
            let window = app
                .get_webview_window("main")
                .ok_or_else(|| error::AppError::Internal("main window missing".into()))?;
            window.set_title("NevIxplore")?;
            tracing::info!(dir = %start_dir.display(), "window ready");
            Ok(())
        })
        .build(tauri::generate_context!())
        .unwrap_or_else(|err| {
            tracing::error!(%err, "failed to build the app");
            std::process::exit(1);
        });

    app.run(|_handle, event| {
        if let RunEvent::ExitRequested { .. } = event {
            tracing::info!("shutting down");
        }
    });
}

/// Logs go to stderr; a file sink would need a rolling writer that we do not
/// need before PR-14. `RUST_LOG=debug` turns on the noisy path.
fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_target(false).init();
}
