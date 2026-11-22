//! The break overlay: one window per display, covering it during a break.

use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};

use crate::error::Result;

const PREFIX: &str = "break-overlay-";

pub fn is_overlay(label: &str) -> bool {
    label.starts_with(PREFIX)
}

/// Covers every connected display. Does nothing if overlays are already up.
pub fn open(app: &AppHandle) -> Result<()> {
    if !crate::prefs::get(app).break_overlay || any_open(app) {
        return Ok(());
    }
    for (i, monitor) in app.available_monitors()?.into_iter().enumerate() {
        let window = WebviewWindowBuilder::new(
            app,
            format!("{PREFIX}{i}"),
            WebviewUrl::App("index.html".into()),
        )
        .title("Halo break")
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible_on_all_workspaces(true)
        .visible(false)
        .build()?;
        let PhysicalPosition { x, y } = *monitor.position();
        let PhysicalSize { width, height } = *monitor.size();
        window.set_position(PhysicalPosition::new(x, y))?;
        window.set_size(PhysicalSize::new(width, height))?;
        window.show()?;
        if i == 0 {
            window.set_focus()?;
        }
    }
    Ok(())
}

/// Removes every overlay window.
pub fn close(app: &AppHandle) -> Result<()> {
    for (label, window) in app.webview_windows() {
        if is_overlay(&label) {
            window.destroy()?;
        }
    }
    Ok(())
}

fn any_open(app: &AppHandle) -> bool {
    app.webview_windows().keys().any(|l| is_overlay(l))
}
