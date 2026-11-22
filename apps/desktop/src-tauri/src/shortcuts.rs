//! Global shortcuts, active from any app: ⌥⌘K palette, ⌥⌘S start or pause,
//! ⌥⌘M mini timer. A shortcut another app already owns is skipped, not fatal.

use halo_timer::{Phase, Status};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::state::AppState;
use crate::timer::{self, PresetId};

type Action = fn(&AppHandle) -> crate::error::Result<()>;

pub fn register(app: &AppHandle) {
    let shortcuts: [(&str, Action); 3] = [
        ("Alt+Super+K", crate::windows::toggle_palette),
        ("Alt+Super+S", start_or_pause),
        ("Alt+Super+M", crate::mini::toggle),
    ];
    for (keys, action) in shortcuts {
        let result = app
            .global_shortcut()
            .on_shortcut(keys, move |app, _, event| {
                if event.state() == ShortcutState::Pressed
                    && let Err(e) = action(app)
                {
                    eprintln!("shortcut {keys}: {e}");
                }
            });
        if let Err(e) = result {
            eprintln!("shortcut {keys} not registered: {e}");
        }
    }
}

/// Pauses a running phase, resumes a paused one, or starts what's waiting.
fn start_or_pause(app: &AppHandle) -> crate::error::Result<()> {
    let s = timer::engine_snapshot(&app.state::<AppState>())?;
    match (s.status, s.phase) {
        (Status::Running, _) => timer::pause(app).map(drop),
        (Status::Paused, _) => timer::resume(app).map(drop),
        (Status::Idle, Phase::Focus) => {
            let preset = PresetId::from_kind(s.kind);
            timer::start(app, preset, String::new()).map(drop)
        }
        (Status::Idle, _) => timer::start_pending(app).map(drop),
    }
}
