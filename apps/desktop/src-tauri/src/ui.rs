//! Requests from Rust surfaces (menu bar, palette) for the main window's UI.

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;
use tauri_specta::Event;

use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum UiAction {
    OpenSettings,
    LogSession,
}

/// Asks the main window to open a panel. Emitted after the window is shown.
#[derive(Debug, Clone, Serialize, Type, tauri_specta::Event)]
#[tauri_specta(event_name = "ui://request")]
pub struct UiRequest {
    pub action: UiAction,
}

/// Shows the main window and asks it to open `action`.
pub fn request(app: &AppHandle, action: UiAction) -> Result<()> {
    crate::windows::show_main(app);
    UiRequest { action }.emit_to(app, "main")?;
    Ok(())
}
