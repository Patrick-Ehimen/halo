//! Chart time: every few seconds, which charting app or site is in front.
//! See docs/adr/0001-frontmost-app-and-browser-url.md.

use std::collections::HashMap;
use std::time::Duration;

use halo_trading::ChartRule;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use super::{Desk, now_i64, settings};
use crate::error::{Error, Result};
use crate::state::AppState;

const POLL: Duration = Duration::from_secs(5);
/// A stretch continues across gaps up to this long (a quick look elsewhere).
const GAP_MS: i64 = 15_000;
/// Ask a browser that refused Automation again after this long.
const RETRY_BLOCKED_MS: i64 = 10 * 60_000;

/// What the tracker saw last.
#[derive(Debug, Default)]
pub struct Tracker {
    current: Option<Current>,
    /// When charting outside the window started, for the nudge.
    outside_since: Option<i64>,
    nudged: bool,
    /// Browsers that refused Automation, by bundle id, and when.
    blocked: HashMap<String, i64>,
}

#[derive(Debug)]
struct Current {
    row: i64,
    label: String,
    last_seen: i64,
}

/// A charting app or site in front right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    pub label: String,
    pub app: String,
    pub host: Option<String>,
}

/// What a poll should do, worked out without touching the database.
#[derive(Debug, PartialEq, Eq)]
enum Step {
    Extend(i64),
    Start,
    Idle,
}

impl Tracker {
    pub fn forget_blocked(&mut self) {
        self.blocked.clear();
    }

    pub fn any_blocked(&self) -> bool {
        !self.blocked.is_empty()
    }

    fn step(&self, seen: Option<&Seen>, now: i64) -> Step {
        match (seen, &self.current) {
            (Some(s), Some(c)) if c.label == s.label && now - c.last_seen <= GAP_MS => {
                Step::Extend(c.row)
            }
            (Some(_), _) => Step::Start,
            (None, _) => Step::Idle,
        }
    }

    /// Whether to nudge now. Charting outside the window continuously (gaps
    /// under `GAP_MS` allowed) for `after_ms` nudges once per stretch.
    fn nudge_due(&mut self, charting_outside: bool, now: i64, after_ms: i64) -> bool {
        if charting_outside {
            let since = *self.outside_since.get_or_insert(now);
            if !self.nudged && now - since >= after_ms {
                self.nudged = true;
                return true;
            }
        } else if self
            .current
            .as_ref()
            .is_none_or(|c| now - c.last_seen > GAP_MS)
        {
            self.outside_since = None;
            self.nudged = false;
        }
        false
    }
}

/// Starts the background poll. Runs for the life of the app.
pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(POLL);
            if let Err(e) = poll_once(&app) {
                eprintln!("chart time: {e}");
            }
        }
    });
}

fn poll_once(app: &AppHandle) -> Result<()> {
    let s = settings(app);
    let now = now_i64();
    let seen = look(app, &s.rules(), s.track_browsers, now);
    let in_window = halo_trading::window_state(&s.window(), now).open;

    let desk = app.state::<Desk>();
    let nudge = {
        let mut t = desk.tracker.lock().map_err(|_| Error::Poisoned)?;
        match t.step(seen.as_ref(), now) {
            Step::Extend(row) => {
                let state = app.state::<AppState>();
                let conn = state.db()?;
                halo_store::trading::extend_chart(&conn, row, now)?;
                if let Some(c) = t.current.as_mut() {
                    c.last_seen = now;
                }
            }
            Step::Start => {
                if let Some(seen) = &seen {
                    let state = app.state::<AppState>();
                    let conn = state.db()?;
                    let row = halo_store::trading::start_chart(
                        &conn,
                        &seen.label,
                        &seen.app,
                        seen.host.as_deref(),
                        now,
                    )?;
                    t.current = Some(Current {
                        row,
                        label: seen.label.clone(),
                        last_seen: now,
                    });
                }
            }
            Step::Idle => {}
        }
        let after = i64::from(s.nudge_minutes) * 60_000;
        let due = t.nudge_due(seen.is_some() && !in_window, now, after);
        (due && s.nudge).then(|| seen.as_ref().map(|x| x.label.clone()))
    };

    if let Some(label) = nudge.flatten() {
        let body = format!(
            "{} min on {label} outside your window. Step away until it opens.",
            s.nudge_minutes
        );
        if let Err(e) = app
            .notification()
            .builder()
            .title("Charting outside your window")
            .body(body)
            .show()
        {
            eprintln!("nudge: {e}");
        }
    }
    // The menu bar's trading countdown moves by the minute.
    crate::tray::refresh(app)?;
    Ok(())
}

