//! Sessions logged by hand, for focus that happened away from the timer.

use crate::config::{Kind, MINUTE};
use crate::engine::{Error, Millis, Result, SessionSummary};

/// Longest session that can be logged by hand: 10 hours.
pub const MAX_MANUAL_MINUTES: u32 = 600;

/// Builds a completed session from a manual entry, or explains what's wrong.
///
/// The session must have ended by `now`; a minute of slack allows for a
/// session that ends right now.
pub fn manual_session(
    kind: Kind,
    started_at: Millis,
    minutes: u32,
    now: Millis,
) -> Result<SessionSummary> {
    if minutes == 0 {
        return Err(Error::Invalid("A session is at least 1 minute"));
    }
    if minutes > MAX_MANUAL_MINUTES {
        return Err(Error::Invalid("A session can be at most 10 hours"));
    }
    let length = u64::from(minutes) * MINUTE;
    let ended_at = started_at.saturating_add(length);
    if ended_at > now.saturating_add(MINUTE) {
        return Err(Error::Invalid("That session hasn't ended yet"));
    }
    Ok(SessionSummary {
        kind,
        started_at,
        ended_at,
        planned_ms: if kind == Kind::Flowtime { 0 } else { length },
        focused_ms: length,
        overflow_ms: 0,
        pauses: 0,
        completed: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: Millis = 1_700_000_000_000;

    #[test]
    fn logs_a_finished_session() {
        let s = manual_session(Kind::Deep, NOW - 120 * MINUTE, 50, NOW).unwrap();
        assert_eq!(s.ended_at, NOW - 70 * MINUTE);
        assert_eq!(s.focused_ms, 50 * MINUTE);
        assert_eq!(s.planned_ms, 50 * MINUTE);
        assert!(s.completed);
    }

    #[test]
    fn flowtime_has_no_goal() {
        let s = manual_session(Kind::Flowtime, NOW - 60 * MINUTE, 40, NOW).unwrap();
        assert_eq!(s.planned_ms, 0);
    }

    #[test]
    fn rejects_zero_and_too_long() {
        assert!(manual_session(Kind::Deep, NOW - MINUTE, 0, NOW).is_err());
        assert!(manual_session(Kind::Deep, NOW - 700 * MINUTE, 601, NOW).is_err());
        assert!(manual_session(Kind::Deep, NOW - 700 * MINUTE, 600, NOW).is_ok());
    }

    #[test]
    fn rejects_a_session_that_ends_in_the_future() {
        assert!(manual_session(Kind::Pomodoro, NOW - 10 * MINUTE, 25, NOW).is_err());
        // Ending right now is fine.
        assert!(manual_session(Kind::Pomodoro, NOW - 25 * MINUTE, 25, NOW).is_ok());
    }
}
