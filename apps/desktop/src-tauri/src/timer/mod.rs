//! Runs the `halo-timer` engine for the app: owns the live timer, drives it
//! from a tokio interval, saves finished sessions, and emits the events every
//! window renders from. Windows never compute time themselves.

mod dto;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use halo_timer::{Config, Millis, Phase, Preset, SessionSummary, Timer, Transition};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

pub use dto::{
    CheckInResponse, PresetId, SessionKind, TimerPhaseEvent, TimerSettings, TimerSnapshot,
    TimerTick,
};

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
    /// The running focus session's row, written when it started.
    session_id: Option<String>,
    /// Minutes of focus already saved to that row.
    saved_minutes: u64,
    /// When the open check-in was asked.
    check_in_at: Option<Millis>,
    /// The task the session was started from, and its project.
    task_id: Option<String>,
    project_id: Option<String>,
    /// A note on the running session.
    note: String,
    /// The last saved focus session and its intention, for the reflection card.
    last_session: Option<(String, String)>,
}

impl LiveTimer {
    pub fn new() -> Self {
        LiveTimer {
            engine: Timer::new(Config::default()),
            intention: String::new(),
            session_id: None,
            saved_minutes: 0,
            check_in_at: None,
            task_id: None,
            project_id: None,
            note: String::new(),
            last_session: None,
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
    let (transitions, snapshot, engine) = {
        let mut live = state.timer()?;
        let transitions = action(&mut live, now)?;
        persist(&state, &mut live, &transitions, now)?;
        (transitions, snap(&live, now), live.engine.snapshot(now))
    };
    announce(app, &transitions, &snapshot, &engine)?;
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

/// The engine's own snapshot, for Rust-side surfaces like the menu bar.
pub fn engine_snapshot(state: &AppState) -> Result<halo_timer::Snapshot> {
    Ok(state.timer()?.engine.snapshot(now_ms()))
}

pub fn snapshot(app: &AppHandle) -> Result<TimerSnapshot> {
    let state = app.state::<AppState>();
    let live = state.timer()?;
    Ok(snap(&live, now_ms()))
}

pub fn start(app: &AppHandle, preset: PresetId, intention: String) -> Result<TimerSnapshot> {
    act(app, |live, now| {
        let transitions = live.engine.start_focus(preset.preset(), now)?;
        live.intention = intention.trim().to_string();
        live.task_id = None;
        live.project_id = None;
        live.note.clear();
        Ok(transitions)
    })
}

/// Starts a focus session on a task: its title is the intention, and the
/// session counts toward the task (and its project's goal).
pub fn start_task(app: &AppHandle, task_id: &str, preset: PresetId) -> Result<TimerSnapshot> {
    let task = {
        let state = app.state::<AppState>();
        let conn = state.db()?;
        halo_store::tasks::get(&conn, task_id)?
            .ok_or(Error::Invalid("That task no longer exists"))?
    };
    act(app, |live, now| {
        let transitions = live.engine.start_focus(preset.preset(), now)?;
        live.intention = task.title.clone();
        live.task_id = Some(task.id.clone());
        live.project_id = task.project_id.clone();
        live.note.clear();
        Ok(transitions)
    })
}

/// Saves a note on the running focus session.
pub fn set_note(app: &AppHandle, note: &str) -> Result<TimerSnapshot> {
    let note = note.trim().chars().take(2000).collect::<String>();
    let state = app.state::<AppState>();
    act(app, |live, now| {
        if let Some(id) = live.session_id.clone() {
            let conn = state.db()?;
            halo_store::sessions::set_notes(&conn, &id, &note, ms_i64(now))?;
        }
        live.note = note.clone();
        Ok(Vec::new())
    })
}

fn snap(live: &LiveTimer, now: Millis) -> TimerSnapshot {
    let mut s = TimerSnapshot::new(&live.engine, &live.intention, now);
    if s.phase == dto::TimerPhase::Focus && s.status != dto::TimerStatus::Idle {
        s.task_id = live.task_id.clone();
        s.note = live.note.clone();
    }
    s
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
    let (transitions, snapshot, engine) = {
        let mut live = state.timer()?;
        let transitions = live.engine.tick(now);
        persist(&state, &mut live, &transitions, now)?;
        save_progress(&state, &mut live, now)?;
        (transitions, snap(&live, now), live.engine.snapshot(now))
    };
    // Emit when the shown second or the state changes.
    let shown = (snapshot.elapsed_ms / 1000, snapshot.key());
    if transitions.is_empty() && last_shown.as_ref() == Some(&shown) {
        return Ok(());
    }
    *last_shown = Some(shown);
    announce(app, &transitions, &snapshot, &engine)
}

/// Keeps the database in step with the engine: a row when focus starts, its
/// final numbers when it ends, and the time of any check-in asked.
fn persist(
    state: &AppState,
    live: &mut LiveTimer,
    transitions: &[Transition],
    now: Millis,
) -> Result<()> {
    for t in transitions {
        match t {
            Transition::PhaseStarted(Phase::Focus) => {
                let s = live.engine.snapshot(now);
                let row = halo_store::sessions::StartedSession {
                    kind: s.kind.as_str(),
                    planned_secs: secs(s.planned_ms),
                    started_at: ms_i64(s.started_at.unwrap_or(now)),
                    intention: &live.intention,
                    task_id: live.task_id.as_deref(),
                    project_id: live.project_id.as_deref(),
                };
                let conn = state.db()?;
                live.session_id = Some(halo_store::sessions::start(&conn, &row, ms_i64(now))?);
                live.saved_minutes = 0;
            }
            Transition::FocusEnded(summary) => {
                live.check_in_at = None;
                match live.session_id.take() {
                    Some(id) => {
                        live.last_session = finish_session(state, &id, summary, now)?
                            .then(|| (id, live.intention.clone()));
                    }
                    // Started before this run kept rows (shouldn't happen): save it whole.
                    None => save_session(state, summary, &live.intention, "timer")?,
                }
            }
            Transition::CheckInDue => live.check_in_at = Some(now),
            _ => {}
        }
    }
    Ok(())
}

/// Saves a running session's progress about once a minute, so a crash or a
/// forced quit loses at most a minute of it.
fn save_progress(state: &AppState, live: &mut LiveTimer, now: Millis) -> Result<()> {
    let Some(id) = live.session_id.clone() else {
        return Ok(());
    };
    let s = live.engine.snapshot(now);
    let minutes = s.elapsed_ms / halo_timer::MINUTE;
    if s.phase != Phase::Focus || minutes <= live.saved_minutes {
        return Ok(());
    }
    let conn = state.db()?;
    halo_store::sessions::update_progress(&conn, &id, &progress_of(&s), ms_i64(now))?;
    live.saved_minutes = minutes;
    Ok(())
}

fn progress_of(s: &halo_timer::Snapshot) -> halo_store::sessions::Progress {
    let flowtime = s.kind == halo_timer::Kind::Flowtime;
    halo_store::sessions::Progress {
        planned_secs: if flowtime { 0 } else { secs(s.planned_ms) },
        focused_secs: secs(s.elapsed_ms),
        overflow_secs: if flowtime {
            0
        } else {
            secs(s.elapsed_ms.saturating_sub(s.planned_ms))
        },
        pauses: s.pauses,
    }
}

/// Saves a finished session. Returns whether it was kept.
fn finish_session(state: &AppState, id: &str, s: &SessionSummary, now: Millis) -> Result<bool> {
    let conn = state.db()?;
    // Sessions under a minute are accidental starts; don't keep them.
    if s.focused_ms < halo_timer::MINUTE {
        halo_store::sessions::discard(&conn, id, ms_i64(now))?;
        return Ok(false);
    }
    let progress = halo_store::sessions::Progress {
        planned_secs: secs(s.planned_ms),
        focused_secs: secs(s.focused_ms),
        overflow_secs: secs(s.overflow_ms),
        pauses: s.pauses,
    };
    halo_store::sessions::finish(&conn, id, ms_i64(s.ended_at), &progress, ms_i64(now))?;
    Ok(true)
}

/// The last saved focus session and the break waiting after it.
pub struct LastSession {
    /// The session's id and intention.
    pub session: Option<(String, String)>,
    /// "Start 10 min break" when a break is waiting to start.
    pub pending_break: Option<String>,
}

pub fn last_session(state: &AppState) -> Result<LastSession> {
    let live = state.timer()?;
    let s = live.engine.snapshot(now_ms());
    let pending_break = (s.status == halo_timer::Status::Idle
        && matches!(s.phase, Phase::ShortBreak | Phase::LongBreak))
    .then(|| format!("Start {} min break", s.planned_ms / halo_timer::MINUTE));
    Ok(LastSession {
        session: live.last_session.clone(),
        pending_break,
    })
}

/// Saves the answer to the open check-in and closes the card.
pub fn record_check_in(app: &AppHandle, response: CheckInResponse) -> Result<()> {
    save_check_in(&app.state::<AppState>(), response, now_ms())?;
    crate::windows::hide_check_in(app)
}

fn save_check_in(state: &AppState, response: CheckInResponse, now: Millis) -> Result<()> {
    let mut live = state.timer()?;
    if let Some(id) = live.session_id.clone() {
        let at = live.check_in_at.take().unwrap_or(now);
        let conn = state.db()?;
        halo_store::check_ins::record(&conn, &id, ms_i64(at), response.into(), ms_i64(now))?;
    }
    Ok(())
}

/// Closes sessions a crash or forced quit left open. Called at startup.
pub fn close_abandoned(state: &AppState) -> Result<()> {
    let conn = state.db()?;
    halo_store::sessions::close_abandoned(&conn, ms_i64(now_ms()))?;
    Ok(())
}

fn save_session(state: &AppState, s: &SessionSummary, intention: &str, source: &str) -> Result<()> {
    // Sessions under a minute are accidental starts; don't keep them.
    if s.focused_ms < halo_timer::MINUTE {
        return Ok(());
    }
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

fn announce(
    app: &AppHandle,
    transitions: &[Transition],
    snapshot: &TimerSnapshot,
    engine: &halo_timer::Snapshot,
) -> Result<()> {
    for t in transitions {
        TimerPhaseEvent::from_transition(t).emit(app)?;
    }
    TimerTick(snapshot.clone()).emit(app)?;
    if let Err(e) = crate::tray::sync(app, engine) {
        eprintln!("menu bar: {e}");
    }
    if !transitions.is_empty() {
        crate::notify::on_transitions(app, transitions, engine.planned_ms);
        if let Err(e) = surfaces(app, transitions) {
            eprintln!("windows: {e}");
        }
    }
    Ok(())
}

/// Opens and closes the break overlay and the check-in card as phases change.
fn surfaces(app: &AppHandle, transitions: &[Transition]) -> Result<()> {
    for t in transitions {
        match t {
            Transition::PhaseStarted(Phase::ShortBreak | Phase::LongBreak) => {
                crate::overlay::open(app)?;
            }
            Transition::PhaseStarted(Phase::Focus) | Transition::BreakEnded { .. } => {
                crate::overlay::close(app)?;
            }
            Transition::CheckInDue => crate::windows::show_check_in(app)?,
            Transition::FocusEnded(summary) => {
                crate::windows::hide_check_in(app)?;
                if summary.focused_ms >= halo_timer::MINUTE && crate::prefs::get(app).reflect {
                    crate::windows::show_reflection(app)?;
                }
                // Task and goal counts include the session now.
                crate::plan::changed(app);
            }
            Transition::GoalReached => {}
        }
    }
    Ok(())
}

fn secs(ms: u64) -> u32 {
    u32::try_from(ms / 1000).unwrap_or(u32::MAX)
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
    /// The preset to restart from a session kind (custom falls back to Pomodoro).
    pub fn from_kind(kind: halo_timer::Kind) -> Self {
        match kind {
            halo_timer::Kind::Deep => PresetId::Deep,
            halo_timer::Kind::Flowtime => PresetId::Flowtime,
            halo_timer::Kind::Pomodoro | halo_timer::Kind::Custom => PresetId::Pomodoro,
        }
    }

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
            check_in_minutes: Some(20),
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

    #[test]
    fn settings_saved_before_check_ins_existed_still_load() {
        let state = state();
        save_raw(
            &state,
            r#"{"overflow":false,"autoStartBreaks":true,"autoStartFocus":false,"pauseLimit":2,"longBreakEvery":4}"#,
        );
        load_settings(&state).unwrap();
        let c = config(&state);
        assert!(!c.overflow);
        assert_eq!(c.check_in_every_ms, Some(15 * halo_timer::MINUTE));
    }

    /// Runs one engine step the way the app does, persisting what happened.
    fn step(state: &AppState, now: Millis, f: impl FnOnce(&mut LiveTimer) -> Vec<Transition>) {
        let mut live = state.timer().unwrap();
        let transitions = f(&mut live);
        persist(state, &mut live, &transitions, now).unwrap();
        save_progress(state, &mut live, now).unwrap();
    }

    fn query_u32(state: &AppState, sql: &str) -> u32 {
        state
            .db()
            .unwrap()
            .query_row(sql, [], |r| r.get(0))
            .unwrap()
    }

    fn query_i64(state: &AppState, sql: &str) -> i64 {
        state
            .db()
            .unwrap()
            .query_row(sql, [], |r| r.get(0))
            .unwrap()
    }

    fn query_text(state: &AppState, sql: &str) -> String {
        state
            .db()
            .unwrap()
            .query_row(sql, [], |r| r.get(0))
            .unwrap()
    }

    const T0: Millis = 1_700_000_000_000;
    const MIN: Millis = halo_timer::MINUTE;

    #[test]
    fn a_session_is_written_at_start_and_finished_at_the_end() {
        let state = state();
        step(&state, T0, |l| {
            l.intention = "Write tests".into();
            l.engine.start_focus(Preset::DEEP, T0).unwrap()
        });
        assert_eq!(
            query_u32(
                &state,
                "SELECT COUNT(*) FROM sessions WHERE ended_at IS NULL"
            ),
            1
        );

        step(&state, T0 + 3 * MIN, |l| l.engine.tick(T0 + 3 * MIN));
        assert_eq!(query_u32(&state, "SELECT focused_secs FROM sessions"), 180);

        step(&state, T0 + 10 * MIN, |l| {
            l.engine.end(T0 + 10 * MIN).unwrap()
        });
        assert_eq!(
            query_i64(&state, "SELECT ended_at FROM sessions"),
            (T0 + 10 * MIN) as i64
        );
        assert_eq!(query_u32(&state, "SELECT focused_secs FROM sessions"), 600);
        assert_eq!(
            query_text(&state, "SELECT intention FROM sessions"),
            "Write tests"
        );
    }

    #[test]
    fn an_accidental_start_leaves_no_row() {
        let state = state();
        step(&state, T0, |l| {
            l.engine.start_focus(Preset::DEEP, T0).unwrap()
        });
        step(&state, T0 + 20_000, |l| l.engine.end(T0 + 20_000).unwrap());
        assert_eq!(query_u32(&state, "SELECT COUNT(*) FROM sessions"), 0);
    }

    #[test]
    fn check_ins_are_saved_against_the_running_session() {
        let state = state();
        state.timer().unwrap().engine.set_config(Config {
            check_in_every_ms: Some(15 * MIN),
            ..Config::default()
        });
        step(&state, T0, |l| {
            l.engine.start_focus(Preset::DEEP, T0).unwrap()
        });
        step(&state, T0 + 15 * MIN, |l| l.engine.tick(T0 + 15 * MIN));
        save_check_in(&state, CheckInResponse::Drifted, T0 + 15 * MIN + 5_000).unwrap();

        assert_eq!(
            query_i64(
                &state,
                "SELECT at FROM check_ins WHERE response = 'drifted'"
            ),
            (T0 + 15 * MIN) as i64
        );
        assert_eq!(query_u32(&state, "SELECT distractions FROM sessions"), 1);
    }
}
