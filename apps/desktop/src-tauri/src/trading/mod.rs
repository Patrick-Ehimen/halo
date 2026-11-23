//! The trading desk: session clocks, the pre-trade checklist, the head check,
//! the unlock gate and chart time. The rules live in `halo-trading`; this
//! module reads storage and formats everything in the user's zone.
//!
//! Halo never places trades, holds exchange keys, or gives advice. It only
//! manages time, attention and the user's own rules.

mod dto;
pub mod settings;
pub mod tracker;

use std::collections::BTreeMap;
use std::sync::Mutex;

use chrono::{Datelike, Days, NaiveDate, TimeZone};
use chrono_tz::Tz;
use halo_plan::day_bounds;
use halo_store::Connection;
use halo_trading::{Gate, Market, Span, countdown, fx_weekend, gate, market_state, spans};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

pub use dto::TradingView;
use dto::{
    ChartDay, ChartSourceTime, ChartView, CheckItemView, ClockView, GateState, GateView,
    HeadCheckView, Segment, TimelineRow,
};
pub use settings::TradingSettings;
use settings::{MarketId, SETTINGS_KEY};

use crate::error::{Error, Result};
use crate::state::AppState;
use crate::timer::now_ms;

/// Head check factors to pick from.
pub const FACTORS: [&str; 5] = [
    "Slept well",
    "Took a loss today",
    "Feeling rushed",
    "FOMO",
    "Calm",
];

/// The checklist a new desk starts with. Editable.
const DEFAULT_CHECKLIST: [(&str, &str); 6] = [
    (
        "Higher-timeframe bias written",
        "Daily and 4H, one sentence",
    ),
    ("Key levels marked", "On your charts before the open"),
    (
        "News checked for today",
        "High-impact events on the calendar",
    ),
    ("Risk per trade set", "Max 0.5% of the account"),
    ("Max trades for today set", "Default 3"),
    ("Not chasing a loss", "Last loss was more than 24h ago"),
];

const DAY_MS: i64 = 86_400_000;

/// Trading desk state managed by Tauri: settings and the chart-time tracker.
pub struct Desk {
    settings: Mutex<TradingSettings>,
    pub tracker: Mutex<tracker::Tracker>,
}

impl Desk {
    pub fn load(state: &AppState) -> Self {
        let settings = state
            .db()
            .ok()
            .and_then(|conn| {
                halo_store::settings::get(&conn, SETTINGS_KEY)
                    .ok()
                    .flatten()
            })
            .and_then(|json| serde_json::from_str::<TradingSettings>(&json).ok())
            .map(TradingSettings::tidied)
            .unwrap_or_default();
        Desk {
            settings: Mutex::new(settings),
            tracker: Mutex::new(tracker::Tracker::default()),
        }
    }
}

/// Tells every window the desk changed (checklist, head check, settings).
#[derive(Debug, Clone, Serialize, Type, tauri_specta::Event)]
#[tauri_specta(event_name = "trading://changed")]
pub struct TradingChanged;

fn changed(app: &AppHandle) {
    if let Err(e) = TradingChanged.emit(app) {
        eprintln!("trading event: {e}");
    }
}

fn now_i64() -> i64 {
    i64::try_from(now_ms()).unwrap_or(i64::MAX)
}

pub fn settings(app: &AppHandle) -> TradingSettings {
    app.state::<Desk>()
        .settings
        .lock()
        .map(|s| s.clone())
        .unwrap_or_default()
}

pub fn set_settings(app: &AppHandle, next: TradingSettings) -> Result<TradingSettings> {
    let next = next.validated()?;
    let json = serde_json::to_string(&next)?;
    {
        let state = app.state::<AppState>();
        let conn = state.db()?;
        halo_store::settings::set(&conn, SETTINGS_KEY, &json, now_i64())?;
    }
    let turned_on_browsers = {
        let desk = app.state::<Desk>();
        let mut current = desk.settings.lock().map_err(|_| Error::Poisoned)?;
        let on = next.track_browsers && !current.track_browsers;
        *current = next.clone();
        on
    };
    if turned_on_browsers && let Ok(mut t) = app.state::<Desk>().tracker.lock() {
        // Ask again: the user may have just allowed Automation.
        t.forget_blocked();
    }
    crate::tray::refresh(app)?;
    changed(app);
    Ok(next)
}

