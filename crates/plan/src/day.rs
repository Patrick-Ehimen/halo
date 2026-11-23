//! The user's local day: where it starts and ends in unix ms, and where a
//! moment falls on it. Generic over the time zone so DST days can be tested.

use chrono::{Days, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike};

/// Unix ms of the local start of `date` and of the next day.
/// A DST gap at midnight resolves to the first valid moment.
pub fn day_bounds<Tz: TimeZone>(tz: &Tz, date: NaiveDate) -> (i64, i64) {
    let start = local_ms(tz, date.and_time(NaiveTime::MIN));
    let end = date
        .checked_add_days(Days::new(1))
        .map_or(i64::MAX, |d| local_ms(tz, d.and_time(NaiveTime::MIN)));
    (start, end)
}

/// The local date and minute of the day (0..1440) for unix ms.
pub fn local_minute<Tz: TimeZone>(tz: &Tz, ms: i64) -> Option<(NaiveDate, u32)> {
    let t = tz.timestamp_millis_opt(ms).single()?.naive_local();
    Some((t.date(), t.hour() * 60 + t.minute()))
}

/// Unix ms for a local date and time. In a DST gap, the first valid moment
/// after it; when a time happens twice, the first.
pub fn at_local<Tz: TimeZone>(tz: &Tz, date: NaiveDate, time: NaiveTime) -> i64 {
    local_ms(tz, date.and_time(time))
}

fn local_ms<Tz: TimeZone>(tz: &Tz, naive: NaiveDateTime) -> i64 {
    match tz.from_local_datetime(&naive) {
        LocalResult::Single(t) | LocalResult::Ambiguous(t, _) => t.timestamp_millis(),
        // A gap (clocks jumped forward): try each following minute.
        LocalResult::None => (1..=180)
            .find_map(|m| {
                tz.from_local_datetime(&(naive + chrono::Duration::minutes(m)))
                    .earliest()
            })
            .map_or(naive.and_utc().timestamp_millis(), |t| t.timestamp_millis()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::{Africa::Lagos, America::New_York};

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn t(h: u32, m: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, m, 0).unwrap()
    }

    const HOUR: i64 = 3_600_000;

    #[test]
    fn an_ordinary_day_is_24_hours() {
        let (start, end) = day_bounds(&Lagos, d(2026, 11, 21));
        assert_eq!(end - start, 24 * HOUR);
        // Lagos is UTC+1: local midnight is 23:00 UTC the day before.
        assert_eq!(
            start,
            d(2026, 11, 20)
                .and_time(t(23, 0))
                .and_utc()
                .timestamp_millis()
        );
    }

    #[test]
    fn dst_days_are_23_and_25_hours() {
        let (s, e) = day_bounds(&New_York, d(2026, 3, 8)); // spring forward
        assert_eq!(e - s, 23 * HOUR);
        let (s, e) = day_bounds(&New_York, d(2026, 11, 1)); // fall back
        assert_eq!(e - s, 25 * HOUR);
    }

    #[test]
    fn minutes_round_trip_through_local_time() {
        let ms = at_local(&Lagos, d(2026, 11, 21), t(10, 28));
        assert_eq!(local_minute(&Lagos, ms), Some((d(2026, 11, 21), 628)));
    }

    #[test]
    fn a_time_in_the_spring_gap_moves_forward() {
        // 02:30 doesn't exist on 8 Mar 2026 in New York; it becomes 03:00.
        let ms = at_local(&New_York, d(2026, 3, 8), t(2, 30));
        assert_eq!(local_minute(&New_York, ms), Some((d(2026, 3, 8), 180)));
    }

    #[test]
    fn a_repeated_time_takes_the_first() {
        // 01:30 happens twice on 1 Nov 2026 in New York.
        let ms = at_local(&New_York, d(2026, 11, 1), t(1, 30));
        let (start, _) = day_bounds(&New_York, d(2026, 11, 1));
        assert_eq!(ms - start, 90 * 60_000);
    }
}
