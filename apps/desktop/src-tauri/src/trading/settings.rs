//! Trading desk settings, saved as JSON under the `trading` settings key.

use chrono_tz::Tz;
use halo_trading::{ChartRule, Market, TradingWindow};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::error::{Error, Result};

pub const SETTINGS_KEY: &str = "trading";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum MarketId {
    Sydney,
    Tokyo,
    London,
    NewYork,
}

impl From<MarketId> for Market {
    fn from(m: MarketId) -> Self {
        match m {
            MarketId::Sydney => Market::Sydney,
            MarketId::Tokyo => Market::Tokyo,
            MarketId::London => Market::London,
            MarketId::NewYork => Market::NewYork,
        }
    }
}

/// A charting app or site to count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChartSource {
    pub label: String,
    /// App names, e.g. "TradingView".
    pub apps: Vec<String>,
    /// Sites, e.g. "tradingview.com" (subdomains count).
    pub hosts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct TradingSettings {
    /// IANA zone the desk is shown in.
    pub zone: String,
    /// The session your trading window follows.
    pub market: MarketId,
    /// Days the window is open, Monday first, in that market's own days.
    pub days: Vec<bool>,
    /// Notify after charting outside the window for `nudge_minutes`.
    pub nudge: bool,
    pub nudge_minutes: u32,
    /// Read the browser's front tab (needs Automation permission).
    pub track_browsers: bool,
    /// Show the time to your window in the menu bar when no session runs.
    pub menubar_countdown: bool,
    pub sources: Vec<ChartSource>,
}

impl Default for TradingSettings {
    fn default() -> Self {
        TradingSettings {
            zone: "Africa/Lagos".into(),
            market: MarketId::NewYork,
            days: vec![true, true, true, true, true, false, false],
            nudge: true,
            nudge_minutes: 5,
            track_browsers: false,
            menubar_countdown: true,
            sources: ChartRule::defaults()
                .into_iter()
                .map(|r| ChartSource {
                    label: r.label,
                    apps: r.apps,
                    hosts: r.hosts,
                })
                .collect(),
        }
    }
}

impl TradingSettings {
    pub fn tz(&self) -> Tz {
        self.zone.parse().unwrap_or(chrono_tz::Africa::Lagos)
    }

    pub fn window(&self) -> TradingWindow {
        let mut days = [false; 7];
        for (d, on) in days.iter_mut().zip(&self.days) {
            *d = *on;
        }
        TradingWindow {
            market: self.market.into(),
            days,
        }
    }

    pub fn rules(&self) -> Vec<ChartRule> {
        self.sources
            .iter()
            .map(|s| ChartRule {
                label: s.label.clone(),
                apps: s.apps.clone(),
                hosts: s.hosts.clone(),
            })
            .collect()
    }

    /// Puts each source's entries where they belong: anything without a dot
    /// is an app, anything with one a site. Fixes sources saved before apps
    /// could be typed into the sites field ("bybit app" as a site).
    pub fn tidied(mut self) -> Self {
        for s in &mut self.sources {
            s.label = s.label.trim().to_string();
            let entries: Vec<String> = s.apps.iter().chain(&s.hosts).cloned().collect();
            let (apps, hosts) = halo_trading::sort_entries(entries.iter().map(String::as_str));
            s.apps = apps;
            s.hosts = hosts;
        }
        self
    }

    /// Checks and tidies settings from the UI.
    pub fn validated(mut self) -> Result<Self> {
        if self.zone.parse::<Tz>().is_err() {
            return Err(Error::Invalid("Pick a time zone from the list"));
        }
        if self.days.len() != 7 || !self.days.iter().any(|d| *d) {
            return Err(Error::Invalid("Pick at least one trading day"));
        }
        if !(1..=60).contains(&self.nudge_minutes) {
            return Err(Error::Invalid("Nudge after 1 to 60 minutes"));
        }
        if self.sources.len() > 20 {
            return Err(Error::Invalid("Track up to 20 charting apps and sites"));
        }
        self = self.tidied();
        for s in &self.sources {
            if s.label.is_empty() {
                return Err(Error::Invalid("Give each charting source a name"));
            }
            if s.apps.is_empty() && s.hosts.is_empty() {
                return Err(Error::Invalid(
                    "Add an app or a site to each charting source",
                ));
            }
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_lagos_and_new_york_weekdays() {
        let s = TradingSettings::default();
        assert_eq!(s.tz(), chrono_tz::Africa::Lagos);
        assert_eq!(s.window(), TradingWindow::default());
        assert_eq!(s.sources.len(), 3);
    }

    #[test]
    fn validation() {
        let bad_zone = TradingSettings {
            zone: "Mars/Base".into(),
            ..Default::default()
        };
        assert!(bad_zone.validated().is_err());
        let no_days = TradingSettings {
            days: vec![false; 7],
            ..Default::default()
        };
        assert!(no_days.validated().is_err());
        let mut s = TradingSettings::default();
        s.sources.push(ChartSource {
            label: " Bybit ".into(),
            apps: vec![],
            hosts: vec!["https://www.Bybit.com/trade".into(), " ".into()],
        });
        let s = s.validated().unwrap();
        assert_eq!(s.sources[3].label, "Bybit");
        assert_eq!(s.sources[3].hosts, ["bybit.com"]);
        // An app typed where sites go moves to apps.
        let mut typed = TradingSettings::default();
        typed.sources.push(ChartSource {
            label: "Bybit".into(),
            apps: vec![],
            hosts: vec!["bybit app".into(), "bybit.com".into()],
        });
        let typed = typed.tidied();
        assert_eq!(typed.sources[3].apps, ["bybit"]);
        assert_eq!(typed.sources[3].hosts, ["bybit.com"]);
        let mut empty = TradingSettings::default();
        empty.sources[0].apps.clear();
        empty.sources[0].hosts.clear();
        assert!(empty.validated().is_err());
    }

    #[test]
    fn missing_fields_take_defaults() {
        let s: TradingSettings = serde_json::from_str(r#"{"zone":"Europe/London"}"#).unwrap();
        assert_eq!(s.zone, "Europe/London");
        assert_eq!(s.nudge_minutes, 5);
    }
}
