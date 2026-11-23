//! Stats, today at a glance, export, and the post-session reflection.

use serde::Deserialize;
use specta::Type;
use tauri::AppHandle;

use crate::error::IpcError;
use crate::stats::{self, ReflectionView, StatsRange, StatsView, TodayFocus};

/// Focus today: the goal ring, the tree, the streak.
#[tauri::command]
#[specta::specta]
pub fn today_focus(app: AppHandle) -> Result<TodayFocus, IpcError> {
    Ok(stats::today(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn stats_view(app: AppHandle, range: StatsRange) -> Result<StatsView, IpcError> {
    Ok(stats::view(&app, range)?)
}

#[derive(Debug, Clone, Copy, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ExportFormat {
    Csv,
    Json,
}

/// Writes every session to Downloads. Returns the file's path.
#[tauri::command]
#[specta::specta]
pub fn export_sessions(app: AppHandle, format: ExportFormat) -> Result<String, IpcError> {
    Ok(stats::export(&app, matches!(format, ExportFormat::Json))?)
}

/// What the reflection card shows; `null` when there's nothing to reflect on.
#[tauri::command]
#[specta::specta]
pub fn reflection_state(app: AppHandle) -> Result<Option<ReflectionView>, IpcError> {
    Ok(stats::reflection(&app)?)
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ReflectionInput {
    /// 1 Rough, 2 Okay, 3 Good, 4 In flow; `null` to skip.
    pub rating: Option<u8>,
    pub distractions: u32,
    pub notes: String,
    /// Start the break that's waiting.
    pub start_break: bool,
}

/// Saves the reflection and closes the card.
#[tauri::command]
#[specta::specta]
pub fn reflect(app: AppHandle, input: ReflectionInput) -> Result<(), IpcError> {
    Ok(stats::reflect(
        &app,
        input.rating,
        input.distractions,
        &input.notes,
        input.start_break,
    )?)
}
