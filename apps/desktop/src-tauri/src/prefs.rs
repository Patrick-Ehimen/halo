//! App preferences that aren't timer rules: alerts, the break overlay and the
//! menu bar's look. Saved as JSON under the `prefs` settings key.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

use crate::error::Result;
use crate::state::AppState;
use crate::timer::now_ms;

const SETTINGS_KEY: &str = "prefs";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum MenubarShows {
    /// The ring and the time.
    Timer,
    /// The ring alone.
    Icon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct Prefs {
    /// Native notifications when a session completes, a goal is reached, or a break ends.
    pub notifications: bool,
    /// Cover every display during breaks.
    pub break_overlay: bool,
    pub menubar_shows: MenubarShows,
    pub menubar_seconds: bool,
    /// Template icon that matches other menu bar icons; off uses Halo's colours.
    pub menubar_monochrome: bool,
    /// Daily focus goal in minutes: the goal ring, the tree, the heatmap.
    pub daily_goal_minutes: u32,
    /// Ask "How did it go?" after each session.
    pub reflect: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            notifications: true,
            break_overlay: true,
            menubar_shows: MenubarShows::Timer,
            menubar_seconds: true,
            menubar_monochrome: true,
            daily_goal_minutes: 300,
            reflect: true,
        }
    }
}

impl Prefs {
    pub fn menubar_options(&self) -> halo_menubar::Options {
        halo_menubar::Options {
            show_time: self.menubar_shows == MenubarShows::Timer,
            show_seconds: self.menubar_seconds,
        }
    }
}

pub type PrefsState = Mutex<Prefs>;

/// Loads saved prefs, falling back to defaults (and filling in new fields).
pub fn load(state: &AppState) -> Prefs {
    state
        .db()
        .ok()
        .and_then(|conn| {
            halo_store::settings::get(&conn, SETTINGS_KEY)
                .ok()
                .flatten()
        })
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

pub fn get(app: &AppHandle) -> Prefs {
    app.state::<PrefsState>()
        .lock()
        .map(|p| *p)
        .unwrap_or_default()
}

/// Saves and applies new prefs.
pub fn set(app: &AppHandle, prefs: Prefs) -> Result<Prefs> {
    if !(30..=16 * 60).contains(&prefs.daily_goal_minutes) {
        return Err(crate::error::Error::Invalid(
            "Set a daily goal between 30 minutes and 16 hours",
        ));
    }
    let json = serde_json::to_string(&prefs)?;
    {
        let state = app.state::<AppState>();
        let conn = state.db()?;
        halo_store::settings::set(
            &conn,
            SETTINGS_KEY,
            &json,
            i64::try_from(now_ms()).unwrap_or(i64::MAX),
        )?;
    }
    if let Ok(mut current) = app.state::<PrefsState>().lock() {
        *current = prefs;
    }
    crate::tray::refresh(app)?;
    if !prefs.break_overlay {
        crate::overlay::close(app)?;
    }
    Ok(prefs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_take_defaults() {
        let p: Prefs = serde_json::from_str(r#"{"notifications":false}"#).unwrap();
        assert_eq!(
            p,
            Prefs {
                notifications: false,
                ..Prefs::default()
            }
        );
    }

    #[test]
    fn icon_only_hides_the_time() {
        let p = Prefs {
            menubar_shows: MenubarShows::Icon,
            menubar_seconds: false,
            ..Prefs::default()
        };
        let o = p.menubar_options();
        assert!(!o.show_time);
        assert!(!o.show_seconds);
    }
}
