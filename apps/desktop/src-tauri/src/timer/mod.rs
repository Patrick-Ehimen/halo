//! Runs the `halo-timer` engine for the app: owns the live timer, drives it
//! from a tokio interval, saves finished sessions, and emits the events every
//! window renders from. Windows never compute time themselves.

mod dto;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use halo_timer::{Config, Millis, Preset, SessionSummary, Timer, Transition};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

pub use dto::{PresetId, SessionKind, TimerPhaseEvent, TimerSettings, TimerSnapshot, TimerTick};

use crate::error::{Error, Result};
use crate::state::AppState;

/// How often the engine is advanced. Ticks are only emitted when the shown
/// second (or the state) changes, so this stays cheap.
const DRIVE_EVERY: Duration = Duration::from_millis(200);

/// Settings key for [`TimerSettings`].
const SETTINGS_KEY: &str = "timer";

/// The live timer plus what the engine doesn't track (the intention).
pub struct LiveTimer {
    engine: Timer,
    intention: String,
}

impl LiveTimer {
    pub fn new() -> Self {
        LiveTimer {
            engine: Timer::new(Config::default()),
            intention: String::new(),
        }
    }
}

pub fn now_ms() -> Millis {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// Applies an action to the live timer, saves and announces what happened,
/// and returns the new snapshot (also emitted as a tick).
pub fn act<F>(app: &AppHandle, action: F) -> Result<TimerSnapshot>
where
    F: FnOnce(&mut LiveTimer, Millis) -> Result<Vec<Transition>>,
{
    let state = app.state::<AppState>();
    let now = now_ms();
    let (transitions, snapshot) = {
        let mut live = state.timer()?;
        let transitions = action(&mut live, now)?;
        save_finished(&state, &live, &transitions)?;
        (
            transitions,
            TimerSnapshot::new(&live.engine, &live.intention, now),
        )
    };
    announce(app, &transitions, &snapshot)?;
    Ok(snapshot)
}

/// Applies saved settings at startup. Bad or missing settings keep the defaults.
pub fn load_settings(state: &AppState) -> Result<()> {
    let saved = {
        let conn = state.db()?;
        halo_store::settings::get(&conn, SETTINGS_KEY)?
    };
    let Some(json) = saved else { return Ok(()) };
    let config: Config = match serde_json::from_str::<TimerSettings>(&json) {
        Ok(s) => s.into(),
        Err(e) => {
            eprintln!("timer settings ignored: {e}");
            return Ok(());
        }
    };
    if let Err(e) = config.validate() {
        eprintln!("timer settings ignored: {e}");
        return Ok(());
    }
    state.timer()?.engine.set_config(config);
    Ok(())
}

pub fn settings(app: &AppHandle) -> Result<TimerSettings> {
    let state = app.state::<AppState>();
    let live = state.timer()?;
    Ok(live.engine.config().into())
}

/// Validates, saves and applies new settings. The running phase keeps its length.
pub fn set_settings(app: &AppHandle, settings: TimerSettings) -> Result<TimerSnapshot> {
    let config: Config = settings.into();
    config.validate()?;
    let json = serde_json::to_string(&settings)?;
    let state = app.state::<AppState>();
    act(app, |live, now| {
        {
            let conn = state.db()?;
            halo_store::settings::set(&conn, SETTINGS_KEY, &json, ms_i64(now))?;
        }
        live.engine.set_config(config);
        Ok(Vec::new())
    })
}

/// Saves a session that happened away from the timer.
pub fn log_manual(
    app: &AppHandle,
    kind: SessionKind,
    started_at: Millis,
    minutes: u32,
    intention: &str,
) -> Result<()> {
    let state = app.state::<AppState>();
    log_manual_at(&state, kind, started_at, minutes, intention, now_ms())
}

fn log_manual_at(
    state: &AppState,
    kind: SessionKind,
    started_at: Millis,
    minutes: u32,
    intention: &str,
    now: Millis,
) -> Result<()> {
    let summary = halo_timer::manual_session(kind.into(), started_at, minutes, now)?;
    save_session(state, &summary, intention.trim(), "manual")
}

pub fn snapshot(app: &AppHandle) -> Result<TimerSnapshot> {
    let state = app.state::<AppState>();
    let live = state.timer()?;
    Ok(TimerSnapshot::new(&live.engine, &live.intention, now_ms()))
}

pub fn start(app: &AppHandle, preset: PresetId, intention: String) -> Result<TimerSnapshot> {
    act(app, |live, now| {
        let transitions = live.engine.start_focus(preset.preset(), now)?;
        live.intention = intention.trim().to_string();
        Ok(transitions)
    })
}

pub fn select_preset(app: &AppHandle, preset: PresetId) -> Result<TimerSnapshot> {
    act(app, |live, _| {
        live.engine.select_preset(preset.preset())?;
        Ok(Vec::new())
    })
}

/// Starts whatever is waiting: a pending break, or focus with the current preset.
pub fn start_pending(app: &AppHandle) -> Result<TimerSnapshot> {
    act(app, |live, now| Ok(live.engine.start(now)?))
}

pub fn pause(app: &AppHandle) -> Result<TimerSnapshot> {
    act(app, |live, now| {
        live.engine.pause(now)?;
        Ok(Vec::new())
    })
}

pub fn resume(app: &AppHandle) -> Result<TimerSnapshot> {
    act(app, |live, now| {
        live.engine.resume(now)?;
        Ok(Vec::new())
    })
}

pub fn extend(app: &AppHandle, minutes: u32) -> Result<TimerSnapshot> {
    act(app, |live, now| {
        live.engine
            .extend(u64::from(minutes) * halo_timer::MINUTE, now)?;
        Ok(Vec::new())
    })
}

pub fn end(app: &AppHandle) -> Result<TimerSnapshot> {
    act(app, |live, now| Ok(live.engine.end(now)?))
}

/// Advances the engine forever. Spawned once at startup.
pub fn spawn_driver(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(DRIVE_EVERY);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut last_shown = None;
        loop {
            interval.tick().await;
            if let Err(e) = drive_once(&app, &mut last_shown) {
                eprintln!("timer: {e}");
            }
        }
    });
}

