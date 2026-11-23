//! Spreads each session's focus over the local days and hours it spanned.
//! Focus is shared out in proportion to wall time, so a session across
//! midnight counts on both days and pauses spread evenly.

use std::collections::BTreeMap;

use chrono::{Days, NaiveDate, NaiveTime, TimeZone, Timelike};

use crate::Session;

/// Focused seconds per local date, for sessions overlapping `[from, to]` (dates included).
pub fn by_day<Tz: TimeZone>(
    tz: &Tz,
    sessions: &[Session],
    from: NaiveDate,
    to: NaiveDate,
) -> BTreeMap<NaiveDate, u64> {
    let mut out: BTreeMap<NaiveDate, u64> = BTreeMap::new();
    let mut d = from;
    while d <= to {
        out.insert(d, 0);
        match d.checked_add_days(Days::new(1)) {
            Some(next) => d = next,
            None => break,
        }
    }
    for s in sessions {
        spread(tz, s, Bucket::Day, |key, secs| {
            if let Key::Day(date) = key
                && let Some(v) = out.get_mut(&date)
            {
                *v += secs;
            }
        });
    }
    out
}

/// Focused seconds per local hour of the day (0..24).
pub fn by_hour<Tz: TimeZone>(tz: &Tz, sessions: &[Session]) -> [u64; 24] {
    let mut out = [0u64; 24];
    for s in sessions {
        spread(tz, s, Bucket::Hour, |key, secs| {
            if let Key::Hour(h) = key
                && let Some(v) = out.get_mut(h as usize)
            {
                *v += secs;
            }
        });
    }
    out
}

/// Heatmap shade 0..=4 for a day's focus against the daily goal:
/// none, under a quarter, under half, under the goal, goal met.
pub fn heat_level(secs: u64, goal_secs: u64) -> u8 {
    if secs == 0 {
        return 0;
    }
    if goal_secs == 0 {
        return 4;
    }
    match secs * 4 / goal_secs {
        0 => 1,
        1 => 2,
        2 | 3 => 3,
        _ => 4,
    }
}

#[derive(Clone, Copy)]
enum Bucket {
    Day,
    Hour,
}

#[derive(Clone, Copy)]
enum Key {
    Day(NaiveDate),
    Hour(u32),
}

/// Calls `add(bucket, seconds)` for each local day or hour the session
/// touched, sharing its focused seconds by wall-time overlap.
fn spread<Tz: TimeZone>(tz: &Tz, s: &Session, bucket: Bucket, mut add: impl FnMut(Key, u64)) {
    let focused = u64::from(s.focused_secs);
    let wall = s.ended_at.saturating_sub(s.started_at);
    let Some(start) = tz.timestamp_millis_opt(s.started_at).single() else {
        return;
    };
    if wall <= 0 {
        let local = start.naive_local();
        add(key_of(bucket, local.date(), local.hour()), focused);
        return;
    }
    // Walk bucket boundaries in local time from the start to the end.
    let mut cursor = s.started_at;
    let mut given = 0u64;
    while cursor < s.ended_at {
        let Some(local) = tz.timestamp_millis_opt(cursor).single() else {
            break;
        };
        let local = local.naive_local();
        let next_local = match bucket {
            Bucket::Day => local
                .date()
                .checked_add_days(Days::new(1))
                .map(|d| d.and_time(NaiveTime::MIN)),
            Bucket::Hour => Some(
                local.date().and_time(NaiveTime::MIN)
                    + chrono::Duration::hours(i64::from(local.hour()) + 1),
            ),
        };
        let boundary = next_local
            .and_then(|n| tz.from_local_datetime(&n).earliest())
            .map_or(s.ended_at, |t| t.timestamp_millis())
            .max(cursor + 1);
        let end = boundary.min(s.ended_at);
        let share = if end == s.ended_at {
            focused - given // the last piece takes the rounding remainder
        } else {
            (focused as u128 * (end - cursor) as u128 / wall as u128) as u64
        };
        given += share;
        add(key_of(bucket, local.date(), local.hour()), share);
        cursor = end;
    }
}

