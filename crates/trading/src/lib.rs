//! The trading desk's rules, as pure functions of time.
//!
//! Sessions are defined in their own time zone with local hours, so daylight
//! saving is handled by the zone database, never by fixed UTC offsets. All
//! instants are unix milliseconds.
//!
//! Halo never places trades, holds keys or gives advice. This crate only
//! answers "is it open", "when does it change", and "may I trade now by my
//! own rules".

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod chart;
mod clock;
mod gate;

pub use chart::{ChartRule, outside_ms, overlap_ms, sort_entries};
pub use clock::{
    Market, Span, State, TradingWindow, day_segments, fx_weekend, market_state, spans,
    window_spans, window_state,
};
pub use gate::{Gate, gate};

/// "4h 31m 12s", "12m 05s", "0m 09s". Display only.
pub fn countdown(ms: i64) -> String {
    let secs = ms.max(0) / 1000;
    let (h, m, s) = (secs / 3600, secs % 3600 / 60, secs % 60);
    if h > 0 {
        format!("{h}h {m:02}m {s:02}s")
    } else {
        format!("{m}m {s:02}s")
    }
}

#[cfg(test)]
mod tests {
    use super::countdown;

    #[test]
    fn countdowns() {
        assert_eq!(countdown(16_272_000), "4h 31m 12s");
        assert_eq!(countdown(725_000), "12m 05s");
        assert_eq!(countdown(-5), "0m 00s");
    }
}