// ---------------------------------------------------------------- view

pub fn view(app: &AppHandle) -> Result<TradingView> {
    let s = settings(app);
    let blocked = app
        .state::<Desk>()
        .tracker
        .lock()
        .map(|t| t.any_blocked())
        .unwrap_or(false);
    let state = app.state::<AppState>();
    let conn = state.db()?;
    view_in(&conn, &s, now_i64(), blocked)
}

fn date_of(tz: &Tz, ms: i64) -> NaiveDate {
    tz.timestamp_millis_opt(ms)
        .single()
        .map_or_else(|| chrono::Utc::now().date_naive(), |t| t.date_naive())
}

fn hm(tz: &Tz, ms: i64) -> String {
    tz.timestamp_millis_opt(ms)
        .single()
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or_default()
}

/// "Africa/Lagos" → "Lagos", "America/New_York" → "New York".
fn zone_label(zone: &str) -> String {
    zone.rsplit('/').next().unwrap_or(zone).replace('_', " ")
}

/// "4h 31m", "12m". For short labels.
fn short_countdown(ms: i64) -> String {
    let mins = (ms.max(0) + 59_999) / 60_000;
    if mins >= 60 {
        format!("{}h {:02}m", mins / 60, mins % 60)
    } else {
        format!("{mins}m")
    }
}

fn mins_label(ms: i64) -> String {
    short_countdown(ms - ms.rem_euclid(60_000)).replace("h 00m", "h")
}

fn market_id(m: Market) -> MarketId {
    match m {
        Market::Sydney => MarketId::Sydney,
        Market::Tokyo => MarketId::Tokyo,
        Market::London => MarketId::London,
        Market::NewYork => MarketId::NewYork,
    }
}

/// Settings key set once the suggested checklist has been offered, so a
/// checklist the user emptied stays empty.
const CHECKLIST_SEEDED: &str = "trading.checklistSeeded";

/// Adds the suggested checklist the first time the desk opens, and never again.
fn ensure_checklist(conn: &Connection, now: i64) -> Result<()> {
    if halo_store::settings::get(conn, CHECKLIST_SEEDED)?.is_some() {
        return Ok(());
    }
    if !halo_store::trading::checklist_started(conn)? {
        add_suggested(conn, now)?;
    }
    halo_store::settings::set(conn, CHECKLIST_SEEDED, "true", now)?;
    Ok(())
}

fn add_suggested(conn: &Connection, now: i64) -> Result<()> {
    for (label, hint) in DEFAULT_CHECKLIST {
        halo_store::trading::add_item(conn, label, hint, now)?;
    }
    Ok(())
}

/// Adds the suggested rules back, skipping any already on the checklist.
pub fn restore_suggested(app: &AppHandle) -> Result<()> {
    {
        let state = app.state::<AppState>();
        let conn = state.db()?;
        let date = date_of(&settings(app).tz(), now_i64())
            .format("%Y-%m-%d")
            .to_string();
        let have: Vec<String> = halo_store::trading::checklist(&conn, &date)?
            .into_iter()
            .map(|i| i.label.to_lowercase())
            .collect();
        for (label, hint) in DEFAULT_CHECKLIST {
            if !have.contains(&label.to_lowercase()) {
                halo_store::trading::add_item(&conn, label, hint, now_i64())?;
            }
        }
    }
    changed(app);
    Ok(())
}

