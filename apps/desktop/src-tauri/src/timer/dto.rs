//! Timer types sent over IPC. Durations are whole milliseconds as `u32`/`i32`
//! (about 24 days of range), so they map to plain TypeScript numbers.

use halo_timer::{Config, Kind, Millis, Phase, Status, Timer, Transition};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PresetId {
    Pomodoro,
    Deep,
    Flowtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TimerPhase {
    Focus,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TimerStatus {
    Idle,
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum SessionKind {
    Pomodoro,
    Deep,
    Flowtime,
    Custom,
}

/// Timer settings, as stored under the `timer` settings key and edited in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TimerSettings {
    pub overflow: bool,
    pub auto_start_breaks: bool,
    pub auto_start_focus: bool,
    /// `null` means unlimited.
    pub pause_limit: Option<u32>,
    pub long_break_every: u32,
}

impl From<Config> for TimerSettings {
    fn from(c: Config) -> Self {
        TimerSettings {
            overflow: c.overflow,
            auto_start_breaks: c.auto_start_breaks,
            auto_start_focus: c.auto_start_focus,
            pause_limit: c.pause_limit,
            long_break_every: c.long_break_every,
        }
    }
}

impl From<TimerSettings> for Config {
    fn from(s: TimerSettings) -> Self {
        Config {
            overflow: s.overflow,
            auto_start_breaks: s.auto_start_breaks,
            auto_start_focus: s.auto_start_focus,
            pause_limit: s.pause_limit,
            long_break_every: s.long_break_every,
        }
    }
}

/// Everything a window needs to draw the timer.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TimerSnapshot {
    pub phase: TimerPhase,
    pub status: TimerStatus,
    pub kind: SessionKind,
    /// Length of the phase; 0 for a Flowtime focus session.
    pub planned_ms: u32,
    pub elapsed_ms: u32,
    /// Negative while overflowing; `-elapsedMs` for Flowtime.
    pub remaining_ms: i32,
    pub overflowing: bool,
    /// Flowtime only: the break earned so far. 0 otherwise.
    pub earned_break_ms: u32,
    /// Ring fill in thousandths (0..=1000). Flowtime fills once an hour.
    pub progress_permille: u32,
    pub completed_in_cycle: u32,
    pub long_break_every: u32,
    /// `null` when unlimited or on a break.
    pub pauses_left: Option<u32>,
    pub overflow_enabled: bool,
    pub auto_start_breaks: bool,
    pub intention: String,
}

impl TimerSnapshot {
    pub fn new(engine: &Timer, intention: &str, now: Millis) -> Self {
        let s = engine.snapshot(now);
        let config = engine.config();
        let progress = if s.phase == Phase::Focus && s.kind == Kind::Flowtime {
            (s.elapsed_ms % HOUR) * 1000 / HOUR
        } else {
            (s.elapsed_ms.min(s.planned_ms) * 1000)
                .checked_div(s.planned_ms)
                .unwrap_or(0)
        };
        TimerSnapshot {
            phase: s.phase.into(),
            status: s.status.into(),
            kind: s.kind.into(),
            planned_ms: ms(s.planned_ms),
            elapsed_ms: ms(s.elapsed_ms),
            remaining_ms: i32::try_from(s.remaining_ms).unwrap_or(if s.remaining_ms < 0 {
                i32::MIN
            } else {
                i32::MAX
            }),
            overflowing: s.overflowing,
            earned_break_ms: if s.phase == Phase::Focus && s.kind == Kind::Flowtime {
                ms(halo_timer::flowtime_break_ms(s.elapsed_ms))
            } else {
                0
            },
            progress_permille: u32::try_from(progress).unwrap_or(1000),
            completed_in_cycle: s.completed_in_cycle,
            long_break_every: s.long_break_every,
            pauses_left: s.pauses_left,
            overflow_enabled: config.overflow,
            auto_start_breaks: config.auto_start_breaks,
            intention: if s.phase == Phase::Focus && s.status != Status::Idle {
                intention.to_string()
            } else {
                String::new()
            },
        }
    }

    /// Identifies the state apart from elapsed time, to decide when to emit.
    pub fn key(&self) -> String {
        format!(
            "{:?}{:?}{:?}{}{:?}",
            self.phase, self.status, self.kind, self.planned_ms, self.pauses_left
        )
    }
}

const HOUR: u64 = 60 * halo_timer::MINUTE;

fn ms(v: u64) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

/// Emitted whenever the shown second or the timer state changes.
#[derive(Debug, Clone, Serialize, Type, tauri_specta::Event)]
#[tauri_specta(event_name = "timer://tick")]
pub struct TimerTick(pub TimerSnapshot);

/// Emitted on every phase change, for notifications and sounds.
#[derive(Debug, Clone, Serialize, Type, tauri_specta::Event)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[tauri_specta(event_name = "timer://phase")]
pub enum TimerPhaseEvent {
    PhaseStarted { phase: TimerPhase },
    GoalReached,
    FocusEnded { completed: bool, focused_ms: u32 },
    BreakEnded { skipped: bool },
}

impl TimerPhaseEvent {
    pub fn from_transition(t: &Transition) -> Self {
        match *t {
            Transition::PhaseStarted(phase) => TimerPhaseEvent::PhaseStarted {
                phase: phase.into(),
            },
            Transition::GoalReached => TimerPhaseEvent::GoalReached,
            Transition::FocusEnded(s) => TimerPhaseEvent::FocusEnded {
                completed: s.completed,
                focused_ms: ms(s.focused_ms),
            },
            Transition::BreakEnded { skipped } => TimerPhaseEvent::BreakEnded { skipped },
        }
    }
}

impl From<Phase> for TimerPhase {
    fn from(p: Phase) -> Self {
        match p {
            Phase::Focus => TimerPhase::Focus,
            Phase::ShortBreak => TimerPhase::ShortBreak,
            Phase::LongBreak => TimerPhase::LongBreak,
        }
    }
}

impl From<Status> for TimerStatus {
    fn from(s: Status) -> Self {
        match s {
            Status::Idle => TimerStatus::Idle,
            Status::Running => TimerStatus::Running,
            Status::Paused => TimerStatus::Paused,
        }
    }
}

impl From<SessionKind> for Kind {
    fn from(k: SessionKind) -> Self {
        match k {
            SessionKind::Pomodoro => Kind::Pomodoro,
            SessionKind::Deep => Kind::Deep,
            SessionKind::Flowtime => Kind::Flowtime,
            SessionKind::Custom => Kind::Custom,
        }
    }
}

impl From<Kind> for SessionKind {
    fn from(k: Kind) -> Self {
        match k {
            Kind::Pomodoro => SessionKind::Pomodoro,
            Kind::Deep => SessionKind::Deep,
            Kind::Flowtime => SessionKind::Flowtime,
            Kind::Custom => SessionKind::Custom,
        }
    }
}
