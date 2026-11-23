#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod commands;
mod error;
mod mini;
mod notify;
mod overlay;
mod plan;
mod platform;
mod prefs;
mod shield;
mod shortcuts;
mod state;
mod stats;
mod timer;
mod tracking;
mod trading;
mod tray;
mod ui;
mod windows;

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
            commands::app::open_main_window,
            commands::app::quit_app,
            commands::app::hide_popover,
            commands::app::set_mini_visible,
            commands::app::mini_visible,
            commands::app::get_prefs,
            commands::app::set_prefs,
            commands::app::toggle_palette,
            commands::app::hide_palette,
            commands::app::request_ui,
            commands::app::dismiss_check_in,
            commands::app::close_break_overlay,
            commands::timer::check_in,
            commands::timer::timer_start_task,
            commands::timer::timer_set_note,
            commands::plan::plan_view,
            commands::plan::quick_add_preview,
            commands::plan::quick_add,
            commands::plan::goal_create,
            commands::plan::goal_update,
            commands::plan::goal_set_done,
            commands::plan::goal_delete,
            commands::plan::milestone_create,
            commands::plan::milestone_set_done,
            commands::plan::milestone_delete,
            commands::plan::project_create,
            commands::plan::project_update,
            commands::plan::project_delete,
            commands::plan::task_create,
            commands::plan::task_update,
            commands::plan::task_complete,
            commands::plan::task_reopen,
            commands::plan::task_delete,
            commands::plan::block_create,
            commands::plan::block_move,
            commands::plan::block_delete,
            commands::timer::timer_state,
            commands::timer::timer_start,
            commands::timer::timer_select_preset,
            commands::timer::timer_start_pending,
            commands::timer::timer_pause,
            commands::timer::timer_resume,
            commands::timer::timer_extend,
            commands::timer::timer_end,
            commands::stats::today_focus,
            commands::stats::stats_view,
            commands::stats::export_sessions,
            commands::stats::reflection_state,
            commands::stats::reflect,
            commands::timer::timer_settings,
            commands::timer::timer_set_settings,
            commands::timer::log_session,
            commands::trading::trading_view,
            commands::trading::trading_settings,
            commands::trading::trading_set_settings,
            commands::trading::checklist_set,
            commands::trading::checklist_add,
            commands::trading::checklist_remove,
            commands::trading::checklist_reset,
            commands::trading::checklist_restore_suggested,
            commands::trading::head_check,
            commands::trading::head_check_clear,
            commands::trading::open_automation_settings,
        ])
        .events(collect_events![
            timer::TimerTick,
            timer::TimerPhaseEvent,
            ui::UiRequest,
            plan::PlanChanged,
            trading::TradingChanged
        ])
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
            timer::close_abandoned(&state)?;
            let saved_prefs = prefs::load(&state);
            let initial = timer::engine_snapshot(&state)?;
            let desk = trading::Desk::load(&state);
            app.manage(state);
            app.manage(desk);
            app.manage(std::sync::Mutex::new(saved_prefs));
            app.manage(std::sync::Mutex::new(tray::TrayCache::default()));

            let handle = app.handle();
            windows::create_popover(handle)?;
            windows::create_palette(handle)?;
            windows::create_check_in(handle)?;
            windows::create_reflection(handle)?;
            mini::create(handle)?;
            tray::create(handle, &initial)?;
            shortcuts::register(handle);
            timer::spawn_driver(app.handle().clone());
            trading::tracker::spawn(app.handle().clone());

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == mini::MINI
                && let tauri::WindowEvent::Moved(position) = event
            {
                mini::moved(window.app_handle(), *position);
            }
            if window.label() == windows::PALETTE
                && let tauri::WindowEvent::Focused(false) = event
                && let Err(e) = windows::hide_palette(window.app_handle())
            {
                eprintln!("hide palette: {e}");
            }
            // The popover closes when it loses focus, like a native menu.
            if window.label() == windows::POPOVER
                && let tauri::WindowEvent::Focused(false) = event
                && let Err(e) = windows::hide_popover(window.app_handle())
            {
                eprintln!("hide popover: {e}");
            }
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
        if let tauri::RunEvent::ExitRequested { .. } = _event
            && let Err(e) = mini::save(_handle)
        {
            eprintln!("save mini timer: {e}");
        }
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen {
            has_visible_windows: false,
            ..
        } = _event
        {
            windows::show_main(_handle);
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
