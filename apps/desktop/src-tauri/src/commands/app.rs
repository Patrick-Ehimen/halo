use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

use crate::{error::IpcError, state::AppState};

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub schema_version: u32,
}

#[tauri::command]
#[specta::specta]
pub fn app_info(state: State<'_, AppState>) -> Result<AppInfo, IpcError> {
    let conn = state.db()?;
    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version: halo_store::schema_version(&conn).map_err(crate::error::Error::from)?,
    })
}

/// Shows the main window (and closes the popover).
#[tauri::command]
#[specta::specta]
pub fn open_main_window(app: AppHandle) {
    crate::windows::show_main(&app);
}

#[tauri::command]
#[specta::specta]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
#[specta::specta]
pub fn hide_popover(app: AppHandle) -> Result<(), IpcError> {
    Ok(crate::windows::hide_popover(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn set_mini_visible(app: AppHandle, visible: bool) -> Result<(), IpcError> {
    Ok(crate::mini::set_visible(&app, visible)?)
}

#[tauri::command]
#[specta::specta]
pub fn mini_visible(app: AppHandle) -> bool {
    crate::mini::is_visible(&app)
}

#[tauri::command]
#[specta::specta]
pub fn get_prefs(app: AppHandle) -> crate::prefs::Prefs {
    crate::prefs::get(&app)
}

#[tauri::command]
#[specta::specta]
pub fn set_prefs(
    app: AppHandle,
    prefs: crate::prefs::Prefs,
) -> Result<crate::prefs::Prefs, IpcError> {
    Ok(crate::prefs::set(&app, prefs)?)
}

#[tauri::command]
#[specta::specta]
pub fn toggle_palette(app: AppHandle) -> Result<(), IpcError> {
    Ok(crate::windows::toggle_palette(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn hide_palette(app: AppHandle) -> Result<(), IpcError> {
    Ok(crate::windows::hide_palette(&app)?)
}

/// Shows the main window and opens a panel there (settings, manual log).
#[tauri::command]
#[specta::specta]
pub fn request_ui(app: AppHandle, action: crate::ui::UiAction) -> Result<(), IpcError> {
    crate::windows::hide_palette(&app)?;
    Ok(crate::ui::request(&app, action)?)
}

/// Closes the check-in card without answering.
#[tauri::command]
#[specta::specta]
pub fn dismiss_check_in(app: AppHandle) -> Result<(), IpcError> {
    Ok(crate::windows::hide_check_in(&app)?)
}

/// Closes the break overlay; the break itself keeps running.
#[tauri::command]
#[specta::specta]
pub fn close_break_overlay(app: AppHandle) -> Result<(), IpcError> {
    Ok(crate::overlay::close(&app)?)
}
