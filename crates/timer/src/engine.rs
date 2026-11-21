use std::fmt;

use crate::config::{Config, Kind, Preset, flowtime_break_ms};

/// Unix time in milliseconds. The engine never reads a clock itself: every
/// call takes `now`, so the caller (or a test) controls time.
pub type Millis = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn is_break(self) -> bool {
        matches!(self, Phase::ShortBreak | Phase::LongBreak)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Waiting to start the current phase.
    Idle,
    Running,
    Paused,
}

/// A finished focus session, ready to be saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionSummary {
    pub kind: Kind,
    pub started_at: Millis,
    pub ended_at: Millis,
    /// The goal, including any +5 min. 0 for Flowtime.
    pub planned_ms: u64,
    /// Time actually focused, pauses excluded, overflow included.
    pub focused_ms: u64,
    /// Time focused past the goal. Always 0 for Flowtime.
    pub overflow_ms: u64,
    pub pauses: u32,
    /// Reached the goal (always true for Flowtime). Only completed sessions
    /// count as a round and earn a break.
    pub completed: bool,
}

/// Something that happened, for the caller to save and announce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    PhaseStarted(Phase),
    /// Overflow is on and the focus session just passed its goal. It keeps counting.
    GoalReached,
    FocusEnded(SessionSummary),
    BreakEnded {
        skipped: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// A phase is already running or paused.
    AlreadyActive,
    /// Nothing is running or paused.
    NotActive,
    NotRunning,
    NotPaused,
    PauseLimitReached,
    /// Only a focus session can be extended.
    NotFocus,
    /// Flowtime has no goal to extend.
    NoGoal,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Error::AlreadyActive => "a session is already in progress",
            Error::NotActive => "no session is in progress",
            Error::NotRunning => "the timer isn't running",
            Error::NotPaused => "the timer isn't paused",
            Error::PauseLimitReached => "no pauses left this session",
            Error::NotFocus => "only a focus session can be extended",
            Error::NoGoal => "Flowtime has no goal to extend",
        })
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// Everything a window needs to draw the timer at one moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    pub phase: Phase,
    pub status: Status,
    pub kind: Kind,
    /// Length of the current phase. 0 for a Flowtime focus session.
    pub planned_ms: u64,
    pub elapsed_ms: u64,
    /// Time left; negative while overflowing. Equals `-elapsed_ms` for Flowtime.
    pub remaining_ms: i64,
    pub overflowing: bool,
    /// Completed focus sessions in the current long-break cycle.
    pub completed_in_cycle: u32,
    pub long_break_every: u32,
    /// Pauses left in this focus session; `None` when unlimited or on a break.
    pub pauses_left: Option<u32>,
    /// When the current phase started; `None` while idle.
    pub started_at: Option<Millis>,
}

/// The timer state machine.
///
/// Focus → (completed) → short or long break → focus, with optional
/// auto-start between phases. Ending a focus session before its goal saves it
/// as not completed and earns no break.
#[derive(Debug, Clone)]
pub struct Timer {
    config: Config,
    preset: Preset,
    phase: Phase,
    status: Status,
    completed_rounds: u32,
    planned_ms: u64,
    /// Elapsed time from earlier running stretches of this phase.
    banked_ms: u64,
    /// Start of the current running stretch.
    run_started_at: Option<Millis>,
    /// Start of the current phase.
    started_at: Option<Millis>,
    pauses: u32,
    goal_announced: bool,
}

impl Timer {
    pub fn new(config: Config) -> Self {
        Timer {
            config,
            preset: Preset::POMODORO,
            phase: Phase::Focus,
            status: Status::Idle,
            completed_rounds: 0,
            planned_ms: Preset::POMODORO.focus_ms,
            banked_ms: 0,
            run_started_at: None,
            started_at: None,
            pauses: 0,
            goal_announced: false,
        }
    }

    pub fn config(&self) -> Config {
        self.config
    }