fn view_in(conn: &Connection, s: &TradingSettings, now: i64, blocked: bool) -> Result<TradingView> {
    ensure_checklist(conn, now)?;
    let tz = s.tz();
    let today = date_of(&tz, now);
    let today_key = today.format("%Y-%m-%d").to_string();
    let window = s.window();
    let mine = window.market;
    let weekend = fx_weekend(now);

    // Clocks.
    let clocks: Vec<ClockView> = Market::ALL
        .iter()
        .map(|&m| {
            let st = market_state(m, now);
            let next = spans(m, now, now + 9 * DAY_MS);
            let range = next
                .first()
                .map(|sp| format!("{}–{}", hm(&tz, sp.start), hm(&tz, sp.end)))
                .unwrap_or_default();
            let (status, countdown_label) = if weekend {
                (
                    "Weekend",
                    "FX desks closed · crypto still trades".to_string(),
                )
            } else if st.open {
                (
                    "Open",
                    format!("Closes in {}", countdown(st.next_change - now)),
                )
            } else {
                (
                    "Closed",
                    format!("Opens in {}", countdown(st.next_change - now)),
                )
            };
            ClockView {
                market: market_id(m),
                name: m.name().to_string(),
                mine: m == mine,
                open: st.open,
                status: status.to_string(),
                local_time: hm(&m.zone(), now),
                range,
                countdown: countdown_label,
            }
        })
        .collect();

    // Your window.
    let ws = halo_trading::window_state(&window, now);
    let has_next = ws.next_change != i64::MAX;
    let (headline, headline_time) = if ws.open {
        (
            "Your window is open · closes in".to_string(),
            countdown(ws.next_change - now),
        )
    } else if !has_next {
        ("No trading window set".to_string(), String::new())
    } else if weekend {
        let day = tz
            .timestamp_millis_opt(ws.next_change)
            .single()
            .map(|t| t.format("%A").to_string())
            .unwrap_or_default();
        (
            format!("Weekend · your window reopens {day}"),
            String::new(),
        )
    } else {
        (
            "Your window opens in".to_string(),
            countdown(ws.next_change - now),
        )
    };
    let short_label = match (ws.open, has_next) {
        (true, _) => format!(
            "{} closes in {}",
            mine.short(),
            short_countdown(ws.next_change - now)
        ),
        (false, true) => format!(
            "{} opens in {}",
            mine.short(),
            short_countdown(ws.next_change - now)
        ),
        (false, false) => "No trading window".to_string(),
    };
    let overlap = market_state(Market::London, now).open && market_state(Market::NewYork, now).open;

    // 24-hour strips in your zone.
    let (day_start, day_end) = day_bounds(&tz, today);
    let permille = |ms: i64| -> u32 {
        let len = (day_end - day_start).max(1);
        u32::try_from(((ms.clamp(day_start, day_end) - day_start) * 1000) / len).unwrap_or(1000)
    };
    let timeline = Market::ALL
        .iter()
        .map(|&m| TimelineRow {
            name: m.name().to_string(),
            mine: m == mine,
            segments: halo_trading::day_segments(m, &tz, today)
                .into_iter()
                .map(|(start, end)| Segment {
                    start,
                    end,
                    inside: m == mine,
                })
                .collect(),
        })
        .collect();

    // Checklist.
    let items = halo_store::trading::checklist(conn, &today_key)?;
    let done = items.iter().filter(|i| i.checked).count();
    let checklist_done = done == items.len();
    let checklist_summary = if items.is_empty() {
        "Add the rules you check before trading".to_string()
    } else if checklist_done {
        "All done for today".to_string()
    } else {
        format!("{done} of {} done · resets every morning", items.len())
    };

    // Head check.
    let latest = halo_store::trading::latest_head_check(conn, &today_key)?;
    let lowest = halo_store::trading::lowest_head_check(conn, &today_key)?;
    let locked_today = lowest.is_some_and(|l| l <= 2);
    let head_check = latest.as_ref().map(|h| HeadCheckView {
        score: h.score,
        factors: serde_json::from_str(&h.factors_json).unwrap_or_default(),
        at_label: format!("Logged at {}", hm(&tz, h.at)),
        locked_today,
    });
    let score = if locked_today {
        lowest
    } else {
        latest.map(|h| h.score)
    };

    // The gate.
    let g = gate(checklist_done, score, ws.open);
    let gate_view = gate_view(
        g,
        if has_next {
            Some(hm(&tz, ws.next_change))
        } else {
            None
        },
    );

    let chart = chart_view(conn, s, &tz, today, now, blocked, &permille)?;

    Ok(TradingView {
        zone_label: zone_label(&s.zone),
        now_label: tz
            .timestamp_millis_opt(now)
            .single()
            .map(|t| t.format("%H:%M:%S").to_string())
            .unwrap_or_default(),
        headline,
        headline_time,
        window_open: ws.open,
        weekend,
        overlap,
        clocks,
        timeline,
        now_permille: permille(now),
        checklist: items
            .into_iter()
            .map(|i| CheckItemView {
                id: i.id,
                label: i.label,
                hint: i.hint,
                checked: i.checked,
            })
            .collect(),
        checklist_summary,
        checklist_done: u32::try_from(done).unwrap_or(u32::MAX),
        head_check,
        factor_options: FACTORS.iter().map(|f| (*f).to_string()).collect(),
        gate: gate_view,
        chart,
        short_label,
    })
}

