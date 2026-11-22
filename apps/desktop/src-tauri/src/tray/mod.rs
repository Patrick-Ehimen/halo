//! The menu bar item: a ring icon, the countdown as its title, and a
//! right-click menu. What it shows comes from `halo-menubar` and the user's
//! prefs; this module only pushes changes to macOS, and only when something
//! visibly changed.

use std::sync::Mutex;

use halo_menubar::{ICON_SIZE, Icon, TrayView};
use halo_timer::{Snapshot, Status};
use tauri::image::Image;
use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::error::Result;
use crate::timer::{self, PresetId};
use crate::ui::{self, UiAction};

const TRAY_ID: &str = "main";

/// Halo's colours for a non-monochrome icon.
const AMBER: [u8; 3] = [242, 181, 68];
const SKY: [u8; 3] = [108, 182, 255];

/// What the menu bar item shows, including how the icon is coloured.
#[derive(Clone, PartialEq, Eq)]
struct Shown {
    view: TrayView,
    /// `None` is a template icon; otherwise the icon's colour.
    tint: Option<[u8; 3]>,
}

/// What the menu bar item last showed, so updates only touch what changed.
#[derive(Default)]
pub struct TrayCache {
    shown: Option<Shown>,
    menu: Option<MenuKey>,
}

/// The menu's shape: it's rebuilt only when this changes, not every second.
#[derive(Clone, Copy, PartialEq, Eq)]
struct MenuKey {
    /// The start items only work when no session is running.
    can_start: bool,
}

impl MenuKey {
    fn of(_app: &AppHandle, s: &Snapshot) -> Self {
        MenuKey {
            can_start: s.status == Status::Idle,
        }
    }
}

fn shown(app: &AppHandle, s: &Snapshot) -> Shown {
    let prefs = crate::prefs::get(app);
    let view = halo_menubar::view(s, prefs.menubar_options());
    let tint = (!prefs.menubar_monochrome).then(|| {
        if s.status != Status::Idle && (s.phase.is_break() || s.overflowing) {
            SKY
        } else {
            AMBER
        }
    });
    Shown { view, tint }
}

/// Creates the menu bar item. Called once at startup.
pub fn create(app: &AppHandle, initial: &Snapshot) -> Result<()> {
    let first = shown(app, initial);
    let key = MenuKey::of(app, initial);
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(image(first.view.icon, first.tint))
        .icon_as_template(first.tint.is_none())
        .tooltip(&first.view.tooltip)
        .menu(&build_menu(app, key)?)
        .show_menu_on_left_click(false)
        .on_menu_event(on_menu)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
                && let Err(e) = crate::windows::toggle_popover(tray.app_handle(), rect)
            {
                eprintln!("popover: {e}");
            }
        })
        .build(app)?;
    if first.view.title.is_some()
        && let Some(tray) = app.tray_by_id(TRAY_ID)
    {
        tray.set_title(first.view.title.as_deref())?;
    }
    if let Ok(mut cache) = app.state::<Mutex<TrayCache>>().lock() {
        cache.shown = Some(first);
        cache.menu = Some(key);
    }
    Ok(())
}

/// Brings the menu bar item up to date with the timer.
pub fn sync(app: &AppHandle, snapshot: &Snapshot) -> Result<()> {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return Ok(());
    };
    let next = shown(app, snapshot);
    let key = MenuKey::of(app, snapshot);
    let cache = app.state::<Mutex<TrayCache>>();
    let Ok(mut cache) = cache.lock() else {
        return Ok(());
    };
    let last = cache.shown.clone();

    if last.as_ref().map(|s| &s.view.title) != Some(&next.view.title) {
        tray.set_title(next.view.title.as_deref())?;
    }
    if last.as_ref().map(|s| (s.view.icon, s.tint)) != Some((next.view.icon, next.tint)) {
        tray.set_icon(Some(image(next.view.icon, next.tint)))?;
        tray.set_icon_as_template(next.tint.is_none())?;
    }
    if last.as_ref().map(|s| &s.view.tooltip) != Some(&next.view.tooltip) {
        tray.set_tooltip(Some(&next.view.tooltip))?;
    }
    if cache.menu != Some(key) {
        tray.set_menu(Some(build_menu(app, key)?))?;
        cache.menu = Some(key);
    }
    cache.shown = Some(next);
    Ok(())
}

/// Updates the item after something outside the timer changed (prefs, the
/// mini timer).
pub fn refresh(app: &AppHandle) -> Result<()> {
    let snapshot = crate::timer::engine_snapshot(&app.state::<crate::state::AppState>())?;
    sync(app, &snapshot)
}

fn image(icon: Icon, tint: Option<[u8; 3]>) -> Image<'static> {
    Image::new_owned(halo_menubar::render(icon, tint), ICON_SIZE, ICON_SIZE)
}

/// The right-click menu, as in the design (`design/canvas/MenuBar.dc.html`).
/// Shortcuts are shown but not wired yet. Shield, head check and the trading
/// desk are greyed out until their phases (6 and 5).
fn build_menu(app: &AppHandle, key: MenuKey) -> Result<Menu<Wry>> {
    let item = |id: &str, text: &str, enabled: bool, accel: &str| {
        MenuItem::with_id(app, id, text, enabled, Some(accel))
    };
    let sep = || PredefinedMenuItem::separator(app);
    let menu = Menu::new(app)?;
    menu.append(&item(
        "start:pomodoro",
        "Start Pomodoro 25",
        key.can_start,
        "CmdOrCtrl+1",
    )?)?;
    menu.append(&item(
        "start:deep",
        "Start Deep 50",
        key.can_start,
        "CmdOrCtrl+2",
    )?)?;
    menu.append(&item(
        "start:flowtime",
        "Start Flowtime",
        key.can_start,
        "CmdOrCtrl+3",
    )?)?;
    menu.append(&sep()?)?;
    menu.append(&item(
        "shield",
        "Turn Shield on or off",
        false,
        "CmdOrCtrl+Shift+S",
    )?)?;
    menu.append(&item(
        "head-check",
        "Head check…",
        false,
        "CmdOrCtrl+Shift+H",
    )?)?;
    menu.append(&item("trading", "Open Trading desk", false, "CmdOrCtrl+T")?)?;
    menu.append(&sep()?)?;
    menu.append(&item("settings", "Settings…", true, "CmdOrCtrl+,")?)?;
    menu.append(&item("quit", "Quit Halo", true, "CmdOrCtrl+Q")?)?;
    Ok(menu)
}

fn on_menu(app: &AppHandle, event: MenuEvent) {
    let result = match event.id().as_ref() {
        "start:pomodoro" => timer::start(app, PresetId::Pomodoro, String::new()).map(drop),
        "start:deep" => timer::start(app, PresetId::Deep, String::new()).map(drop),
        "start:flowtime" => timer::start(app, PresetId::Flowtime, String::new()).map(drop),
        "settings" => ui::request(app, UiAction::OpenSettings),
        "quit" => {
            app.exit(0);
            Ok(())
        }
        _ => Ok(()),
    };
    if let Err(e) = result {
        eprintln!("menu bar: {e}");
    }
}
