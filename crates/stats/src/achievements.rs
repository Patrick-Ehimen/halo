//! Achievements, worked out from history every time (nothing is stored).

use std::collections::{BTreeMap, BTreeSet};

use chrono::{NaiveDate, TimeZone, Timelike};

use crate::{Session, split};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Achievement {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    /// Progress toward it, and what's needed.
    pub progress: u32,
    pub target: u32,
    /// The local date it was first earned.
    pub earned_on: Option<NaiveDate>,
}

/// Every achievement with progress. `sessions` is the whole history, oldest
/// first; `goals_done` are the dates goals were finished.
pub fn achievements<Tz: TimeZone>(
    tz: &Tz,
    sessions: &[Session],
    goals_done: &[NaiveDate],
) -> Vec<Achievement> {
    let local = |ms: i64| {
        tz.timestamp_millis_opt(ms)
            .single()
            .map(|t| t.naive_local())
    };

    // Running counts, and the date each threshold was first reached.
    let first =
        |count: usize, target: usize, date: Option<NaiveDate>, slot: &mut Option<NaiveDate>| {
            if count >= target && slot.is_none() {
                *slot = date;
            }
        };

    let mut early = (0, None);
    let mut strict = (0, None);
    let mut flow = (0, None);
    let mut century = (0, None);
    let mut first_session = None;
    for s in sessions {
        let start = local(s.started_at);
        let date = start.map(|t| t.date());
        if first_session.is_none() {
            first_session = date;
        }
        century.0 += 1;
        first(century.0, 100, date, &mut century.1);
        if start.is_some_and(|t| t.hour() < 9) {
            early.0 += 1;
            first(early.0, 5, date, &mut early.1);
        }
        if s.strict {
            strict.0 += 1;
            first(strict.0, 10, date, &mut strict.1);
        }
        if s.rating == Some(4) {
            flow.0 += 1;
            first(flow.0, 10, date, &mut flow.1);
        }
    }

    // Days: the best day so far, and the longest run, in date order.
    let (from, to) = match (sessions.first(), sessions.last()) {
        (Some(a), Some(b)) => (
            local(a.started_at).map(|t| t.date()),
            local(b.ended_at).map(|t| t.date()),
        ),
        _ => (None, None),
    };
    let days: BTreeMap<NaiveDate, u64> = match (from, to) {
        (Some(f), Some(t)) => split::by_day(tz, sessions, f, t),
        _ => BTreeMap::new(),
    };
    let mut deep = (0u64, None);
    let mut run = (0u32, 0u32, None::<NaiveDate>); // (current, best, earned)
    let mut prev: Option<NaiveDate> = None;
    let active: BTreeSet<NaiveDate> = days
        .iter()
        .filter(|(_, s)| **s > 0)
        .map(|(d, _)| *d)
        .collect();
    for (d, secs) in &days {
        deep.0 = deep.0.max(*secs);
        if *secs >= 4 * 3600 && deep.1.is_none() {
            deep.1 = Some(*d);
        }
        if active.contains(d) {
            run.0 = match prev {
                Some(p) if p.succ_opt() == Some(*d) => run.0 + 1,
                _ => 1,
            };
            prev = Some(*d);
            run.1 = run.1.max(run.0);
            if run.0 >= 7 && run.2.is_none() {
                run.2 = Some(*d);
            }
        }
    }

    let n = |x: usize| u32::try_from(x).unwrap_or(u32::MAX);
    vec![
        Achievement {
            id: "first-session",
            name: "First session",
            description: "Finish your first focus session",
            progress: n(sessions.len().min(1)),
            target: 1,
            earned_on: first_session,
        },
        Achievement {
            id: "deep-diver",
            name: "Deep diver",
            description: "4h focus in one day",
            progress: u32::try_from(deep.0 / 60).unwrap_or(u32::MAX).min(240),
            target: 240,
            earned_on: deep.1,
        },
        Achievement {
            id: "early-bird",
            name: "Early bird",
            description: "5 sessions before 9:00",
            progress: n(early.0).min(5),
            target: 5,
            earned_on: early.1,
        },
        Achievement {
            id: "iron-streak",
            name: "Iron streak",
            description: "7 days in a row",
            progress: run.1.min(7),
            target: 7,
            earned_on: run.2,
        },
        Achievement {
            id: "in-flow",
            name: "In the zone",
            description: "Rate 10 sessions In flow",
            progress: n(flow.0).min(10),
            target: 10,
            earned_on: flow.1,
        },
        Achievement {
            id: "unshakeable",
            name: "Unshakeable",
            description: "10 strict sessions",
            progress: n(strict.0).min(10),
            target: 10,
            earned_on: strict.1,
        },
        Achievement {
            id: "century",
            name: "Century",
            description: "100 sessions",
            progress: n(century.0).min(100),
            target: 100,
            earned_on: century.1,
        },
        Achievement {
            id: "goal-getter",
            name: "Goal getter",
            description: "Finish a goal",
            progress: n(goals_done.len().min(1)),
            target: 1,
            earned_on: goals_done.iter().min().copied(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::Africa::Lagos;

    fn at(day: u32, h: u32, m: u32) -> i64 {
        Lagos
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2026, 11, day)
                    .unwrap()
                    .and_hms_opt(h, m, 0)
                    .unwrap(),
            )
            .unwrap()
            .timestamp_millis()
    }

    fn s(day: u32, h: u32, mins: u32) -> Session {
        let start = at(day, h, 0);
        Session {
            started_at: start,
            ended_at: start + i64::from(mins) * 60_000,
            focused_secs: mins * 60,
            planned_secs: 3000,
            rating: None,
            distractions: 0,
            strict: false,
            project_id: None,
        }
    }

    fn get<'a>(all: &'a [Achievement], id: &str) -> &'a Achievement {
        all.iter().find(|a| a.id == id).unwrap()
    }

    #[test]
    fn nothing_earned_without_sessions() {
        let all = achievements(&Lagos, &[], &[]);
        assert!(all.iter().all(|a| a.earned_on.is_none() && a.progress == 0));
    }

    #[test]
    fn early_bird_earns_on_the_fifth_early_session() {
        let sessions: Vec<Session> = (10..15).map(|d| s(d, 8, 25)).collect();
        let all = achievements(&Lagos, &sessions, &[]);
        let a = get(&all, "early-bird");
        assert_eq!(
            (a.progress, a.earned_on),
            (5, NaiveDate::from_ymd_opt(2026, 11, 14))
        );
        assert!(get(&all, "first-session").earned_on.is_some());
    }

    #[test]
    fn deep_diver_needs_four_hours_in_a_day() {
        let sessions = vec![s(10, 9, 120), s(10, 13, 100), s(11, 9, 130), s(11, 13, 110)];
        let all = achievements(&Lagos, &sessions, &[]);
        let a = get(&all, "deep-diver");
        assert_eq!(a.earned_on, NaiveDate::from_ymd_opt(2026, 11, 11));
        assert_eq!(a.progress, 240);
    }

    #[test]
    fn iron_streak_tracks_the_longest_run() {
        let mut sessions: Vec<Session> = (1..=5).map(|d| s(d, 10, 25)).collect();
        sessions.extend((8..=14).map(|d| s(d, 10, 25)));
        let all = achievements(&Lagos, &sessions, &[]);
        let a = get(&all, "iron-streak");
        assert_eq!(
            (a.progress, a.earned_on),
            (7, NaiveDate::from_ymd_opt(2026, 11, 14))
        );
    }

    #[test]
    fn goal_getter_uses_the_first_finished_goal() {
        let d1 = NaiveDate::from_ymd_opt(2026, 11, 20).unwrap();
        let d0 = NaiveDate::from_ymd_opt(2026, 10, 2).unwrap();
        let all = achievements(&Lagos, &[], &[d1, d0]);
        assert_eq!(get(&all, "goal-getter").earned_on, Some(d0));
    }
}
