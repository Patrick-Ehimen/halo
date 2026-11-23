//! Trading desk types sent over IPC. Every time and label is worked out in Rust.

use serde::Serialize;
use specta::Type;

use super::settings::MarketId;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ClockView {
    pub market: MarketId,
    pub name: String,
    /// The market your window follows.
    pub mine: bool,
    pub open: bool,
    /// "Open", "Closed" or "Weekend".
    pub status: String,
    /// The market's own time, "17:28".
    pub local_time: String,
    /// The session in your zone, "15:30–22:00".
    pub range: String,
    /// "Closes in 2h 05m 10s", "Opens in 4h 31m 12s", or the weekend note.
    pub countdown: String,
}

/// A piece of a 24-hour strip, in thousandths of the day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub start: u32,
    pub end: u32,
    /// For chart time: inside the trading window.
    pub inside: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TimelineRow {
    pub name: String,
    pub mine: bool,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CheckItemView {
    pub id: String,
    pub label: String,
    pub hint: String,
    pub checked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum GateState {
    Steps,
    SitOut,
    Ready,
    Open,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GateView {
    pub state: GateState,
    /// "2 steps to unlock", "Sitting out today", "Ready for 15:30", "Unlocked".
    pub label: String,
    pub detail: String,
    /// Head check was 3: half size, max 2 trades.
    pub half_size: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HeadCheckView {
    pub score: u8,
    pub factors: Vec<String>,
    /// "Logged at 09:12".
    pub at_label: String,
    /// A score of 1–2 today: locked for the rest of the day.
    pub locked_today: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChartDay {
    /// "M".
    pub label: String,
    pub minutes: u32,
    pub today: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChartSourceTime {
    pub label: String,
    /// Outside the window, "22m" ("<1m" under a minute).
    pub outside: String,
    /// Share of today's outside time, in thousandths.
    pub permille: u32,
    /// "53%".
    pub share: String,
    /// The source's colour, the same everywhere it shows.
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChartView {
    /// "41m".
    pub outside_label: String,
    /// "outside of 2h 05m today".
    pub total_label: String,
    /// Today's chart time on a 24-hour strip.
    pub segments: Vec<Segment>,
    /// Your window on the same strip.
    pub window: Vec<Segment>,
    /// Outside minutes, last 7 days, oldest first.
    pub week: Vec<ChartDay>,
    pub sources: Vec<ChartSourceTime>,
    /// "Tracked from TradingView, Binance and DEX Screener".
    pub tracking_label: String,
    /// Why browser sites aren't counted, when they aren't.
    pub browser_note: Option<String>,
    /// Automation was refused: offer to open System Settings.
    pub browser_blocked: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TradingView {
    /// "Lagos".
    pub zone_label: String,
    /// "10:28:12" in your zone.
    pub now_label: String,
    /// "Your window opens in", "Your window is open · closes in", or the weekend line.
    pub headline: String,
    /// "4h 31m 12s", empty on the weekend.
    pub headline_time: String,
    pub window_open: bool,
    pub weekend: bool,
    /// London and New York both open: peak liquidity.
    pub overlap: bool,
    pub clocks: Vec<ClockView>,
    pub timeline: Vec<TimelineRow>,
    /// Now on the 24-hour strips, thousandths of the day.
    pub now_permille: u32,
    pub checklist: Vec<CheckItemView>,
    /// "3 of 6 done · resets every morning".
    pub checklist_summary: String,
    pub checklist_done: u32,
    pub head_check: Option<HeadCheckView>,
    pub factor_options: Vec<String>,
    pub gate: GateView,
    pub chart: ChartView,
    /// For the popover header: "NY opens in 4h 31m".
    pub short_label: String,
}
