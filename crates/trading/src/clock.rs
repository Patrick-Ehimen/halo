//! Session clocks: Sydney, Tokyo, London and New York, each open on its own
//! local weekdays between its own local hours.

use chrono::{Datelike, Days, NaiveDate, NaiveTime, TimeZone, Weekday};
use chrono_tz::Tz;
use halo_plan::{at_local, day_bounds};

const DAY_MS: i64 = 86_400_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Market {
    Sydney,
    Tokyo,
    London,
    NewYork,
}

impl Market {
    /// In the order the trading day moves around the world.
    pub const ALL: [Market; 4] = [
        Market::Sydney,
        Market::Tokyo,
        Market::London,
        Market::NewYork,
    ];

    pub fn zone(self) -> Tz {
        match self {
            Market::Sydney => chrono_tz::Australia::Sydney,
            Market::Tokyo => chrono_tz::Asia::Tokyo,
            Market::London => chrono_tz::Europe::London,
            Market::NewYork => chrono_tz::America::New_York,
        }
    }

    /// Local open and close.
    pub fn hours(self) -> (NaiveTime, NaiveTime) {
        let t = |h, m| NaiveTime::from_hms_opt(h, m, 0).unwrap_or(NaiveTime::MIN);
        match self {
            Market::Sydney | Market::Tokyo => (t(9, 0), t(18, 0)),
            Market::London => (t(8, 0), t(16, 30)),
            Market::NewYork => (t(9, 30), t(16, 0)),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Market::Sydney => "Sydney",
            Market::Tokyo => "Tokyo",
            Market::London => "London",
            Market::NewYork => "New York",
        }
    }

    /// For the menu bar: "SYD", "TYO", "LDN", "NY".
    pub fn short(self) -> &'static str {
        match self {
            Market::Sydney => "SYD",
            Market::Tokyo => "TYO",
            Market::London => "LDN",
            Market::NewYork => "NY",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Market::Sydney => "sydney",
            Market::Tokyo => "tokyo",
            Market::London => "london",
            Market::NewYork => "newYork",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Market::ALL.into_iter().find(|m| m.id() == id)
    }

    /// The session that opens on local `date`, if that's a weekday.
    fn span_on(self, date: NaiveDate) -> Option<Span> {
        if matches!(date.weekday(), Weekday::Sat | Weekday::Sun) {
            return None;
        }
        let tz = self.zone();
        let (open, close) = self.hours();
        Some(Span {
            market: self,
            start: at_local(&tz, date, open),
            end: at_local(&tz, date, close),
        })
    }

    fn local_date(self, ms: i64) -> Option<NaiveDate> {
        self.zone()
            .timestamp_millis_opt(ms)
            .single()
            .map(|t| t.date_naive())
    }
}

/// One session, open over `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub market: Market,
    pub start: i64,
    pub end: i64,
}

impl Span {
    pub fn contains(&self, ms: i64) -> bool {
        self.start <= ms && ms < self.end
    }

    /// The local weekday the session opened on, in its own zone.
    pub fn weekday(&self) -> Option<Weekday> {
        self.market.local_date(self.start).map(|d| d.weekday())
    }
}

/// Sessions of `market` overlapping `[from, to)`, oldest first.
pub fn spans(market: Market, from: i64, to: i64) -> Vec<Span> {
    let (Some(first), Some(last)) = (
        market.local_date(from.saturating_sub(DAY_MS)),
        market.local_date(to.saturating_add(DAY_MS)),
    ) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut d = first;
    while d <= last {
        if let Some(s) = market.span_on(d).filter(|s| s.end > from && s.start < to) {
            out.push(s);
        }
        match d.checked_add_days(Days::new(1)) {
            Some(n) => d = n,
            None => break,
        }
    }
    out
}

/// Open or closed, and when that changes next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub open: bool,
    /// The close if open, otherwise the next open. Unix ms.
    pub next_change: i64,
}

fn state_of(all: &[Span], now: i64) -> State {
    if let Some(s) = all.iter().find(|s| s.contains(now)) {
        return State {
            open: true,
            next_change: s.end,
        };
    }
    State {
        open: false,
        next_change: all
            .iter()
            .filter(|s| s.start > now)
            .map(|s| s.start)
            .min()
            .unwrap_or(i64::MAX),
    }
}

/// Weekends and holidays aside, the next week always holds the next change.
const LOOKAHEAD: i64 = 9 * DAY_MS;

pub fn market_state(market: Market, now: i64) -> State {
    state_of(&spans(market, now, now + LOOKAHEAD), now)
}