fn gate_view(g: Gate, opens_at: Option<String>) -> GateView {
    match g {
        Gate::SitOut => GateView {
            state: GateState::SitOut,
            label: "Sitting out today".into(),
            detail: "Your head check says rest. Protecting the account is a win too.".into(),
            half_size: false,
        },
        Gate::Steps {
            checklist,
            head_check,
        } => {
            let n = u8::from(checklist) + u8::from(head_check);
            GateView {
                state: GateState::Steps,
                label: format!("{n} {} to unlock", if n == 1 { "step" } else { "steps" }),
                detail: match (checklist, head_check) {
                    (true, true) => {
                        "Finish the checklist and the head check to unlock your window."
                    }
                    (true, false) => "Finish the checklist to unlock your window.",
                    _ => "Do the head check to unlock your window.",
                }
                .into(),
                half_size: false,
            }
        }
        Gate::Ready { half_size } => GateView {
            state: GateState::Ready,
            label: opens_at.map_or_else(|| "Ready".to_string(), |t| format!("Ready for {t}")),
            detail: if half_size {
                "Rules met. When your window opens, trade at half size, max 2 trades."
            } else {
                "Checklist done and head check logged. You're clear when your window opens."
            }
            .into(),
            half_size,
        },
        Gate::Open { half_size } => GateView {
            state: GateState::Open,
            label: "Clear to trade".into(),
            detail: if half_size {
                "Trade at half size, max 2 trades today."
            } else {
                "Inside your window with your rules met."
            }
            .into(),
            half_size,
        },
    }
}

/// Splits `[a, b)` at the window's edges: (start, end, inside).
fn split(a: i64, b: i64, window: &[Span]) -> Vec<(i64, i64, bool)> {
    let mut cuts = vec![a, b];
    for s in window {
        for t in [s.start, s.end] {
            if t > a && t < b {
                cuts.push(t);
            }
        }
    }
    cuts.sort_unstable();
    cuts.dedup();
    cuts.windows(2)
        .filter_map(|w| match w {
            [x, y] => Some((*x, *y, window.iter().any(|s| s.contains(*x)))),
            _ => None,
        })
        .collect()
}

/// Colours for charting sources, by their place in the list. Green and
/// coral are left out: they mean in and outside the window.
const SOURCE_COLORS: [&str; 6] = [
    "#F2B544", "#6CB6FF", "#C59CFF", "#7FD1C9", "#F28DB2", "#B8D86B",
];

/// A source's colour: by its place in the user's list, so it doesn't change
/// from day to day. Sources removed since keep a neutral grey.
fn source_color(s: &TradingSettings, label: &str) -> &'static str {
    s.sources
        .iter()
        .position(|x| x.label == label)
        .and_then(|i| SOURCE_COLORS.get(i % SOURCE_COLORS.len()))
        .copied()
        .unwrap_or("#8A8E9E")
}

/// "A", "A and B", "A, B and C".
fn and_list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// A chart stretch cut at the window's edges: (source, start, end, inside).
type Piece = (String, i64, i64, bool);
/// A day's chart time: (total ms, outside ms, pieces).
type DayChart = (i64, i64, Vec<Piece>);

