//! Halo's stats, as pure functions over saved sessions: where focus went by
//! day and hour, the focus score, streaks, achievements, the tree that grows
//! with today's focus, and CSV export.
//!
//! Everything takes the user's time zone and "today" as inputs, so tests can
//! pin them and DST days behave.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod achievements;
mod export;
mod score;
mod split;
mod streak;
mod tree;

pub use achievements::{Achievement, achievements};
pub use export::csv;
pub use score::{ScoreParts, focus_score};
pub use split::{by_day, by_hour, heat_level};
pub use streak::{Streak, streak};
pub use tree::{TreeStage, tree_stage};

/// A saved focus session, as stats needs it. Times are unix ms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub started_at: i64,
    pub ended_at: i64,
    /// Time focused, pauses excluded.
    pub focused_secs: u32,
    /// The goal; 0 for Flowtime.
    pub planned_secs: u32,
    /// 1 Rough, 2 Okay, 3 Good, 4 In flow.
    pub rating: Option<u8>,
    pub distractions: u32,
    pub strict: bool,
    pub project_id: Option<String>,
}

impl Session {
    /// Reached its goal (Flowtime always does).
    pub fn completed(&self) -> bool {
        self.planned_secs == 0 || self.focused_secs >= self.planned_secs
    }
}