/// The FX weekend: from New York's Friday close to Sydney's Monday open.
/// No session is open, and the last one to close was New York on a Friday.
pub fn fx_weekend(now: i64) -> bool {
    if Market::ALL.iter().any(|&m| market_state(m, now).open) {
        return false;
    }
    let last = Market::ALL
        .iter()
        .flat_map(|&m| spans(m, now - 4 * DAY_MS, now))
        .filter(|s| s.end <= now)
        .max_by_key(|s| s.end);
    last.is_some_and(|s| s.market == Market::NewYork && s.weekday() == Some(Weekday::Fri))
}

/// The hours the user allows themselves to trade: one market's session, on
/// chosen weekdays (that market's local weekdays).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradingWindow {
    pub market: Market,
    /// Monday first.
    pub days: [bool; 7],
}

impl Default for TradingWindow {
    /// New York, weekdays.
    fn default() -> Self {
        TradingWindow {
            market: Market::NewYork,
            days: [true, true, true, true, true, false, false],
        }
    }
}

impl TradingWindow {
    fn allows(&self, s: &Span) -> bool {
        s.weekday().is_some_and(|d| {
            self.days
                .get(d.num_days_from_monday() as usize)
                .copied()
                .unwrap_or(false)
        })
    }
}

/// The window's spans overlapping `[from, to)`.
pub fn window_spans(w: &TradingWindow, from: i64, to: i64) -> Vec<Span> {
    spans(w.market, from, to)
        .into_iter()
        .filter(|s| w.allows(s))
        .collect()
}

pub fn window_state(w: &TradingWindow, now: i64) -> State {
    // Two weeks covers a window open one day a week.
    state_of(&window_spans(w, now, now + 2 * 7 * DAY_MS), now)
}

