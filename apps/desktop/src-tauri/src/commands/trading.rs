//! The trading desk. Halo never places trades, holds keys or gives advice.

use tauri::AppHandle;

use crate::error::IpcError;
use crate::trading::{self, TradingSettings, TradingView};

/// Clocks, checklist, head check, gate and chart time, as of now.
#[tauri::command]
#[specta::specta]
pub fn trading_view(app: AppHandle) -> Result<TradingView, IpcError> {
    Ok(trading::view(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn trading_settings(app: AppHandle) -> Result<TradingSettings, IpcError> {
    Ok(trading::settings(&app))
}

#[tauri::command]
#[specta::specta]
pub fn trading_set_settings(
    app: AppHandle,
    settings: TradingSettings,
) -> Result<TradingSettings, IpcError> {
    Ok(trading::set_settings(&app, settings)?)
}

/// Ticks or unticks a checklist item for today.
#[tauri::command]
#[specta::specta]
pub fn checklist_set(app: AppHandle, id: String, checked: bool) -> Result<(), IpcError> {
    Ok(trading::set_checked(&app, &id, checked)?)
}

#[tauri::command]
#[specta::specta]
pub fn checklist_add(app: AppHandle, label: String, hint: String) -> Result<(), IpcError> {
    Ok(trading::add_item(&app, &label, &hint)?)
}

#[tauri::command]
#[specta::specta]
pub fn checklist_remove(app: AppHandle, id: String) -> Result<(), IpcError> {
    Ok(trading::remove_item(&app, &id)?)
}

/// Adds the suggested rules back (any not already there).
#[tauri::command]
#[specta::specta]
pub fn checklist_restore_suggested(app: AppHandle) -> Result<(), IpcError> {
    Ok(trading::restore_suggested(&app)?)
}

/// Unticks today's checklist.
#[tauri::command]
#[specta::specta]
pub fn checklist_reset(app: AppHandle) -> Result<(), IpcError> {
    Ok(trading::reset_checklist(&app)?)
}

/// "How clear is your head", 1–5, with what's going on.
#[tauri::command]
#[specta::specta]
pub fn head_check(app: AppHandle, score: u8, factors: Vec<String>) -> Result<(), IpcError> {
    Ok(trading::head_check(&app, score, factors)?)
}

/// Takes back today's head check (not a 1–2, which locks the day).
#[tauri::command]
#[specta::specta]
pub fn head_check_clear(app: AppHandle) -> Result<(), IpcError> {
    Ok(trading::clear_head_check(&app)?)
}

/// Opens System Settings at Privacy & Security → Automation.
#[tauri::command]
#[specta::specta]
pub fn open_automation_settings() -> Result<(), IpcError> {
    #[cfg(target_os = "macos")]
    crate::platform::macos::browser::open_automation_settings()
        .map_err(crate::error::Error::from)?;
    Ok(())
}
