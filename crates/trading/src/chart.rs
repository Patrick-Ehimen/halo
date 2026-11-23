//! Chart time: time spent on charting apps and sites, and how much of it
//! fell outside the trading window.

use crate::Span;

/// A charting app or site to track. Matches an app by name (ignoring case,
/// spaces and a trailing "app") or by a part of its bundle id, or a browser
/// host (the host or any subdomain of it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartRule {
    /// "TradingView", shown in the breakdown.
    pub label: String,
    pub apps: Vec<String>,
    pub hosts: Vec<String>,
}

impl ChartRule {
    pub fn new(label: &str, apps: &[&str], hosts: &[&str]) -> Self {
        ChartRule {
            label: label.to_string(),
            apps: apps.iter().map(|s| (*s).to_string()).collect(),
            hosts: hosts.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    /// The defaults: TradingView, Binance and DEX Screener.
    pub fn defaults() -> Vec<ChartRule> {
        vec![
            ChartRule::new("TradingView", &["TradingView"], &["tradingview.com"]),
            ChartRule::new("Binance", &["Binance"], &["binance.com"]),
            ChartRule::new("DEX Screener", &[], &["dexscreener.com"]),
        ]
    }

    pub fn matches(&self, app: &str, bundle_id: &str, host: Option<&str>) -> bool {
        let name = app_key(app);
        let bundle_parts: Vec<String> = bundle_id.split('.').map(app_key).collect();
        let app_hit = self.apps.iter().map(|a| app_key(a)).any(|a| {
            !a.is_empty() && (a == name || bundle_parts.contains(&a) || a == app_key(bundle_id))
        });
        let host_hit = host.is_some_and(|h| {
            let h = h.trim().trim_end_matches('.').to_ascii_lowercase();
            self.hosts.iter().any(|rule| {
                let rule = rule.trim().to_ascii_lowercase();
                !rule.is_empty() && (h == rule || h.ends_with(&format!(".{rule}")))
            })
        });
        app_hit || host_hit
    }

    /// The first rule matching, if any.
    pub fn find<'a>(
        rules: &'a [ChartRule],
        app: &str,
        bundle_id: &str,
        host: Option<&str>,
    ) -> Option<&'a ChartRule> {
        rules.iter().find(|r| r.matches(app, bundle_id, host))
    }
}

/// "Pewter Desk app" → "pewterdesk": lower case, letters and digits only,
/// without a trailing " app".
fn app_key(name: &str) -> String {
    let trimmed = name.trim();
    let lower = trimmed.to_ascii_lowercase();
    let base = match lower.strip_suffix(" app") {
        Some(rest) if !rest.trim().is_empty() => rest,
        _ => lower.as_str(),
    };
    base.chars().filter(char::is_ascii_alphanumeric).collect()
}

/// Sorts what someone typed into apps and sites: anything with a dot (and no
/// spaces) is a site ("https://www.Bybit.com/trade" → "bybit.com"), anything
/// else an app ("Bybit", "Bybit app" → "Bybit").
pub fn sort_entries<'a>(entries: impl IntoIterator<Item = &'a str>) -> (Vec<String>, Vec<String>) {
    let mut apps: Vec<String> = Vec::new();
    let mut hosts: Vec<String> = Vec::new();
    for raw in entries {
        let e = raw.trim();
        if e.is_empty() {
            continue;
        }
        if e.contains('.') && !e.contains(' ') {
            let lower = e.to_ascii_lowercase();
            let rest = lower.split("://").last().unwrap_or(&lower);
            let host = rest.split(['/', '?', '#']).next().unwrap_or("");
            let host = host.trim_start_matches("www.").trim_end_matches('.');
            if !host.is_empty() && !hosts.iter().any(|x| x == host) {
                hosts.push(host.to_string());
            }
        } else {
            let lower = e.to_ascii_lowercase();
            let name = if lower.ends_with(" app") && e.len() > 4 {
                e[..e.len() - 4].trim()
            } else {
                e
            };
            if !name.is_empty() && !apps.iter().any(|x| app_key(x) == app_key(name)) {
                apps.push(name.to_string());
            }
        }
    }
    (apps, hosts)
}

/// Milliseconds of `[start, end)` inside any of `spans`.
pub fn overlap_ms(start: i64, end: i64, spans: &[Span]) -> i64 {
    spans
        .iter()
        .map(|s| (end.min(s.end) - start.max(s.start)).max(0))
        .sum()
}

/// Milliseconds of `[start, end)` outside the window's `spans`.
pub fn outside_ms(start: i64, end: i64, spans: &[Span]) -> i64 {
    ((end - start).max(0) - overlap_ms(start, end, spans)).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Market;

    fn span(start: i64, end: i64) -> Span {
        Span {
            market: Market::NewYork,
            start,
            end,
        }
    }

    #[test]
    fn matches_apps_and_hosts() {
        let rules = ChartRule::defaults();
        let label =
            |app, bundle, host| ChartRule::find(&rules, app, bundle, host).map(|r| r.label.clone());
        assert_eq!(
            label(
                "TradingView",
                "com.tradingview.tradingviewapp.desktop",
                None
            )
            .as_deref(),
            Some("TradingView")
        );
        assert_eq!(
            label(
                "Google Chrome",
                "com.google.Chrome",
                Some("www.tradingview.com")
            )
            .as_deref(),
            Some("TradingView")
        );
        assert_eq!(
            label("Arc", "company.thebrowser.Browser", Some("dexscreener.com")).as_deref(),
            Some("DEX Screener")
        );
        assert_eq!(
            label("Safari", "com.apple.Safari", Some("nottradingview.com")),
            None
        );
        assert_eq!(label("Safari", "com.apple.Safari", None), None);
    }

    #[test]
    fn app_names_match_loosely_and_by_bundle_id() {
        let rule = ChartRule::new("PewterDesk", &["Pewter Desk"], &[]);
        assert!(rule.matches("PewterDesk", "app.pewterdesk.desktop", None));
        assert!(rule.matches("pewterdesk", "", None));
        let bybit = ChartRule::new("Bybit", &["bybit app"], &[]);
        assert!(bybit.matches("Bybit", "com.bybit.desktop", None));
        // By bundle id alone, when the shown name differs.
        let by_id = ChartRule::new("Bybit", &["bybit"], &[]);
        assert!(by_id.matches("Bybit Trading", "com.bybit.desktop", None));
        assert!(!by_id.matches("Notes", "com.apple.Notes", None));
        // "Snapp" isn't "Sn" just because it ends in "app".
        assert!(!ChartRule::new("Sn", &["Sn"], &[]).matches("Snapp", "", None));
    }

    #[test]
    fn sorts_typed_entries_into_apps_and_sites() {
        let (apps, hosts) = sort_entries([
            "Bybit",
            "bybit app",
            "https://www.Bybit.com/trade",
            " ",
            "PewterDesk",
            "dexscreener.com",
        ]);
        assert_eq!(apps, ["Bybit", "PewterDesk"]);
        assert_eq!(hosts, ["bybit.com", "dexscreener.com"]);
    }

    #[test]
    fn splits_inside_and_outside() {
        let w = [span(100, 200), span(300, 400)];
        assert_eq!(overlap_ms(50, 350, &w), 150);
        assert_eq!(outside_ms(50, 350, &w), 150);
        assert_eq!(outside_ms(120, 180, &w), 0);
        assert_eq!(outside_ms(200, 300, &w), 100);
        assert_eq!(outside_ms(10, 5, &w), 0);
    }
}
