//! The floating mini timer: a small always-on-top window. Whether it's shown
//! and where it was dragged are saved under the `mini` settings key.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

use crate::error::Result;
use crate::state::AppState;
use crate::timer::now_ms;

pub const MINI: &str = "mini";

/// Pill size in logical pixels: the window is exactly the pill. No shadow,
/// native or CSS, so nothing trails it when it's dragged.
const WIDTH: f64 = 380.0;
const HEIGHT: f64 = 52.0;
/// Distance from the screen's top-right corner for the first placement.
const MARGIN: f64 = 24.0;

const SETTINGS_KEY: &str = "mini";

/// What's saved: shown or not, and the last position (physical pixels).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiniPlacement {
    pub visible: bool,
    pub position: Option<(i32, i32)>,
}

/// The live placement, saved when the window hides and when Halo quits.
pub type MiniState = Mutex<MiniPlacement>;

/// Creates the mini timer, restoring where it was and whether it was open.
pub fn create(app: &AppHandle) -> Result<()> {
    let saved = load(&app.state::<AppState>());
    let window = WebviewWindowBuilder::new(app, MINI, WebviewUrl::App("index.html".into()))
        .title("Halo mini timer")
        .inner_size(WIDTH, HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible_on_all_workspaces(true)
        .accept_first_mouse(true)
        .focused(false)
        .visible(false)
        .build()?;

    let on_screen = match saved.position {
        Some((x, y)) => app
            .monitor_from_point(f64::from(x), f64::from(y))?
            .is_some(),
        None => false,
    };
    let position = match saved.position {
        Some((x, y)) if on_screen => Some(PhysicalPosition::new(x, y)),
        _ => default_position(app)?,
    };
    if let Some(p) = position {
        window.set_position(p)?;
    }
    app.manage(Mutex::new(MiniPlacement {
        visible: saved.visible,
        position: position.map(|p| (p.x, p.y)),
    }));
    if saved.visible {
        window.show()?;
    }
    Ok(())
}

pub fn is_visible(app: &AppHandle) -> bool {
    app.get_webview_window(MINI)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false)
}

pub fn set_visible(app: &AppHandle, visible: bool) -> Result<()> {
    let Some(window) = app.get_webview_window(MINI) else {
        return Ok(());
    };
    if visible {
        window.show()?;
    } else {
        window.hide()?;
    }
    if let Ok(mut placement) = app.state::<MiniState>().lock() {
        placement.visible = visible;
    }
    save(app)
}

pub fn toggle(app: &AppHandle) -> Result<()> {
    set_visible(app, !is_visible(app))
}

/// Remembers where the window was dragged. Saved on hide and on quit.
pub fn moved(app: &AppHandle, position: PhysicalPosition<i32>) {
    if let Ok(mut placement) = app.state::<MiniState>().lock() {
        placement.position = Some((position.x, position.y));
    }
}

pub fn save(app: &AppHandle) -> Result<()> {
    let placement = match app.state::<MiniState>().lock() {
        Ok(p) => *p,
        Err(_) => return Ok(()),
    };
    let json = serde_json::to_string(&placement)?;
    let state = app.state::<AppState>();
    let conn = state.db()?;
    halo_store::settings::set(
        &conn,
        SETTINGS_KEY,
        &json,
        i64::try_from(now_ms()).unwrap_or(i64::MAX),
    )?;
    Ok(())
}

fn load(state: &AppState) -> MiniPlacement {
    let saved = state.db().ok().and_then(|conn| {
        halo_store::settings::get(&conn, SETTINGS_KEY)
            .ok()
            .flatten()
    });
    saved
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

/// Top-right of the main screen's work area.
fn default_position(app: &AppHandle) -> Result<Option<PhysicalPosition<i32>>> {
    let Some(monitor) = app.primary_monitor()? else {
        return Ok(None);
    };
    let scale = monitor.scale_factor();
    let area = monitor.work_area();
    let x = f64::from(area.position.x) + f64::from(area.size.width) - (WIDTH + MARGIN) * scale;
    let y = f64::from(area.position.y) + MARGIN * scale;
    Ok(Some(PhysicalPosition::new(
        x.round() as i32,
        y.round() as i32,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_round_trips_and_defaults_when_missing() {
        let p = MiniPlacement {
            visible: true,
            position: Some((1200, 40)),
        };
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<MiniPlacement>(&json).unwrap(), p);
        assert_eq!(
            MiniPlacement::default(),
            MiniPlacement {
                visible: false,
                position: None
            }
        );
    }
}
