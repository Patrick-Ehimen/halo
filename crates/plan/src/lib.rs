//! Halo's planning logic, as pure functions: quick-add parsing, repeating
//! tasks, and goal progress and pace. "Today" is always passed in, so the
//! caller decides the user's local day and tests can pin it.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod day;
mod goals;
pub mod quick_add;
mod recurrence;

pub use day::{at_local, day_bounds, local_minute};
pub use goals::{GoalStats, PACE_WEEKS, Pace, goal_stats};
pub use quick_add::{QuickAdd, parse as parse_quick_add};
pub use recurrence::Recurrence;
