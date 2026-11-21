#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod commands;
mod error;
mod platform;
mod shield;
mod state;
mod tracking;

use tauri::Manager;
use tauri_specta::{Builder, ErrorHandlingMode, collect_commands, collect_events};

use crate::state::AppState;

/// Generated TypeScript bindings live in `packages/core` (`@halo/core`).
const BINDINGS_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../packages/core/src/bindings.ts"
);

/// Single source of truth for every IPC command and event.
fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .error_handling(ErrorHandlingMode::Result)
        .commands(collect_commands![commands::app::app_info])
        .events(collect_events![])
}

fn export_bindings(builder: &Builder<tauri::Wry>) -> Result<(), String> {
    builder
        .export(
            specta_typescript::Typescript::default().header("// @ts-nocheck\n/* eslint-disable */"),
            BINDINGS_PATH,
        )
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    if let Err(e) = export_bindings(&builder) {
        eprintln!("failed to export TS bindings: {e}");
    }

    let result = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let conn = halo_store::open(&data_dir.join("halo.db"))?;
            app.manage(AppState::new(conn));

            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(e) = result {
        eprintln!("halo failed to start: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    /// `cargo test` keeps `src/bindings.ts` in sync without launching the app.
    #[test]
    fn export_bindings() {
        super::export_bindings(&super::specta_builder()).unwrap();
    }
}
