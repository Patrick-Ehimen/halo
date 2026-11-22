#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod commands;
mod error;
mod platform;
mod shield;
mod state;
mod timer;
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
        .commands(collect_commands![
            commands::app::app_info,
            commands::timer::timer_state,
            commands::timer::timer_start,
            commands::timer::timer_select_preset,
            commands::timer::timer_start_pending,
            commands::timer::timer_pause,
            commands::timer::timer_resume,
            commands::timer::timer_extend,
            commands::timer::timer_end,
            commands::timer::today_focus,
            commands::timer::timer_settings,
            commands::timer::timer_set_settings,
            commands::timer::log_session,
        ])
        .events(collect_events![timer::TimerTick, timer::TimerPhaseEvent])
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
            let state = AppState::new(conn);
            timer::load_settings(&state)?;
            app.manage(state);
            timer::spawn_driver(app.handle().clone());

            Ok(())
        })
        .on_window_event(|window, event| {
            // On macOS, closing the main window hides it; the timer keeps
            // running and the Dock icon brings it back. ⌘Q quits.
            #[cfg(target_os = "macos")]
            if let tauri::WindowEvent::CloseRequested { api, .. } = event
                && window.label() == "main"
            {
                api.prevent_close();
                if let Err(e) = window.hide() {
                    eprintln!("hide main window: {e}");
                }
            }
            #[cfg(not(target_os = "macos"))]
            let _ = (window, event);
        })
        .build(tauri::generate_context!());

    let app = match result {
        Ok(app) => app,
        Err(e) => {
            eprintln!("halo failed to start: {e}");
            std::process::exit(1);
        }
    };

    app.run(|_handle, _event| {
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen {
            has_visible_windows: false,
            ..
        } = _event
            && let Some(window) = _handle.get_webview_window("main")
            && let Err(e) = window.show().and_then(|()| window.set_focus())
        {
            eprintln!("show main window: {e}");
        }
    });
}

#[cfg(test)]
mod tests {
    /// `cargo test` keeps `src/bindings.ts` in sync without launching the app.
    #[test]
    fn export_bindings() {
        super::export_bindings(&super::specta_builder()).unwrap();
    }
}