/// The charting app or site in front, if any.
#[cfg(target_os = "macos")]
fn look(app: &AppHandle, rules: &[ChartRule], browsers: bool, now: i64) -> Option<Seen> {
    use crate::platform::macos::{browser, front_app};

    let front = front_app()?;
    let host = if browsers && browser::is_supported(&front.bundle_id) {
        let skip = app
            .state::<Desk>()
            .tracker
            .lock()
            .ok()
            .and_then(|t| t.blocked.get(&front.bundle_id).copied())
            .is_some_and(|at| now - at < RETRY_BLOCKED_MS);
        if skip {
            None
        } else {
            match browser::front_tab_host(&front.bundle_id) {
                browser::TabHost::Host(h) => {
                    if let Ok(mut t) = app.state::<Desk>().tracker.lock() {
                        t.blocked.remove(&front.bundle_id);
                    }
                    Some(h)
                }
                browser::TabHost::NotAllowed => {
                    if let Ok(mut t) = app.state::<Desk>().tracker.lock() {
                        t.blocked.insert(front.bundle_id.clone(), now);
                    }
                    None
                }
                _ => None,
            }
        }
    } else {
        None
    };
    ChartRule::find(rules, &front.name, &front.bundle_id, host.as_deref()).map(|r| Seen {
        label: r.label.clone(),
        app: front.name.clone(),
        host,
    })
}

#[cfg(not(target_os = "macos"))]
fn look(_app: &AppHandle, _rules: &[ChartRule], _browsers: bool, _now: i64) -> Option<Seen> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seen(label: &str) -> Seen {
        Seen {
            label: label.into(),
            app: "TradingView".into(),
            host: None,
        }
    }

    #[test]
    fn stretches_extend_until_a_gap_or_a_new_source() {
        let mut t = Tracker::default();
        assert_eq!(t.step(Some(&seen("TradingView")), 0), Step::Start);
        t.current = Some(Current {
            row: 7,
            label: "TradingView".into(),
            last_seen: 0,
        });
        assert_eq!(t.step(Some(&seen("TradingView")), 5_000), Step::Extend(7));
        assert_eq!(t.step(Some(&seen("Binance")), 5_000), Step::Start);
        assert_eq!(t.step(Some(&seen("TradingView")), 20_000), Step::Start);
        assert_eq!(t.step(None, 5_000), Step::Idle);
    }

    #[test]
    fn nudges_once_after_five_minutes_outside() {
        let mut t = Tracker::default();
        let five = 5 * 60_000;
        assert!(!t.nudge_due(true, 0, five));
        assert!(!t.nudge_due(true, 4 * 60_000, five));
        assert!(t.nudge_due(true, five, five));
        assert!(!t.nudge_due(true, five + 5_000, five), "once per stretch");
        // Stop charting long enough and it starts over.
        t.current = Some(Current {
            row: 1,
            label: "TradingView".into(),
            last_seen: five + 5_000,
        });
        assert!(!t.nudge_due(false, five + 60_000, five));
        assert!(t.outside_since.is_none());
        assert!(!t.nudge_due(true, five + 60_000, five));
    }

    #[test]
    fn inside_the_window_never_nudges() {
        let mut t = Tracker::default();
        for m in 0..20 {
            assert!(!t.nudge_due(false, m * 60_000, 5 * 60_000));
        }
    }
}
