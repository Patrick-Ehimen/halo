//! Halo's timer engine.
//!
//! A pure state machine for focus sessions and breaks: presets (Pomodoro,
//! Deep, custom), Flowtime, overflow past the goal, pauses with a per-session
//! limit, long breaks every N rounds, and auto-start between phases.
//!
//! It never reads a clock. Every call takes `now` in unix milliseconds, so the
//! app drives it from a tokio interval and tests drive it directly.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod config;
mod engine;
mod manual;

pub use config::{
    Config, Kind, LONG_BREAK_EVERY, MAX_PAUSE_LIMIT, MINUTE, Preset, SECOND, flowtime_break_ms,
};
pub use engine::{
    Error, Millis, Phase, Result, SessionSummary, Snapshot, Status, Timer, Transition,
};
pub use manual::{MAX_MANUAL_MINUTES, manual_session};
