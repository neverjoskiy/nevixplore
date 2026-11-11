//! Thin `#[tauri::command]` adapters.
//!
//! Rules: commands take and return serde DTOs, do no I/O of their own beyond
//! delegating, and never expose a type with a lifetime. Real work lands in
//! `crates/*` from PR-1 onward.

use serde::Serialize;
use tauri::{State, WebviewWindow};

use crate::AppState;
use crate::error::AppError;
use crate::prefs::{Density, Prefs, Theme};

/// Everything the frontend needs to draw the first frame.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub version: String,
    pub settings: SettingsDto,
    pub start_dir: String,
}

/// The subset of [`Prefs`] the UI mirrors, with names it can use directly.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    pub theme: Theme,
    pub accent: Option<String>,
    pub density: Density,
    pub sidebar_width: u16,
    pub show_sidebar: bool,
    pub show_preview: bool,
    pub preview_enabled: bool,
}

impl From<&Prefs> for SettingsDto {
    fn from(prefs: &Prefs) -> Self {
        Self {
            theme: prefs.theme,
            accent: prefs.accent.clone(),
            density: prefs.density,
            sidebar_width: prefs.sidebar_width,
            show_sidebar: prefs.show_sidebar,
            show_preview: prefs.show_preview,
            preview_enabled: prefs.preview_enabled,
        }
    }
}

/// Called once by the shell on mount.
#[tauri::command]
pub fn bootstrap(state: State<'_, AppState>) -> Bootstrap {
    let prefs = state.prefs.lock().unwrap_or_else(|e| e.into_inner());
    Bootstrap {
        version: env!("CARGO_PKG_VERSION").to_string(),
        settings: SettingsDto::from(&*prefs),
        start_dir: state.paths.start_dir().display().to_string(),
    }
}

/// Persists the theme and applies it to the window immediately.
#[tauri::command]
pub fn set_theme(
    window: WebviewWindow,
    state: State<'_, AppState>,
    theme: Theme,
) -> Result<SettingsDto, AppError> {
    let updated = {
        let mut prefs = state.prefs.lock().unwrap_or_else(|e| e.into_inner());
        prefs.theme = theme;
        prefs
            .save(&state.paths)
            .map_err(|err| AppError::io(state.paths.settings_file(), err))?;
        SettingsDto::from(&*prefs)
    };

    apply_window_theme(&window, theme)?;
    Ok(updated)
}

/// Opens the native folder picker and returns the chosen path.
#[tauri::command]
pub async fn pick_folder(
    window: WebviewWindow,
    start_dir: String,
) -> Result<Option<String>, AppError> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = tokio::sync::oneshot::channel();
    let mut builder = window.dialog().file().set_title("Open folder");
    if !start_dir.is_empty() {
        builder = builder.set_directory(start_dir);
    }
    builder.pick_folder(move |picked| {
        let _ = tx.send(picked);
    });

    let picked = rx
        .await
        .map_err(|_| AppError::Internal("folder picker was closed".into()))?;
    Ok(picked.map(|path| path.to_string()))
}

/// Maps the app theme onto the window's theme, which drives the title bar
/// colour on Windows 11 and is a no-op on Windows 10.
fn apply_window_theme(window: &WebviewWindow, theme: Theme) -> Result<(), AppError> {
    let native = match theme {
        Theme::System => None,
        Theme::Light => Some(tauri::Theme::Light),
        Theme::Dark => Some(tauri::Theme::Dark),
    };
    window.set_theme(native)?;
    Ok(())
}