    /// Takes effect from the next phase; the current one keeps its length.
    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    pub fn preset(&self) -> Preset {
        self.preset
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn status(&self) -> Status {
        self.status
    }

    /// Picks the preset for the next focus session. Only while idle on focus.
    pub fn select_preset(&mut self, preset: Preset) -> Result<()> {
        if self.status != Status::Idle {
            return Err(Error::AlreadyActive);
        }
        self.preset = preset;
        if self.phase == Phase::Focus {
            self.planned_ms = preset.focus_ms;
        }
        Ok(())
    }

    /// Starts a focus session with `preset`. A pending break is skipped.
    pub fn start_focus(&mut self, preset: Preset, now: Millis) -> Result<Vec<Transition>> {
        if self.status != Status::Idle {
            return Err(Error::AlreadyActive);
        }
        self.preset = preset;
        self.begin(Phase::Focus, preset.focus_ms, now);
        Ok(vec![Transition::PhaseStarted(Phase::Focus)])
    }

    /// Starts whatever phase is waiting: the pending break, or a focus
    /// session with the current preset.
    pub fn start(&mut self, now: Millis) -> Result<Vec<Transition>> {
        if self.status != Status::Idle {
            return Err(Error::AlreadyActive);
        }
        let planned = match self.phase {
            Phase::Focus => self.preset.focus_ms,
            Phase::ShortBreak | Phase::LongBreak => self.planned_ms,
        };
        self.begin(self.phase, planned, now);
        Ok(vec![Transition::PhaseStarted(self.phase)])
    }

    pub fn pause(&mut self, now: Millis) -> Result<()> {
        if self.status != Status::Running {
            return Err(Error::NotRunning);
        }
        if self.phase == Phase::Focus
            && let Some(limit) = self.config.pause_limit
            && self.pauses >= limit
        {
            return Err(Error::PauseLimitReached);
        }
        self.banked_ms = self.elapsed(now);
        self.run_started_at = None;
        self.status = Status::Paused;
        if self.phase == Phase::Focus {
            self.pauses += 1;
        }
        Ok(())
    }

    pub fn resume(&mut self, now: Millis) -> Result<()> {
        if self.status != Status::Paused {
            return Err(Error::NotPaused);
        }
        self.run_started_at = Some(now);
        self.status = Status::Running;
        Ok(())
    }

    /// Moves the goal of the current focus session (e.g. +5 min).
    pub fn extend(&mut self, by_ms: u64, now: Millis) -> Result<()> {
        if self.status == Status::Idle {
            return Err(Error::NotActive);
        }
        if self.phase != Phase::Focus {
            return Err(Error::NotFocus);
        }
        if self.preset.is_flowtime() {
            return Err(Error::NoGoal);
        }
        self.planned_ms += by_ms;
        if self.elapsed(now) < self.planned_ms {
            self.goal_announced = false;
        }
        Ok(())
    }

    /// Ends the current phase now. A focus session is saved (completed if it
    /// reached its goal); a break is skipped.
    pub fn end(&mut self, now: Millis) -> Result<Vec<Transition>> {
        if self.status == Status::Idle {
            return Err(Error::NotActive);
        }
        let elapsed = self.elapsed(now);
        Ok(match self.phase {
            Phase::Focus => self.finish_focus(now, elapsed),
            Phase::ShortBreak | Phase::LongBreak => {
                self.go_idle(Phase::Focus, self.preset.focus_ms);
                vec![Transition::BreakEnded { skipped: true }]
            }
        })
    }

    /// Advances the timer to `now`. Call it about once a second.
    pub fn tick(&mut self, now: Millis) -> Vec<Transition> {
        if self.status != Status::Running {
            return Vec::new();
        }
        let elapsed = self.elapsed(now);
        if self.phase == Phase::Focus && self.preset.is_flowtime() {
            return Vec::new();
        }
        if elapsed < self.planned_ms {
            return Vec::new();
        }
        // The moment the goal was reached, so late ticks don't stretch the phase.
        let goal_at = now.saturating_sub(elapsed - self.planned_ms);
        match self.phase {
            Phase::Focus if self.config.overflow => {
                if self.goal_announced {
                    Vec::new()
                } else {
                    self.goal_announced = true;
                    vec![Transition::GoalReached]
                }
            }
            Phase::Focus => self.finish_focus(goal_at, self.planned_ms),
            Phase::ShortBreak | Phase::LongBreak => {
                let mut out = vec![Transition::BreakEnded { skipped: false }];
                if self.config.auto_start_focus {
                    self.begin(Phase::Focus, self.preset.focus_ms, goal_at);
                    out.push(Transition::PhaseStarted(Phase::Focus));
                } else {
                    self.go_idle(Phase::Focus, self.preset.focus_ms);
                }
                out
            }
        }
    }

    pub fn snapshot(&self, now: Millis) -> Snapshot {
        let elapsed = self.elapsed(now);
        let flow_focus = self.phase == Phase::Focus && self.preset.is_flowtime();
        let remaining = if flow_focus {
            -to_i64(elapsed)
        } else {
            to_i64(self.planned_ms) - to_i64(elapsed)
        };
        let every = self.config.long_break_every.max(1);
        let completed_in_cycle = match self.phase {
            Phase::LongBreak => every,
            _ => self.completed_rounds % every,
        };
        let pauses_left = match (self.phase, self.config.pause_limit) {
            (Phase::Focus, Some(limit)) => Some(limit.saturating_sub(self.pauses)),
            _ => None,
        };
        Snapshot {
            phase: self.phase,
            status: self.status,
            kind: self.preset.kind,
            planned_ms: self.planned_ms,
            elapsed_ms: elapsed,
            remaining_ms: remaining,
            overflowing: !flow_focus && self.phase == Phase::Focus && elapsed > self.planned_ms,
            completed_in_cycle,
            long_break_every: every,
            pauses_left,
            started_at: self.started_at,
        }
    }

    fn elapsed(&self, now: Millis) -> u64 {
        let running = self
            .run_started_at
            .map_or(0, |start| now.saturating_sub(start));
        self.banked_ms + running
    }

    fn begin(&mut self, phase: Phase, planned_ms: u64, at: Millis) {
        self.phase = phase;
        self.status = Status::Running;
        self.planned_ms = planned_ms;
        self.banked_ms = 0;
        self.run_started_at = Some(at);
        self.started_at = Some(at);
        self.pauses = 0;
        self.goal_announced = false;
    }

    fn go_idle(&mut self, phase: Phase, planned_ms: u64) {
        self.phase = phase;
        self.status = Status::Idle;
        self.planned_ms = planned_ms;
        self.banked_ms = 0;
        self.run_started_at = None;
        self.started_at = None;
        self.pauses = 0;
        self.goal_announced = false;
    }

    fn finish_focus(&mut self, ended_at: Millis, focused_ms: u64) -> Vec<Transition> {
        let flowtime = self.preset.is_flowtime();
        let completed = flowtime || focused_ms >= self.planned_ms;
        let summary = SessionSummary {
            kind: self.preset.kind,
            started_at: self.started_at.unwrap_or(ended_at),
            ended_at,
            planned_ms: if flowtime { 0 } else { self.planned_ms },
            focused_ms,
            overflow_ms: if flowtime {
                0
            } else {
                focused_ms.saturating_sub(self.planned_ms)
            },
            pauses: self.pauses,
            completed,
        };
        let mut out = vec![Transition::FocusEnded(summary)];
        if !completed {
            self.go_idle(Phase::Focus, self.preset.focus_ms);
            return out;
        }
        self.completed_rounds += 1;
        let long = self
            .completed_rounds
            .is_multiple_of(self.config.long_break_every.max(1));
        let phase = if long {
            Phase::LongBreak
        } else {
            Phase::ShortBreak
        };
        let break_ms = if flowtime {
            flowtime_break_ms(focused_ms)
        } else if long {
            self.preset.long_break_ms
        } else {
            self.preset.short_break_ms
        };
        if self.config.auto_start_breaks {
            self.begin(phase, break_ms, ended_at);
            out.push(Transition::PhaseStarted(phase));
        } else {
            self.go_idle(phase, break_ms);
        }
        out
    }
}

fn to_i64(ms: u64) -> i64 {
    i64::try_from(ms).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{MINUTE, SECOND};

    const T0: Millis = 1_700_000_000_000;

    fn timer(config: Config) -> Timer {
        Timer::new(config)
    }

    fn no_auto() -> Config {
        Config {
            auto_start_breaks: false,
            auto_start_focus: false,
            ..Config::default()
        }
    }

    fn summary(ts: &[Transition]) -> SessionSummary {
        ts.iter()
            .find_map(|t| match t {
                Transition::FocusEnded(s) => Some(*s),
                _ => None,
            })
            .expect("a FocusEnded transition")
    }

    #[test]
    fn starts_idle_on_focus() {
        let t = timer(Config::default());
        let s = t.snapshot(T0);
        assert_eq!(s.status, Status::Idle);
        assert_eq!(s.phase, Phase::Focus);
        assert_eq!(s.planned_ms, 25 * MINUTE);
        assert_eq!(s.started_at, None);
    }

    #[test]
    fn counts_down_while_running() {
        let mut t = timer(Config::default());
        assert_eq!(
            t.start_focus(Preset::DEEP, T0).unwrap(),
            vec![Transition::PhaseStarted(Phase::Focus)]
        );
        let s = t.snapshot(T0 + 10 * MINUTE);
        assert_eq!(s.elapsed_ms, 10 * MINUTE);
        assert_eq!(s.remaining_ms, to_i64(40 * MINUTE));
        assert_eq!(s.kind, Kind::Deep);
        assert!(!s.overflowing);
    }

    #[test]
    fn cannot_start_twice() {
        let mut t = timer(Config::default());
        t.start_focus(Preset::POMODORO, T0).unwrap();
        assert_eq!(t.start_focus(Preset::DEEP, T0), Err(Error::AlreadyActive));
        assert_eq!(t.start(T0), Err(Error::AlreadyActive));
        assert_eq!(t.select_preset(Preset::DEEP), Err(Error::AlreadyActive));
    }

    #[test]
    fn pause_freezes_time_and_resume_continues() {
        let mut t = timer(Config::default());
        t.start_focus(Preset::POMODORO, T0).unwrap();
        t.pause(T0 + 5 * MINUTE).unwrap();
        assert_eq!(t.snapshot(T0 + 30 * MINUTE).elapsed_ms, 5 * MINUTE);
        assert_eq!(t.status(), Status::Paused);
        assert!(t.tick(T0 + 60 * MINUTE).is_empty());
        t.resume(T0 + 30 * MINUTE).unwrap();
        assert_eq!(t.snapshot(T0 + 31 * MINUTE).elapsed_ms, 6 * MINUTE);
    }

    #[test]
    fn pause_and_resume_need_the_right_state() {
        let mut t = timer(Config::default());
        assert_eq!(t.pause(T0), Err(Error::NotRunning));
        assert_eq!(t.resume(T0), Err(Error::NotPaused));
        t.start_focus(Preset::POMODORO, T0).unwrap();
        assert_eq!(t.resume(T0), Err(Error::NotPaused));
        t.pause(T0).unwrap();
        assert_eq!(t.pause(T0), Err(Error::NotRunning));
    }

    #[test]
    fn pause_limit_is_per_focus_session() {
        let mut t = timer(Config {
            pause_limit: Some(1),
            ..no_auto()
        });
        t.start_focus(Preset::POMODORO, T0).unwrap();
        assert_eq!(t.snapshot(T0).pauses_left, Some(1));
        t.pause(T0 + MINUTE).unwrap();
        t.resume(T0 + 2 * MINUTE).unwrap();
        assert_eq!(t.snapshot(T0 + 2 * MINUTE).pauses_left, Some(0));
        assert_eq!(t.pause(T0 + 3 * MINUTE), Err(Error::PauseLimitReached));

        // A new session gets its pauses back.
        t.end(T0 + 4 * MINUTE).unwrap();
        t.start_focus(Preset::POMODORO, T0 + 5 * MINUTE).unwrap();
        assert_eq!(t.snapshot(T0 + 5 * MINUTE).pauses_left, Some(1));
        t.pause(T0 + 6 * MINUTE).unwrap();
    }

    #[test]
    fn unlimited_pauses() {
        let mut t = timer(Config {
            pause_limit: None,
            ..Config::default()
        });
        t.start_focus(Preset::POMODORO, T0).unwrap();
        for i in 0..10 {
            t.pause(T0 + i * SECOND).unwrap();
            t.resume(T0 + i * SECOND).unwrap();
        }
        assert_eq!(t.snapshot(T0).pauses_left, None);
    }

    #[test]
    fn breaks_dont_use_pauses() {
        let mut t = timer(Config {
            pause_limit: Some(0),
            ..Config::default()
        });
        t.start_focus(Preset::POMODORO, T0).unwrap();
        t.end(T0 + 25 * MINUTE).unwrap(); // completed, break auto-starts
        assert_eq!(t.phase(), Phase::ShortBreak);
        t.pause(T0 + 26 * MINUTE).unwrap();
        assert_eq!(t.snapshot(T0 + 26 * MINUTE).pauses_left, None);
    }

    #[test]
    fn overflow_keeps_counting_and_announces_the_goal_once() {
        let mut t = timer(Config::default());
        t.start_focus(Preset::POMODORO, T0).unwrap();
        assert!(t.tick(T0 + 24 * MINUTE).is_empty());
        assert_eq!(t.tick(T0 + 25 * MINUTE), vec![Transition::GoalReached]);
        assert!(t.tick(T0 + 26 * MINUTE).is_empty());
        let s = t.snapshot(T0 + 28 * MINUTE);
        assert!(s.overflowing);
        assert_eq!(s.remaining_ms, -to_i64(3 * MINUTE));
        assert_eq!(t.status(), Status::Running);

        let ended = t.end(T0 + 28 * MINUTE).unwrap();
        let sum = summary(&ended);
        assert!(sum.completed);
        assert_eq!(sum.focused_ms, 28 * MINUTE);
        assert_eq!(sum.overflow_ms, 3 * MINUTE);
        assert_eq!(sum.planned_ms, 25 * MINUTE);
    }

    #[test]
    fn without_overflow_the_session_ends_at_the_goal() {
        let mut t = timer(Config {
            overflow: false,
            ..no_auto()
        });
        t.start_focus(Preset::POMODORO, T0).unwrap();
        // A late tick still ends the session at the goal, not at the tick.
        let out = t.tick(T0 + 25 * MINUTE + 3 * SECOND);
        let sum = summary(&out);
        assert!(sum.completed);
        assert_eq!(sum.ended_at, T0 + 25 * MINUTE);
        assert_eq!(sum.focused_ms, 25 * MINUTE);
        assert_eq!(sum.overflow_ms, 0);
        assert_eq!(t.phase(), Phase::ShortBreak);
        assert_eq!(t.status(), Status::Idle);
    }

    #[test]
    fn ending_early_saves_an_incomplete_session_and_earns_no_break() {
        let mut t = timer(Config::default());
        t.start_focus(Preset::POMODORO, T0).unwrap();
        t.pause(T0 + 5 * MINUTE).unwrap();
        t.resume(T0 + 7 * MINUTE).unwrap();
        let out = t.end(T0 + 12 * MINUTE).unwrap();
        assert_eq!(out.len(), 1);
        let sum = summary(&out);
        assert!(!sum.completed);
        assert_eq!(sum.focused_ms, 10 * MINUTE);
        assert_eq!(sum.pauses, 1);
        assert_eq!(sum.started_at, T0);
        assert_eq!(sum.ended_at, T0 + 12 * MINUTE);
        assert_eq!(t.phase(), Phase::Focus);
        assert_eq!(t.status(), Status::Idle);
        assert_eq!(t.snapshot(T0).completed_in_cycle, 0);
    }

    #[test]
    fn end_needs_an_active_phase() {
        let mut t = timer(Config::default());
        assert_eq!(t.end(T0), Err(Error::NotActive));
    }

    #[test]
    fn extend_moves_the_goal() {
        let mut t = timer(Config::default());
        t.start_focus(Preset::POMODORO, T0).unwrap();
        t.extend(5 * MINUTE, T0 + MINUTE).unwrap();
        let s = t.snapshot(T0 + MINUTE);
        assert_eq!(s.planned_ms, 30 * MINUTE);
        assert_eq!(s.remaining_ms, to_i64(29 * MINUTE));
        assert!(t.tick(T0 + 25 * MINUTE).is_empty());
        assert_eq!(t.tick(T0 + 30 * MINUTE), vec![Transition::GoalReached]);
    }

    #[test]
    fn extend_while_overflowing_announces_the_new_goal() {
        let mut t = timer(Config::default());
        t.start_focus(Preset::POMODORO, T0).unwrap();
        t.tick(T0 + 26 * MINUTE);
        t.extend(5 * MINUTE, T0 + 26 * MINUTE).unwrap();
        assert!(!t.snapshot(T0 + 26 * MINUTE).overflowing);
        assert_eq!(t.tick(T0 + 30 * MINUTE), vec![Transition::GoalReached]);
    }

    #[test]
    fn extend_only_applies_to_a_focus_goal() {
        let mut t = timer(Config::default());
        assert_eq!(t.extend(MINUTE, T0), Err(Error::NotActive));
        t.start_focus(Preset::FLOWTIME, T0).unwrap();
        assert_eq!(t.extend(MINUTE, T0), Err(Error::NoGoal));
        t.end(T0 + 10 * MINUTE).unwrap(); // break auto-starts
        assert_eq!(t.extend(MINUTE, T0 + 10 * MINUTE), Err(Error::NotFocus));
    }

    #[test]
    fn flowtime_counts_up_and_earns_a_fifth() {
        let mut t = timer(Config::default());
        t.start_focus(Preset::FLOWTIME, T0).unwrap();
        assert!(t.tick(T0 + 5 * 60 * MINUTE).is_empty());
        let s = t.snapshot(T0 + 40 * MINUTE);
        assert_eq!(s.elapsed_ms, 40 * MINUTE);
        assert_eq!(s.remaining_ms, -to_i64(40 * MINUTE));
        assert!(!s.overflowing);

        let out = t.end(T0 + 40 * MINUTE).unwrap();
        let sum = summary(&out);
        assert!(sum.completed);
        assert_eq!(sum.kind, Kind::Flowtime);
        assert_eq!(sum.planned_ms, 0);
        assert_eq!(sum.overflow_ms, 0);
        assert_eq!(t.phase(), Phase::ShortBreak);
        assert_eq!(t.snapshot(T0 + 40 * MINUTE).planned_ms, 8 * MINUTE);
    }

    #[test]
    fn break_auto_starts_after_a_completed_session() {
        let mut t = timer(Config::default());
        t.start_focus(Preset::DEEP, T0).unwrap();
        let out = t.end(T0 + 50 * MINUTE).unwrap();
        assert_eq!(out[1], Transition::PhaseStarted(Phase::ShortBreak));
        let s = t.snapshot(T0 + 52 * MINUTE);
        assert_eq!(s.status, Status::Running);
        assert_eq!(s.planned_ms, 10 * MINUTE);
        assert_eq!(s.elapsed_ms, 2 * MINUTE);
    }

    #[test]
    fn pending_break_waits_for_start() {
        let mut t = timer(no_auto());
        t.start_focus(Preset::DEEP, T0).unwrap();
        t.end(T0 + 50 * MINUTE).unwrap();
        assert_eq!(t.status(), Status::Idle);
        assert_eq!(t.phase(), Phase::ShortBreak);
        assert_eq!(t.snapshot(T0 + 60 * MINUTE).elapsed_ms, 0);
        assert_eq!(
            t.start(T0 + 60 * MINUTE).unwrap(),
            vec![Transition::PhaseStarted(Phase::ShortBreak)]
        );
        assert_eq!(
            t.snapshot(T0 + 61 * MINUTE).remaining_ms,
            to_i64(9 * MINUTE)
        );
    }

    #[test]
    fn starting_focus_skips_a_pending_break() {
        let mut t = timer(no_auto());
        t.start_focus(Preset::POMODORO, T0).unwrap();
        t.end(T0 + 25 * MINUTE).unwrap();
        t.start_focus(Preset::POMODORO, T0 + 26 * MINUTE).unwrap();
        assert_eq!(t.phase(), Phase::Focus);
        assert_eq!(t.status(), Status::Running);
    }

    #[test]
    fn break_ends_and_waits_for_the_next_session() {
        let mut t = timer(Config::default());
        t.start_focus(Preset::POMODORO, T0).unwrap();
        t.end(T0 + 25 * MINUTE).unwrap();
        assert_eq!(
            t.tick(T0 + 30 * MINUTE),
            vec![Transition::BreakEnded { skipped: false }]
        );
        assert_eq!(t.phase(), Phase::Focus);
        assert_eq!(t.status(), Status::Idle);
    }

    #[test]
    fn focus_auto_starts_after_a_break_at_the_break_end() {
        let mut t = timer(Config {
            auto_start_focus: true,
            ..Config::default()
        });
        t.start_focus(Preset::POMODORO, T0).unwrap();
        t.end(T0 + 25 * MINUTE).unwrap();
        let out = t.tick(T0 + 30 * MINUTE + 2 * SECOND);
        assert_eq!(
            out,
            vec![
                Transition::BreakEnded { skipped: false },
                Transition::PhaseStarted(Phase::Focus)
            ]
        );
        let s = t.snapshot(T0 + 31 * MINUTE);
        assert_eq!(s.started_at, Some(T0 + 30 * MINUTE));
        assert_eq!(s.elapsed_ms, MINUTE);
    }

    #[test]
    fn skipping_a_break_never_auto_starts_focus() {
        let mut t = timer(Config {
            auto_start_focus: true,
            ..Config::default()
        });
        t.start_focus(Preset::POMODORO, T0).unwrap();
        t.end(T0 + 25 * MINUTE).unwrap();
        assert_eq!(
            t.end(T0 + 26 * MINUTE).unwrap(),
            vec![Transition::BreakEnded { skipped: true }]
        );
        assert_eq!(t.status(), Status::Idle);
        assert_eq!(t.phase(), Phase::Focus);
    }

    #[test]
    fn long_break_after_every_n_completed_sessions() {
        let mut t = timer(Config {
            long_break_every: 3,
            ..no_auto()
        });
        let mut now = T0;
        let mut breaks = Vec::new();
        for _ in 0..6 {
            t.start_focus(Preset::POMODORO, now).unwrap();
            now += 25 * MINUTE;
            t.end(now).unwrap();
            breaks.push((t.phase(), t.snapshot(now).planned_ms));
            t.start_focus(Preset::POMODORO, now).unwrap(); // skip the break
            t.end(now).unwrap(); // incomplete: doesn't count
        }
        let short = (Phase::ShortBreak, 5 * MINUTE);
        let long = (Phase::LongBreak, 15 * MINUTE);
        assert_eq!(breaks, vec![short, short, long, short, short, long]);
    }

    #[test]
    fn cycle_progress_for_the_round_dots() {
        let mut t = timer(no_auto());
        let mut now = T0;
        let mut seen = Vec::new();
        for _ in 0..4 {
            t.start_focus(Preset::POMODORO, now).unwrap();
            seen.push(t.snapshot(now).completed_in_cycle);
            now += 25 * MINUTE;
            t.end(now).unwrap();
        }
        assert_eq!(seen, vec![0, 1, 2, 3]);
        assert_eq!(t.phase(), Phase::LongBreak);
        assert_eq!(t.snapshot(now).completed_in_cycle, 4);
        t.end(now).unwrap_err(); // idle: nothing to end
        t.start(now).unwrap();
        t.end(now).unwrap(); // skip the long break
        assert_eq!(t.snapshot(now).completed_in_cycle, 0);
    }

    #[test]
    fn config_changes_apply_to_the_next_phase() {
        let mut t = timer(Config::default());
        t.start_focus(Preset::POMODORO, T0).unwrap();
        t.set_config(Config {
            overflow: false,
            ..Config::default()
        });
        // Overflow is read at the goal, so this session already follows it.
        let out = t.tick(T0 + 25 * MINUTE);
        assert!(summary(&out).completed);
    }

    #[test]
    fn select_preset_sets_the_idle_goal() {
        let mut t = timer(Config::default());
        t.select_preset(Preset::custom(40, 8, 20)).unwrap();
        assert_eq!(t.snapshot(T0).planned_ms, 40 * MINUTE);
        assert_eq!(t.snapshot(T0).kind, Kind::Custom);
        t.start(T0).unwrap();
        assert_eq!(t.preset(), Preset::custom(40, 8, 20));
    }

    #[test]
    fn a_clock_going_backwards_never_underflows() {
        let mut t = timer(Config::default());
        t.start_focus(Preset::POMODORO, T0).unwrap();
        assert_eq!(t.snapshot(T0 - MINUTE).elapsed_ms, 0);
        assert!(t.tick(T0 - MINUTE).is_empty());
    }
}
