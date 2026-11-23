//! Streaks: consecutive local days with at least one saved session.
//!
//! Today counts once it has a session; until then a streak that ran through
//! yesterday is still alive ("Today counts").

use std::collections::BTreeSet;

use chrono::{Days, NaiveDate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Streak {
    /// Days in the current run, ending today or yesterday.
    pub current: u32,
    /// The longest run ever.
    pub longest: u32,
    /// Today already has a session.
    pub today_done: bool,
}

/// `days` are the local dates with at least one session.
pub fn streak(days: &BTreeSet<NaiveDate>, today: NaiveDate) -> Streak {
    let today_done = days.contains(&today);
    let mut current = 0;
    let mut cursor = if today_done {
        Some(today)
    } else {
        today.checked_sub_days(Days::new(1))
    };
    while let Some(d) = cursor.filter(|d| days.contains(d)) {
        current += 1;
        cursor = d.checked_sub_days(Days::new(1));
    }

    let mut longest = 0;
    let mut run = 0;
    let mut prev: Option<NaiveDate> = None;
    for &d in days.iter().filter(|d| **d <= today) {
        run = match prev {
            Some(p) if p.checked_add_days(Days::new(1)) == Some(d) => run + 1,
            _ => 1,
        };
        longest = longest.max(run);
        prev = Some(d);
    }

    Streak {
        current,
        longest,
        today_done,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, day).unwrap()
    }

    fn set(days: &[u32]) -> BTreeSet<NaiveDate> {
        days.iter().map(|&x| d(x)).collect()
    }

    #[test]
    fn counts_back_from_today() {
        let s = streak(&set(&[18, 19, 20, 21]), d(21));
        assert_eq!((s.current, s.longest, s.today_done), (4, 4, true));
    }

    #[test]
    fn today_not_done_yet_keeps_yesterdays_streak() {
        let s = streak(&set(&[19, 20]), d(21));
        assert_eq!((s.current, s.today_done), (2, false));
    }

    #[test]
    fn a_missed_day_ends_the_streak() {
        let s = streak(&set(&[10, 11, 12, 13, 15, 16]), d(18));
        assert_eq!((s.current, s.longest), (0, 4));
        let s = streak(&set(&[10, 11, 12, 13, 15, 16]), d(16));
        assert_eq!(s.current, 2);
    }

    #[test]
    fn crosses_months() {
        let days: BTreeSet<NaiveDate> = [(10, 30), (10, 31), (11, 1)]
            .iter()
            .map(|&(m, x)| NaiveDate::from_ymd_opt(2026, m, x).unwrap())
            .collect();
        assert_eq!(streak(&days, d(1)).current, 3);
    }

    #[test]
    fn no_days_no_streak() {
        assert_eq!(
            streak(&BTreeSet::new(), d(21)),
            Streak {
                current: 0,
                longest: 0,
                today_done: false
            }
        );
    }
}
