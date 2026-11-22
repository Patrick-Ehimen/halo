//! Halo's extra windows. Each loads the same UI bundle; the frontend picks
//! what to render from the window label.

use std::sync::atomic::{AtomicU64, Ordering};

use tauri::{
    AppHandle, Manager, PhysicalPosition, Rect, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

use crate::error::Result;
use crate::timer::now_ms;

pub const POPOVER: &str = "tray-popover";

/// Popover card width in logical pixels.
const POPOVER_WIDTH: f64 = 340.0;
const POPOVER_HEIGHT: f64 = 440.0;
/// Transparent margin around the card for its CSS shadow (logical pixels).
/// macOS draws a transparent window's native shadow as a square, so the
/// native shadow is off and the card casts its own. Keep in sync with
/// `PopoverWindow.tsx`.
const POPOVER_PAD_X: f64 = 14.0;
const POPOVER_PAD_TOP: f64 = 4.0;
/// Gap between the menu bar and the popover, logical pixels.
const POPOVER_GAP: f64 = 6.0;

/// When the popover last hid because it lost focus. Clicking the menu bar
/// icon to close an open popover blurs it first; without this the same click
/// would open it again.
static POPOVER_HIDDEN_AT: AtomicU64 = AtomicU64::new(0);
const REOPEN_GUARD_MS: u64 = 300;

/// Creates the popover hidden, so the first click opens it instantly.
pub fn create_popover(app: &AppHandle) -> Result<()> {
    WebviewWindowBuilder::new(app, POPOVER, WebviewUrl::App("index.html".into()))
        .title("Halo")
        .inner_size(POPOVER_WIDTH + 2.0 * POPOVER_PAD_X, POPOVER_HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible_on_all_workspaces(true)
        .accept_first_mouse(true)
        .visible(false)
        .build()?;
    Ok(())
}

/// Opens the popover under the menu bar icon, or closes it if it's open.
pub fn toggle_popover(app: &AppHandle, icon: Rect) -> Result<()> {
    let Some(popover) = app.get_webview_window(POPOVER) else {
        return Ok(());
    };
    if popover.is_visible()? {
        return hide_popover(app);
    }
    let since_hidden = now_ms().saturating_sub(POPOVER_HIDDEN_AT.load(Ordering::Relaxed));
    if since_hidden < REOPEN_GUARD_MS {
        return Ok(());
    }
    place_under(app, &popover, icon)?;
    popover.show()?;
    popover.set_focus()?;
    Ok(())
}

pub fn hide_popover(app: &AppHandle) -> Result<()> {
    if let Some(popover) = app.get_webview_window(POPOVER)
        && popover.is_visible()?
    {
        popover.hide()?;
        POPOVER_HIDDEN_AT.store(now_ms(), Ordering::Relaxed);
    }
    Ok(())
}

/// Centres the popover under the icon, kept inside the screen's work area.
fn place_under(app: &AppHandle, popover: &WebviewWindow, icon: Rect) -> Result<()> {
    let guess = popover.scale_factor()?;
    let icon_pos = icon.position.to_physical::<f64>(guess);
    let icon_size = icon.size.to_physical::<f64>(guess);
    let monitor = app.monitor_from_point(icon_pos.x, icon_pos.y)?;
    let scale = monitor.as_ref().map_or(guess, |m| m.scale_factor());

    let width = (POPOVER_WIDTH + 2.0 * POPOVER_PAD_X) * scale;
    let mut x = icon_pos.x + icon_size.width / 2.0 - width / 2.0;
    let y = icon_pos.y + icon_size.height + (POPOVER_GAP - POPOVER_PAD_TOP) * scale;
    if let Some(m) = monitor {
        let area = m.work_area();
        let left = f64::from(area.position.x) + 8.0 * scale;
        let right = f64::from(area.position.x) + f64::from(area.size.width) - width - 8.0 * scale;
        x = x.clamp(left, right.max(left));
    }
    popover.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32))?;
    Ok(())
}

/// Shows and focuses the main window.
pub fn show_main(app: &AppHandle) {
    if let Err(e) = hide_popover(app) {
        eprintln!("hide popover: {e}");
    }
    if let Some(window) = app.get_webview_window("main")
        && let Err(e) = window
            .show()
            .and_then(|()| window.unminimize())
            .and_then(|()| window.set_focus())
    {
        eprintln!("show main window: {e}");
    }
}

pub const PALETTE: &str = "palette";
const PALETTE_WIDTH: f64 = 600.0;
const PALETTE_HEIGHT: f64 = 420.0;

/// Creates the ⌘K palette hidden, so it opens instantly.
pub fn create_palette(app: &AppHandle) -> Result<()> {
    WebviewWindowBuilder::new(app, PALETTE, WebviewUrl::App("index.html".into()))
        .title("Halo command palette")
        .inner_size(PALETTE_WIDTH, PALETTE_HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible_on_all_workspaces(true)
        .visible(false)
        .build()?;
    Ok(())
}

/// Opens the palette on the screen with the pointer, or closes it.
pub fn toggle_palette(app: &AppHandle) -> Result<()> {
    let Some(palette) = app.get_webview_window(PALETTE) else {
        return Ok(());
    };
    if palette.is_visible()? {
        palette.hide()?;
        return Ok(());
    }
    let cursor = app.cursor_position()?;
    if let Some(m) = app.monitor_from_point(cursor.x, cursor.y)? {
        let scale = m.scale_factor();
        let area = m.work_area();
        let x =
            f64::from(area.position.x) + (f64::from(area.size.width) - PALETTE_WIDTH * scale) / 2.0;
        let y = f64::from(area.position.y) + f64::from(area.size.height) * 0.18;
        palette.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32))?;
    }
    palette.show()?;
    palette.set_focus()?;
    Ok(())
}

pub fn hide_palette(app: &AppHandle) -> Result<()> {
    if let Some(palette) = app.get_webview_window(PALETTE)
        && palette.is_visible()?
    {
        palette.hide()?;
    }
    Ok(())
}

pub const CHECK_IN: &str = "check-in";
const CHECK_IN_WIDTH: f64 = 380.0;
const CHECK_IN_HEIGHT: f64 = 140.0;
const CHECK_IN_MARGIN: f64 = 16.0;

/// Creates the "Still on it?" card hidden.
pub fn create_check_in(app: &AppHandle) -> Result<()> {
    WebviewWindowBuilder::new(app, CHECK_IN, WebviewUrl::App("index.html".into()))
        .title("Halo check-in")
        .inner_size(CHECK_IN_WIDTH, CHECK_IN_HEIGHT)
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
    Ok(())
}

/// Shows the check-in card at the top right of the main screen, where
/// notifications appear.
pub fn show_check_in(app: &AppHandle) -> Result<()> {
    let Some(card) = app.get_webview_window(CHECK_IN) else {
        return Ok(());
    };
    if let Some(m) = app.primary_monitor()? {
        let scale = m.scale_factor();
        let area = m.work_area();
        let x = f64::from(area.position.x) + f64::from(area.size.width)
            - (CHECK_IN_WIDTH + CHECK_IN_MARGIN) * scale;
        let y = f64::from(area.position.y) + CHECK_IN_MARGIN * scale;
        card.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32))?;
    }
    card.show()?;
    Ok(())
}

pub fn hide_check_in(app: &AppHandle) -> Result<()> {
    if let Some(card) = app.get_webview_window(CHECK_IN)
        && card.is_visible()?
    {
        card.hide()?;
    }
    Ok(())
}