fn chart_view(
    conn: &Connection,
    s: &TradingSettings,
    tz: &Tz,
    today: NaiveDate,
    now: i64,
    blocked: bool,
    permille: &dyn Fn(i64) -> u32,
) -> Result<ChartView> {
    let window = s.window();
    let outside_on = |date: NaiveDate| -> Result<DayChart> {
        let (start, end) = day_bounds(tz, date);
        let end = end.min(now.max(start));
        let ws = halo_trading::window_spans(&window, start, end);
        let mut total = 0;
        let mut outside = 0;
        let mut pieces = Vec::new();
        for a in halo_store::trading::chart_between(conn, start, end)? {
            let (from, to) = (a.started_at.max(start), a.ended_at.min(end));
            for (x, y, inside) in split(from, to, &ws) {
                total += y - x;
                if !inside {
                    outside += y - x;
                }
                pieces.push((a.label.clone(), x, y, inside));
            }
        }
        Ok((total, outside, pieces))
    };

    let (total, outside, pieces) = outside_on(today)?;
    let mut by_source: BTreeMap<String, i64> = BTreeMap::new();
    for (label, x, y, inside) in &pieces {
        if !inside {
            *by_source.entry(label.clone()).or_default() += y - x;
        }
    }
    let mut sources: Vec<(String, i64)> = by_source.into_iter().filter(|(_, ms)| *ms > 0).collect();
    sources.sort_by_key(|(_, ms)| std::cmp::Reverse(*ms));

    let mut week = Vec::new();
    for back in (0..7u64).rev() {
        let date = today.checked_sub_days(Days::new(back)).unwrap_or(today);
        let (_, out, _) = outside_on(date)?;
        week.push(ChartDay {
            label: date
                .weekday()
                .to_string()
                .chars()
                .next()
                .map(String::from)
                .unwrap_or_default(),
            minutes: u32::try_from(out / 60_000).unwrap_or(u32::MAX),
            today: back == 0,
        });
    }

    let (day_start, day_end) = day_bounds(tz, today);
    let labels: Vec<String> = s.sources.iter().map(|x| x.label.clone()).collect();
    let (browser_note, browser_blocked) = if blocked && s.track_browsers {
        (
            Some("Browser sites aren't counted: allow Halo under System Settings → Privacy & Security → Automation.".to_string()),
            true,
        )
    } else if !s.track_browsers {
        (
            Some(
                "Only apps are counted. Turn on browser tracking to count charting sites too."
                    .to_string(),
            ),
            false,
        )
    } else {
        (None, false)
    };

    Ok(ChartView {
        outside_label: mins_label(outside),
        total_label: format!("outside of {} today", mins_label(total)),
        segments: pieces
            .iter()
            .map(|(_, x, y, inside)| Segment {
                start: permille(*x),
                end: permille(*y).max(permille(*x) + 1),
                inside: *inside,
            })
            .collect(),
        window: halo_trading::window_spans(&window, day_start, day_end)
            .iter()
            .map(|sp| Segment {
                start: permille(sp.start),
                end: permille(sp.end),
                inside: true,
            })
            .collect(),
        week,
        sources: sources
            .into_iter()
            .map(|(label, ms)| {
                let permille = u32::try_from((ms * 1000).checked_div(outside).unwrap_or(0))
                    .unwrap_or(1000)
                    .min(1000);
                ChartSourceTime {
                    color: source_color(s, &label).to_string(),
                    outside: if ms < 60_000 {
                        "<1m".to_string()
                    } else {
                        mins_label(ms)
                    },
                    share: format!("{}%", (permille + 5) / 10),
                    permille,
                    label,
                }
            })
            .collect(),
        tracking_label: if labels.is_empty() {
            "No charting apps or sites set".into()
        } else {
            format!("Tracked from {}", and_list(&labels))
        },
        browser_note,
        browser_blocked,
    })
}