fn key_of(bucket: Bucket, date: NaiveDate, hour: u32) -> Key {
    match bucket {
        Bucket::Day => Key::Day(date),
        Bucket::Hour => Key::Hour(hour),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::{Africa::Lagos, America::New_York};

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn at<Tz: TimeZone>(tz: &Tz, date: NaiveDate, h: u32, m: u32) -> i64 {
        tz.from_local_datetime(&date.and_hms_opt(h, m, 0).unwrap())
            .earliest()
            .unwrap()
            .timestamp_millis()
    }

    fn session(started_at: i64, ended_at: i64, focused_secs: u32) -> Session {
        Session {
            started_at,
            ended_at,
            focused_secs,
            planned_secs: 3000,
            rating: None,
            distractions: 0,
            strict: false,
            project_id: None,
        }
    }

    #[test]
    fn a_session_counts_on_its_day() {
        let day = d(2026, 11, 21);
        let s = session(at(&Lagos, day, 9, 30), at(&Lagos, day, 10, 20), 3000);
        let days = by_day(&Lagos, &[s], d(2026, 11, 20), d(2026, 11, 22));
        assert_eq!(days.values().copied().collect::<Vec<_>>(), [0, 3000, 0]);
    }

    #[test]
    fn a_session_across_midnight_is_split_by_wall_time() {
        // 23:30 to 00:30, an hour of wall time, 50 minutes focused.
        let s = session(
            at(&Lagos, d(2026, 11, 21), 23, 30),
            at(&Lagos, d(2026, 11, 22), 0, 30),
            3000,
        );
        let days = by_day(&Lagos, &[s], d(2026, 11, 21), d(2026, 11, 22));
        assert_eq!(days[&d(2026, 11, 21)], 1500);
        assert_eq!(days[&d(2026, 11, 22)], 1500);
    }

    #[test]
    fn hours_add_up_to_the_focused_time() {
        let day = d(2026, 11, 21);
        let s = session(at(&Lagos, day, 9, 40), at(&Lagos, day, 11, 10), 5400);
        let hours = by_hour(&Lagos, &[s]);
        assert_eq!((hours[9], hours[10], hours[11]), (1200, 3600, 600));
        assert_eq!(hours.iter().sum::<u64>(), 5400);
    }

    #[test]
    fn dst_fall_back_day_counts_correctly() {
        // 1 Nov 2026 in New York: 01:00–02:00 happens twice. A session from
        // 00:30 to 02:30 local spans 3 hours of real time.
        let day = d(2026, 11, 1);
        let start = at(&New_York, day, 0, 30);
        let end = start + 3 * 3_600_000;
        let s = session(start, end, 3 * 3600);
        let days = by_day(&New_York, std::slice::from_ref(&s), day, day);
        assert_eq!(days[&day], 3 * 3600);
        let hours = by_hour(&New_York, &[s]);
        assert_eq!(hours.iter().sum::<u64>(), 3 * 3600);
        assert_eq!(hours[1], 2 * 3600, "the repeated hour counts twice");
    }

    #[test]
    fn dst_spring_forward_day_is_split_at_the_real_midnight() {
        // 8 Mar 2026 in New York is 23 hours long.
        let s = session(
            at(&New_York, d(2026, 3, 7), 23, 0),
            at(&New_York, d(2026, 3, 8), 1, 0),
            7200,
        );
        let days = by_day(&New_York, &[s], d(2026, 3, 7), d(2026, 3, 8));
        assert_eq!((days[&d(2026, 3, 7)], days[&d(2026, 3, 8)]), (3600, 3600));
    }

    #[test]
    fn heat_levels_follow_the_daily_goal() {
        let goal = 4 * 3600;
        assert_eq!(heat_level(0, goal), 0);
        assert_eq!(heat_level(1800, goal), 1);
        assert_eq!(heat_level(3600, goal), 2);
        assert_eq!(heat_level(3 * 3600, goal), 3);
        assert_eq!(heat_level(4 * 3600, goal), 4);
        assert_eq!(heat_level(1, 0), 4);
    }
}
