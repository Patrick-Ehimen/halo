use serde::Serialize;
use specta::Type;
use tauri::State;

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
