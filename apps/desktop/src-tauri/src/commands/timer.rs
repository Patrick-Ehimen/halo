use chrono::{Local, NaiveDateTime, TimeZone};
use tauri::AppHandle;

use crate::error::{Error, IpcError};
use crate::timer::{self, CheckInResponse, PresetId, SessionKind, TimerSettings, TimerSnapshot};

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

/// Starts a focus session on a task.
#[tauri::command]
#[specta::specta]
pub fn timer_start_task(
    app: AppHandle,
    task_id: String,
    preset: PresetId,
) -> Result<TimerSnapshot, IpcError> {
    Ok(timer::start_task(&app, &task_id, preset)?)
}

/// Saves a note on the running focus session.
#[tauri::command]
#[specta::specta]
pub fn timer_set_note(app: AppHandle, note: String) -> Result<TimerSnapshot, IpcError> {
    Ok(timer::set_note(&app, &note)?)
}

/// Answers "Still on it?". "I drifted" also counts a distraction.
#[tauri::command]
#[specta::specta]
pub fn check_in(app: AppHandle, response: CheckInResponse) -> Result<(), IpcError> {
    Ok(timer::record_check_in(&app, response)?)
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
