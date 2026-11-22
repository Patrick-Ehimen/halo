use chrono::{Days, Local, NaiveDateTime, NaiveTime, TimeZone};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};

use crate::error::{Error, IpcError};
use crate::state::AppState;
use crate::timer::{self, PresetId, SessionKind, TimerSettings, TimerSnapshot};

#[tauri::command]
#[specta::specta]
pub fn timer_state(app: AppHandle) -> Result<TimerSnapshot, IpcError> {
    Ok(timer::snapshot(&app)?)
}

/// Starts a focus session with an intention. Skips a pending break.
#[tauri::command]
#[specta::specta]
pub fn timer_start(
    app: AppHandle,
    preset: PresetId,
    intention: String,
) -> Result<TimerSnapshot, IpcError> {
    Ok(timer::start(&app, preset, intention)?)
}

#[tauri::command]
#[specta::specta]
pub fn timer_select_preset(app: AppHandle, preset: PresetId) -> Result<TimerSnapshot, IpcError> {
    Ok(timer::select_preset(&app, preset)?)
}

/// Starts the waiting phase: a pending break, or focus with the current preset.
#[tauri::command]
#[specta::specta]
pub fn timer_start_pending(app: AppHandle) -> Result<TimerSnapshot, IpcError> {
    Ok(timer::start_pending(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn timer_pause(app: AppHandle) -> Result<TimerSnapshot, IpcError> {
    Ok(timer::pause(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn timer_resume(app: AppHandle) -> Result<TimerSnapshot, IpcError> {
    Ok(timer::resume(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn timer_extend(app: AppHandle, minutes: u32) -> Result<TimerSnapshot, IpcError> {
    Ok(timer::extend(&app, minutes)?)
}

#[tauri::command]
#[specta::specta]
pub fn timer_end(app: AppHandle) -> Result<TimerSnapshot, IpcError> {
    Ok(timer::end(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn timer_settings(app: AppHandle) -> Result<TimerSettings, IpcError> {
    Ok(timer::settings(&app)?)
}

/// Saves timer settings. The running phase keeps its length; the rest applies now.
#[tauri::command]
#[specta::specta]
pub fn timer_set_settings(
    app: AppHandle,
    settings: TimerSettings,
) -> Result<TimerSnapshot, IpcError> {
    Ok(timer::set_settings(&app, settings)?)
}

/// Logs a session done away from the timer. `started` is the local date and
/// time from a `datetime-local` input, e.g. `2026-11-22T09:30`.
#[tauri::command]
#[specta::specta]
pub fn log_session(
    app: AppHandle,
    kind: SessionKind,
    started: String,
    minutes: u32,
    intention: String,
) -> Result<(), IpcError> {
    let started_at = parse_local(&started).ok_or(Error::Timer(halo_timer::Error::Invalid(
        "Pick when the session started",
    )))?;
    Ok(timer::log_manual(
        &app, kind, started_at, minutes, &intention,
    )?)
}

/// Local `YYYY-MM-DDTHH:MM` to unix milliseconds.
fn parse_local(value: &str) -> Option<u64> {
    let naive = NaiveDateTime::parse_from_str(value.trim(), "%Y-%m-%dT%H:%M").ok()?;
    let local = Local.from_local_datetime(&naive).earliest()?;
    u64::try_from(local.timestamp_millis()).ok()
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TodayFocus {
    pub focused_secs: u32,
    pub sessions: u32,
}

/// Focus saved today, in the user's local day.
#[tauri::command]
#[specta::specta]
pub fn today_focus(state: State<'_, AppState>) -> Result<TodayFocus, IpcError> {
    let today = Local::now().date_naive();
    let start_of = |date: chrono::NaiveDate| {
        Local
            .from_local_datetime(&date.and_time(NaiveTime::MIN))
            .earliest()
            .map_or(0, |t| t.timestamp_millis())
    };
    let from = start_of(today);
    let to = today
        .checked_add_days(Days::new(1))
        .map_or(i64::MAX, start_of);
    let conn = state.db()?;
    let (focused_secs, sessions) =
        halo_store::sessions::totals(&conn, from, to).map_err(Error::from)?;
    Ok(TodayFocus {
        focused_secs,
        sessions,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_local;

    #[test]
    fn parses_datetime_local_input() {
        assert!(parse_local("2026-11-22T09:30").is_some());
        assert!(parse_local(" 2026-11-22T09:30 ").is_some());
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(parse_local(""), None);
        assert_eq!(parse_local("yesterday"), None);
        assert_eq!(parse_local("2026-13-01T09:30"), None);
    }
}