/// Where `market`'s sessions fall on the user's local `date`, as pairs of
/// thousandths of that day (0..=1000). A session crossing the user's
/// midnight comes back as two pieces. DST days (23 or 25 hours) still map to
/// 0..=1000.
pub fn day_segments<Z: TimeZone>(market: Market, tz: &Z, date: NaiveDate) -> Vec<(u32, u32)> {
    let (start, end) = day_bounds(tz, date);
    let len = (end - start).max(1);
    let at = |ms: i64| u32::try_from(((ms.clamp(start, end) - start) * 1000) / len).unwrap_or(1000);
    spans(market, start, end)
        .into_iter()
        .map(|s| (at(s.start), at(s.end)))
        .filter(|(a, b)| b > a)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use chrono_tz::{Africa::Lagos, UTC};

    fn utc(s: &str) -> i64 {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M")
            .unwrap()
            .and_utc()
            .timestamp_millis()
    }

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn new_york_open_and_close_in_winter_and_summer() {
        // EST (UTC−5): 09:30 = 14:30 UTC. Tue 24 Nov 2026.
        assert!(market_state(Market::NewYork, utc("2026-11-24 14:30")).open);
        assert!(!market_state(Market::NewYork, utc("2026-11-24 14:29")).open);
        assert_eq!(
            market_state(Market::NewYork, utc("2026-11-24 15:00")).next_change,
            utc("2026-11-24 21:00")
        );
        // EDT (UTC−4): 09:30 = 13:30 UTC. Tue 14 Jul 2026.
        assert!(market_state(Market::NewYork, utc("2026-07-14 13:30")).open);
        assert_eq!(
            market_state(Market::NewYork, utc("2026-07-14 12:00")).next_change,
            utc("2026-07-14 13:30")
        );
    }

    #[test]
    fn us_dst_switch_week_moves_new_york_but_not_london() {
        // US springs forward Sun 8 Mar 2026; the UK on Sun 29 Mar.
        // Fri 6 Mar: NY opens 14:30 UTC. Mon 9 Mar: 13:30 UTC.
        assert_eq!(
            spans(
                Market::NewYork,
                utc("2026-03-06 00:00"),
                utc("2026-03-06 23:59")
            )[0]
            .start,
            utc("2026-03-06 14:30")
        );
        assert_eq!(
            spans(
                Market::NewYork,
                utc("2026-03-09 00:00"),
                utc("2026-03-09 23:59")
            )[0]
            .start,
            utc("2026-03-09 13:30")
        );
        // London still GMT on 9 Mar: 08:00 = 08:00 UTC; BST from 30 Mar: 07:00 UTC.
        assert_eq!(
            spans(
                Market::London,
                utc("2026-03-09 00:00"),
                utc("2026-03-09 23:59")
            )[0]
            .start,
            utc("2026-03-09 08:00")
        );
        assert_eq!(
            spans(
                Market::London,
                utc("2026-03-30 00:00"),
                utc("2026-03-30 23:59")
            )[0]
            .start,
            utc("2026-03-30 07:00")
        );
    }

    #[test]
    fn sydney_dst_runs_the_other_way() {
        // Sydney: AEDT (UTC+11) until Sun 5 Apr 2026, then AEST (UTC+10).
        // Fri 3 Apr 09:00 AEDT = Thu 2 Apr 22:00 UTC; Mon 6 Apr 09:00 AEST = Sun 5 Apr 23:00 UTC.
        let first = |from, to| spans(Market::Sydney, utc(from), utc(to))[0].start;
        assert_eq!(
            first("2026-04-02 20:00", "2026-04-03 06:00"),
            utc("2026-04-02 22:00")
        );
        assert_eq!(
            first("2026-04-05 20:00", "2026-04-06 06:00"),
            utc("2026-04-05 23:00")
        );
    }

    #[test]
    fn tokyo_never_moves() {
        assert_eq!(
            spans(
                Market::Tokyo,
                utc("2026-07-14 00:00"),
                utc("2026-07-14 23:00")
            )[0]
            .start,
            utc("2026-07-14 00:00")
        );
        assert_eq!(
            spans(
                Market::Tokyo,
                utc("2026-11-24 00:00"),
                utc("2026-11-24 23:00")
            )[0]
            .start,
            utc("2026-11-24 00:00")
        );
    }

    #[test]
    fn the_fx_weekend_runs_from_friday_new_york_close_to_monday_sydney_open() {
        // Fri 27 Nov 2026: NY closes 21:00 UTC. Sydney (AEDT) opens Mon 30 Nov 09:00 = Sun 29 Nov 22:00 UTC.
        assert!(!fx_weekend(utc("2026-11-27 20:59")));
        assert!(fx_weekend(utc("2026-11-27 21:00")));
        assert!(fx_weekend(utc("2026-11-28 12:00")));
        assert!(fx_weekend(utc("2026-11-29 21:59")));
        assert!(!fx_weekend(utc("2026-11-29 22:00")));
        assert!(market_state(Market::Sydney, utc("2026-11-29 22:00")).open);
        // A weekday gap between sessions isn't a weekend: Tue after NY close.
        assert!(!fx_weekend(utc("2026-11-24 21:30")));
        // Saturday: every market is shut and opens next week.
        let sat = utc("2026-11-28 12:00");
        assert_eq!(
            market_state(Market::NewYork, sat).next_change,
            utc("2026-11-30 14:30")
        );
    }

    #[test]
    fn the_window_follows_its_days() {
        let w = TradingWindow::default();
        // Fri NY session is in, then closed until Monday.
        assert!(window_state(&w, utc("2026-11-27 15:00")).open);
        assert_eq!(
            window_state(&w, utc("2026-11-27 22:00")).next_change,
            utc("2026-11-30 14:30")
        );
        // Tuesdays and Thursdays only.
        let tt = TradingWindow {
            market: Market::London,
            days: [false, true, false, true, false, false, false],
        };
        assert!(!window_state(&tt, utc("2026-11-23 09:00")).open);
        assert_eq!(
            window_state(&tt, utc("2026-11-23 09:00")).next_change,
            utc("2026-11-24 08:00")
        );
        assert!(window_state(&tt, utc("2026-11-24 09:00")).open);
    }

    #[test]
    fn segments_cross_the_users_midnight() {
        // Sydney 09:00–18:00 AEDT = 22:00–07:00 UTC = 23:00–08:00 Lagos.
        // On Tue 24 Nov in Lagos: Tuesday's session runs 00:00–08:00 (it opened
        // Monday 23:00 Lagos), Wednesday's starts at 23:00.
        let segs = day_segments(Market::Sydney, &Lagos, d(2026, 11, 24));
        assert_eq!(segs, vec![(0, 333), (958, 1000)]);
        // New York in Lagos time: 15:30–22:00.
        assert_eq!(
            day_segments(Market::NewYork, &Lagos, d(2026, 11, 24)),
            vec![(645, 916)]
        );
        // Saturday in UTC: nothing.
        assert!(day_segments(Market::London, &UTC, d(2026, 11, 28)).is_empty());
    }

    #[test]
    fn segments_on_a_25_hour_day_still_span_the_day() {
        // Sun 25 Oct 2026 in London is 25 hours (BST ends at 01:00 UTC).
        // Sydney's Monday session opens 09:00 AEDT = Sun 22:00 UTC = 22:00 GMT,
        // 23 hours into the 25-hour day.
        let segs = day_segments(Market::Sydney, &chrono_tz::Europe::London, d(2026, 10, 25));
        assert_eq!(segs, vec![(920, 1000)]);
        // Tokyo seen from Sydney on an ordinary Monday: 10:00–19:00 AEST.
        let segs = day_segments(Market::Tokyo, &chrono_tz::Australia::Sydney, d(2026, 4, 6));
        assert_eq!(segs, vec![(416, 791)]);
    }

    #[test]
    fn ids_round_trip() {
        for m in Market::ALL {
            assert_eq!(Market::from_id(m.id()), Some(m));
        }
        assert_eq!(Market::from_id("paris"), None);
    }
}
