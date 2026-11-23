//! Goal progress and pace: are this goal's sessions on track for its date?

use chrono::NaiveDate;

/// How a goal is tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pace {
    /// The target is met.
    Done,
    /// Averaging at least 125% of what's needed.
    Ahead,
    /// Averaging at least 90% of what's needed.
    OnPace,
    Behind,
    /// Set less than a week ago and not yet on pace: too soon to say "behind".
    JustStarted,
    /// The date has passed and the target isn't met.
    Overdue,
    /// No date, so no pace.
    NoDeadline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GoalStats {
    pub done: u32,
    pub target: u32,
    /// 0..=100.
    pub percent: u32,
    /// Days until the due date (negative once passed).
    pub days_left: Option<i64>,
    /// Sessions a week needed to finish on time.
    pub needed_per_week: Option<f64>,
    /// Sessions a week over the last four weeks, or since the goal was set
    /// when that's sooner (counted as at least a week).
    pub averaged_per_week: f64,
    pub pace: Pace,
}

/// How many weeks of history the average looks at.
pub const PACE_WEEKS: u32 = 4;

/// A goal younger than this isn't called behind yet.
const GRACE_DAYS: i64 = 7;

/// `done` sessions toward `target`, due on `due`, set on `started`, with
/// `recent` sessions in the last four weeks.
pub fn goal_stats(
    done: u32,
    target: u32,
    due: Option<NaiveDate>,
    started: NaiveDate,
    today: NaiveDate,
    recent: u32,
) -> GoalStats {
    let percent = if target == 0 {
        0
    } else {
        (u64::from(done) * 100 / u64::from(target)).min(100) as u32
    };
    // A new goal is judged on the weeks it has had, not four weeks it hasn't.
    let age_days = (today - started).num_days().max(0);
    let weeks = (age_days as f64 / 7.0).clamp(1.0, f64::from(PACE_WEEKS));
    let averaged = f64::from(recent) / weeks;
    let days_left = due.map(|d| (d - today).num_days());
    let remaining = target.saturating_sub(done);
    let needed = days_left
        .filter(|&d| d > 0)
        .map(|d| f64::from(remaining) / (d as f64 / 7.0).max(1.0));

    let pace = if target > 0 && done >= target {
        Pace::Done
    } else {
        match (days_left, needed) {
            (None, _) => Pace::NoDeadline,
            (Some(d), _) if d <= 0 => Pace::Overdue,
            (_, Some(n)) if averaged >= n * 1.25 => Pace::Ahead,
            (_, Some(n)) if averaged >= n * 0.9 => Pace::OnPace,
            _ if age_days < GRACE_DAYS => Pace::JustStarted,
            _ => Pace::Behind,
        }
    };

    GoalStats {
        done,
        target,
        percent,
        days_left,
        needed_per_week: needed,
        averaged_per_week: averaged,
        pace,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn today() -> NaiveDate {
        d(2026, 11, 21)
    }

    /// Set long enough ago that the four-week average applies in full.
    fn long_ago() -> NaiveDate {
        d(2026, 1, 1)
    }

    #[test]
    fn a_new_goal_is_just_started_not_behind() {
        let due = Some(d(2027, 3, 1));
        let fresh = goal_stats(0, 60, due, today(), today(), 0);
        assert_eq!(fresh.pace, Pace::JustStarted);
        assert_eq!(fresh.averaged_per_week, 0.0);
        // Six days in, still too soon to judge.
        let six = goal_stats(0, 60, due, d(2026, 11, 15), today(), 0);
        assert_eq!(six.pace, Pace::JustStarted);
        // A week in with nothing done, it is behind.
        let week = goal_stats(0, 60, due, d(2026, 11, 14), today(), 0);
        assert_eq!(week.pace, Pace::Behind);
    }

    #[test]
    fn a_young_goal_averages_over_its_own_weeks() {
        // Set two weeks ago, 10 sessions since: 5 a week, not 2.5.
        let s = goal_stats(10, 60, Some(d(2027, 3, 1)), d(2026, 11, 7), today(), 10);
        assert_eq!(s.averaged_per_week, 5.0);
        // A strong first day can already be ahead.
        let quick = goal_stats(8, 60, Some(d(2027, 3, 1)), today(), today(), 8);
        assert_eq!(quick.pace, Pace::Ahead);
    }

    #[test]
    fn the_design_example_is_on_pace() {
        // 46 of 120, due 15 Mar 2027 (114 days ≈ 16.3 weeks): ~4.5 a week
        // needed; 28 in four weeks is 7 a week.
        let s = goal_stats(46, 120, Some(d(2027, 3, 15)), long_ago(), today(), 28);
        assert_eq!(s.percent, 38);
        assert_eq!(s.days_left, Some(114));
        let needed = s.needed_per_week.unwrap();
        assert!((needed - 4.54).abs() < 0.01, "{needed}");
        assert_eq!(s.averaged_per_week, 7.0);
        assert_eq!(s.pace, Pace::Ahead);
    }

    #[test]
    fn pace_bands() {
        let due = Some(d(2026, 12, 19)); // exactly 4 weeks
        // 40 left over 4 weeks: 10 a week needed.
        assert_eq!(
            goal_stats(0, 40, due, long_ago(), today(), 40).pace,
            Pace::OnPace
        );
        assert_eq!(
            goal_stats(0, 40, due, long_ago(), today(), 36).pace,
            Pace::OnPace
        );
        assert_eq!(
            goal_stats(0, 40, due, long_ago(), today(), 35).pace,
            Pace::Behind
        );
        assert_eq!(
            goal_stats(0, 40, due, long_ago(), today(), 50).pace,
            Pace::Ahead
        );
    }

    #[test]
    fn done_overdue_and_no_deadline() {
        assert_eq!(
            goal_stats(60, 60, None, long_ago(), today(), 0).pace,
            Pace::Done
        );
        assert_eq!(
            goal_stats(70, 60, None, long_ago(), today(), 0).percent,
            100
        );
        let past = goal_stats(10, 60, Some(d(2026, 11, 1)), long_ago(), today(), 4);
        assert_eq!(past.pace, Pace::Overdue);
        assert_eq!(past.needed_per_week, None);
        assert_eq!(
            goal_stats(10, 60, None, long_ago(), today(), 4).pace,
            Pace::NoDeadline
        );
    }

    #[test]
    fn the_last_week_needs_everything_left() {
        // Due in 3 days: weeks are counted as at least one.
        let s = goal_stats(50, 60, Some(d(2026, 11, 24)), long_ago(), today(), 8);
        assert_eq!(s.needed_per_week, Some(10.0));
    }

    #[test]
    fn a_zero_target_never_divides_by_zero() {
        let s = goal_stats(3, 0, Some(d(2027, 1, 1)), long_ago(), today(), 3);
        assert_eq!(s.percent, 0);
        assert_eq!(s.pace, Pace::Ahead);
    }
}