fn drive_once(app: &AppHandle, last_shown: &mut Option<(u32, String)>) -> Result<()> {
    let state = app.state::<AppState>();
    let now = now_ms();
    let (transitions, snapshot) = {
        let mut live = state.timer()?;
        let transitions = live.engine.tick(now);
        save_finished(&state, &live, &transitions)?;
        (
            transitions,
            TimerSnapshot::new(&live.engine, &live.intention, now),
        )
    };
    // Emit when the shown second or the state changes.
    let shown = (snapshot.elapsed_ms / 1000, snapshot.key());
    if transitions.is_empty() && last_shown.as_ref() == Some(&shown) {
        return Ok(());
    }
    *last_shown = Some(shown);
    announce(app, &transitions, &snapshot)
}

fn save_finished(state: &AppState, live: &LiveTimer, transitions: &[Transition]) -> Result<()> {
    for t in transitions {
        if let Transition::FocusEnded(summary) = t {
            save_session(state, summary, &live.intention, "timer")?;
        }
    }
    Ok(())
}

fn save_session(state: &AppState, s: &SessionSummary, intention: &str, source: &str) -> Result<()> {
    // Sessions under a minute are accidental starts; don't keep them.
    if s.focused_ms < halo_timer::MINUTE {
        return Ok(());
    }
    let secs = |ms: u64| u32::try_from(ms / 1000).unwrap_or(u32::MAX);
    let ms = ms_i64;
    let row = halo_store::sessions::NewSession {
        kind: s.kind.as_str(),
        source,
        planned_secs: secs(s.planned_ms),
        started_at: ms(s.started_at),
        ended_at: ms(s.ended_at),
        focused_secs: secs(s.focused_ms),
        overflow_secs: secs(s.overflow_ms),
        pauses: s.pauses,
        intention,
    };
    let conn = state.db()?;
    halo_store::sessions::insert(&conn, &row, ms(now_ms()))?;
    Ok(())
}

fn announce(app: &AppHandle, transitions: &[Transition], snapshot: &TimerSnapshot) -> Result<()> {
    for t in transitions {
        TimerPhaseEvent::from_transition(t).emit(app)?;
    }
    TimerTick(snapshot.clone()).emit(app)?;
    Ok(())
}

fn ms_i64(t: Millis) -> i64 {
    i64::try_from(t).unwrap_or(i64::MAX)
}

impl From<halo_timer::Error> for Error {
    fn from(e: halo_timer::Error) -> Self {
        Error::Timer(e)
    }
}

impl PresetId {
    fn preset(self) -> Preset {
        match self {
            PresetId::Pomodoro => Preset::POMODORO,
            PresetId::Deep => Preset::DEEP,
            PresetId::Flowtime => Preset::FLOWTIME,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn state() -> AppState {
        AppState::new(halo_store::open(Path::new(":memory:")).unwrap())
    }

    fn save_raw(state: &AppState, json: &str) {
        let conn = state.db().unwrap();
        halo_store::settings::set(&conn, SETTINGS_KEY, json, 1).unwrap();
    }

    fn config(state: &AppState) -> Config {
        state.timer().unwrap().engine.config()
    }

    #[test]
    fn startup_applies_saved_settings() {
        let state = state();
        let saved = TimerSettings {
            overflow: false,
            auto_start_breaks: false,
            auto_start_focus: true,
            pause_limit: None,
            long_break_every: 3,
        };
        save_raw(&state, &serde_json::to_string(&saved).unwrap());
        load_settings(&state).unwrap();
        assert_eq!(config(&state), Config::from(saved));
    }

    #[test]
    fn startup_keeps_defaults_without_settings() {
        let state = state();
        load_settings(&state).unwrap();
        assert_eq!(config(&state), Config::default());
    }

    #[test]
    fn startup_ignores_corrupt_or_out_of_range_settings() {
        let state = state();
        save_raw(&state, "not json");
        load_settings(&state).unwrap();
        assert_eq!(config(&state), Config::default());

        save_raw(
            &state,
            r#"{"overflow":true,"autoStartBreaks":true,"autoStartFocus":false,"pauseLimit":9,"longBreakEvery":4}"#,
        );
        load_settings(&state).unwrap();
        assert_eq!(config(&state), Config::default());
    }

    #[test]
    fn manual_log_saves_a_manual_session() {
        let state = state();
        let now = 1_700_000_000_000;
        log_manual_at(
            &state,
            SessionKind::Deep,
            now - 90 * halo_timer::MINUTE,
            50,
            "  Review PR  ",
            now,
        )
        .unwrap();
        let conn = state.db().unwrap();
        let (source, focused, intention): (String, u32, String) = conn
            .query_row(
                "SELECT source, focused_secs, intention FROM sessions",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            (source.as_str(), focused, intention.as_str()),
            ("manual", 3000, "Review PR")
        );
    }

    #[test]
    fn manual_log_refuses_a_future_session() {
        let state = state();
        let now = 1_700_000_000_000;
        let err = log_manual_at(&state, SessionKind::Deep, now, 50, "", now).unwrap_err();
        assert_eq!(err.to_string(), "That session hasn't ended yet");
    }
}