/// "NY in 4:31" before the window, "NY open 1:02" during it. For the menu bar.
pub fn menubar_label(app: &AppHandle, now: i64) -> Option<String> {
    let s = settings(app);
    if !s.menubar_countdown {
        return None;
    }
    let w = s.window();
    let st = halo_trading::window_state(&w, now);
    if st.next_change == i64::MAX {
        return None;
    }
    let mins = ((st.next_change - now).max(0) + 59_999) / 60_000;
    let hm = format!("{}:{:02}", mins / 60, mins % 60);
    Some(if st.open {
        format!("{} open {hm}", w.market.short())
    } else {
        format!("{} in {hm}", w.market.short())
    })
}

// ---------------------------------------------------------------- actions

fn today_key(app: &AppHandle) -> String {
    date_of(&settings(app).tz(), now_i64())
        .format("%Y-%m-%d")
        .to_string()
}

pub fn set_checked(app: &AppHandle, item_id: &str, on: bool) -> Result<()> {
    let date = today_key(app);
    {
        let state = app.state::<AppState>();
        let conn = state.db()?;
        halo_store::trading::set_checked(&conn, item_id, &date, on, now_i64())?;
    }
    changed(app);
    Ok(())
}

pub fn add_item(app: &AppHandle, label: &str, hint: &str) -> Result<()> {
    let label = label.trim();
    if label.is_empty() {
        return Err(Error::Invalid("Write the rule first"));
    }
    if label.chars().count() > 120 || hint.chars().count() > 120 {
        return Err(Error::Invalid("Keep it under 120 characters"));
    }
    {
        let state = app.state::<AppState>();
        let conn = state.db()?;
        halo_store::trading::add_item(&conn, label, hint.trim(), now_i64())?;
    }
    changed(app);
    Ok(())
}

pub fn remove_item(app: &AppHandle, id: &str) -> Result<()> {
    {
        let state = app.state::<AppState>();
        let conn = state.db()?;
        halo_store::trading::remove_item(&conn, id, now_i64())?;
    }
    changed(app);
    Ok(())
}

pub fn reset_checklist(app: &AppHandle) -> Result<()> {
    let date = today_key(app);
    {
        let state = app.state::<AppState>();
        let conn = state.db()?;
        halo_store::trading::reset_checklist(&conn, &date, now_i64())?;
    }
    changed(app);
    Ok(())
}

pub fn head_check(app: &AppHandle, score: u8, factors: Vec<String>) -> Result<()> {
    let date = today_key(app);
    {
        let state = app.state::<AppState>();
        let conn = state.db()?;
        record_head_check(&conn, &date, now_i64(), score, factors)?;
    }
    changed(app);
    Ok(())
}

/// Takes back today's head check. A 1–2 can't be taken back: it locks the day.
pub fn clear_head_check(app: &AppHandle) -> Result<()> {
    let date = today_key(app);
    {
        let state = app.state::<AppState>();
        let conn = state.db()?;
        clear_today(&conn, &date, now_i64())?;
    }
    changed(app);
    Ok(())
}

fn clear_today(conn: &Connection, date: &str, now: i64) -> Result<()> {
    if halo_store::trading::lowest_head_check(conn, date)?.is_some_and(|l| l <= 2) {
        return Err(Error::Invalid(
            "You're sitting out today. Check in again tomorrow.",
        ));
    }
    halo_store::trading::clear_head_checks(conn, date, now)?;
    Ok(())
}

fn record_head_check(
    conn: &Connection,
    date: &str,
    now: i64,
    score: u8,
    factors: Vec<String>,
) -> Result<()> {
    if !(1..=5).contains(&score) {
        return Err(Error::Invalid("Rate your head from 1 to 5"));
    }
    if halo_store::trading::lowest_head_check(conn, date)?.is_some_and(|l| l <= 2) {
        return Err(Error::Invalid(
            "You're sitting out today. Check in again tomorrow.",
        ));
    }
    let mut picked: Vec<String> = FACTORS
        .iter()
        .filter(|f| factors.iter().any(|x| x == *f))
        .map(|f| (*f).to_string())
        .collect();
    picked.dedup();
    halo_store::trading::record_head_check(
        conn,
        date,
        now,
        score,
        &serde_json::to_string(&picked)?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use chrono::NaiveDateTime;

    use super::*;

    fn utc(s: &str) -> i64 {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M")
            .unwrap()
            .and_utc()
            .timestamp_millis()
    }

    fn db() -> Connection {
        halo_store::open(Path::new(":memory:")).unwrap()
    }

    #[test]
    fn a_weekday_morning_in_lagos() {
        let conn = db();
        // Tue 24 Nov 2026, 10:28 UTC = 11:28 Lagos. NY opens 14:30 UTC = 15:30 Lagos.
        let v = view_in(
            &conn,
            &TradingSettings::default(),
            utc("2026-11-24 10:28"),
            false,
        )
        .unwrap();
        assert_eq!(v.zone_label, "Lagos");
        assert_eq!(v.now_label, "11:28:00");
        assert_eq!(
            (v.headline.as_str(), v.headline_time.as_str()),
            ("Your window opens in", "4h 02m 00s")
        );
        assert_eq!(v.short_label, "NY opens in 4h 02m");
        assert!(!v.weekend);
        let ny = v.clocks.iter().find(|c| c.mine).unwrap();
        assert_eq!(
            (ny.name.as_str(), ny.range.as_str(), ny.status.as_str()),
            ("New York", "15:30–22:00", "Closed")
        );
        assert_eq!(ny.local_time, "05:28");
        let london = &v.clocks[2];
        assert_eq!(
            (london.status.as_str(), london.countdown.as_str()),
            ("Open", "Closes in 6h 02m 00s")
        );
        // Sydney crosses Lagos midnight: two pieces.
        assert_eq!(v.timeline[0].segments.len(), 2);
        // A fresh desk gets the default checklist and needs both steps.
        assert_eq!(v.checklist.len(), 6);
        assert_eq!(v.checklist_summary, "0 of 6 done · resets every morning");
        assert_eq!(v.gate.state, GateState::Steps);
        assert_eq!(v.gate.label, "2 steps to unlock");
        assert_eq!(
            v.chart.tracking_label,
            "Tracked from TradingView, Binance and DEX Screener"
        );
        assert!(v.chart.browser_note.is_some());
    }

    #[test]
    fn the_gate_opens_with_the_checklist_head_check_and_window() {
        let conn = db();
        let s = TradingSettings::default();
        let morning = utc("2026-11-24 10:28");
        let v = view_in(&conn, &s, morning, false).unwrap();
        for item in &v.checklist {
            halo_store::trading::set_checked(&conn, &item.id, "2026-11-24", true, morning).unwrap();
        }
        record_head_check(
            &conn,
            "2026-11-24",
            morning,
            4,
            vec!["Calm".into(), "Nope".into()],
        )
        .unwrap();
        let v = view_in(&conn, &s, morning, false).unwrap();
        assert_eq!(v.checklist_summary, "All done for today");
        assert_eq!(v.head_check.as_ref().unwrap().factors, ["Calm"]);
        assert_eq!(
            (v.gate.state, v.gate.label.as_str()),
            (GateState::Ready, "Ready for 15:30")
        );
        let v = view_in(&conn, &s, utc("2026-11-24 15:00"), false).unwrap();
        assert_eq!(v.gate.state, GateState::Open);
        assert!(v.window_open);
        assert!(v.overlap, "London and New York overlap at 15:00 UTC");
        // The next morning the checklist is empty again.
        let v = view_in(&conn, &s, utc("2026-11-25 08:00"), false).unwrap();
        assert_eq!(v.checklist_done, 0);
        assert!(v.head_check.is_none());
    }

    #[test]
    fn a_low_head_check_locks_the_day() {
        let conn = db();
        let s = TradingSettings::default();
        let t = utc("2026-11-24 10:00");
        record_head_check(&conn, "2026-11-24", t, 2, vec![]).unwrap();
        assert!(record_head_check(&conn, "2026-11-24", t + 1, 5, vec![]).is_err());
        let v = view_in(&conn, &s, utc("2026-11-24 15:00"), false).unwrap();
        assert_eq!(v.gate.state, GateState::SitOut);
        assert!(v.head_check.unwrap().locked_today);
        assert!(
            clear_today(&conn, "2026-11-24", t + 2).is_err(),
            "a low score can't be taken back"
        );
        assert!(record_head_check(&conn, "2026-11-25", t + DAY_MS, 3, vec![]).is_ok());
        clear_today(&conn, "2026-11-25", t + DAY_MS + 1).unwrap();
        let v = view_in(&conn, &s, utc("2026-11-25 10:00"), false).unwrap();
        assert!(v.head_check.is_none());
    }

    #[test]
    fn an_emptied_checklist_stays_empty() {
        let conn = db();
        let s = TradingSettings::default();
        let now = utc("2026-11-24 10:00");
        let v = view_in(&conn, &s, now, false).unwrap();
        assert_eq!(v.checklist.len(), 6);
        for item in &v.checklist {
            halo_store::trading::remove_item(&conn, &item.id, now).unwrap();
        }
        let v = view_in(&conn, &s, now + 1, false).unwrap();
        assert!(
            v.checklist.is_empty(),
            "the suggested rules don't come back"
        );
        assert_eq!(
            v.checklist_summary,
            "Add the rules you check before trading"
        );
        // Nothing to tick, so the checklist doesn't hold the gate.
        assert_eq!(
            v.gate.label, "1 step to unlock",
            "only the head check is left"
        );
    }

    #[test]
    fn the_weekend_headline() {
        let conn = db();
        let v = view_in(
            &conn,
            &TradingSettings::default(),
            utc("2026-11-28 12:00"),
            false,
        )
        .unwrap();
        assert!(v.weekend);
        assert_eq!(v.headline, "Weekend · your window reopens Monday");
        assert!(v.clocks.iter().all(|c| c.status == "Weekend"));
    }

    #[test]
    fn chart_time_outside_the_window() {
        let conn = db();
        let s = TradingSettings::default();
        // 13:00–13:41 UTC is outside; 14:20–14:40 straddles the 14:30 open.
        let id = halo_store::trading::start_chart(
            &conn,
            "TradingView",
            "TradingView",
            None,
            utc("2026-11-24 13:00"),
        )
        .unwrap();
        halo_store::trading::extend_chart(&conn, id, utc("2026-11-24 13:41")).unwrap();
        let id = halo_store::trading::start_chart(
            &conn,
            "DEX Screener",
            "Arc",
            Some("dexscreener.com"),
            utc("2026-11-24 14:20"),
        )
        .unwrap();
        halo_store::trading::extend_chart(&conn, id, utc("2026-11-24 14:40")).unwrap();
        let v = view_in(&conn, &s, utc("2026-11-24 16:00"), true).unwrap();
        assert_eq!(v.chart.outside_label, "51m");
        assert_eq!(v.chart.total_label, "outside of 1h 01m today");
        assert_eq!(v.chart.sources.len(), 2);
        assert_eq!(
            (
                v.chart.sources[0].label.as_str(),
                v.chart.sources[0].outside.as_str()
            ),
            ("TradingView", "41m")
        );
        assert_eq!(v.chart.segments.iter().filter(|x| x.inside).count(), 1);
        assert_eq!(v.chart.week.last().unwrap().minutes, 51);
        assert!(v.chart.week.last().unwrap().today);
        // Browser tracking off: the note explains apps only.
        assert!(!v.chart.browser_blocked);
        let on = TradingSettings {
            track_browsers: true,
            ..s
        };
        let v = view_in(&conn, &on, utc("2026-11-24 16:00"), true).unwrap();
        assert!(v.chart.browser_blocked);
    }

    #[test]
    fn helpers() {
        assert_eq!(zone_label("America/New_York"), "New York");
        assert_eq!(short_countdown(61_000), "2m");
        assert_eq!(mins_label(3_660_000), "1h 01m");
        assert_eq!(mins_label(3_600_000), "1h");
        assert_eq!(
            and_list(&["A".into(), "B".into(), "C".into()]),
            "A, B and C"
        );
        let w = [Span {
            market: Market::NewYork,
            start: 10,
            end: 20,
        }];
        assert_eq!(
            split(5, 25, &w),
            vec![(5, 10, false), (10, 20, true), (20, 25, false)]
        );
    }
}
