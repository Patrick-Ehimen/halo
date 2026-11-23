//! The host of a browser's front tab, through AppleScript (`osascript`).
//! Needs the Automation permission for each browser; see
//! docs/adr/0001-frontmost-app-and-browser-url.md.

use std::process::Command;

/// What asking a browser for its front tab gave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabHost {
    Host(String),
    /// No window or tab, or not a web page.
    Nothing,
    /// The user hasn't allowed Halo to control this browser (-1743).
    NotAllowed,
    /// Not a browser Halo can ask.
    Unsupported,
}

/// The AppleScript for a browser, by bundle id.
fn script(bundle_id: &str) -> Option<String> {
    let chromium = |app: &str| {
        format!(
            "tell application id \"{app}\" to if (count of windows) > 0 then get URL of active tab of front window"
        )
    };
    match bundle_id {
        "com.apple.Safari" | "com.apple.SafariTechnologyPreview" => Some(format!(
            "tell application id \"{bundle_id}\" to if (count of windows) > 0 then get URL of current tab of front window"
        )),
        "com.google.Chrome"
        | "com.google.Chrome.beta"
        | "com.brave.Browser"
        | "com.microsoft.edgemac"
        | "com.vivaldi.Vivaldi"
        | "com.operasoftware.Opera"
        | "company.thebrowser.Browser" => Some(chromium(bundle_id)),
        _ => None,
    }
}

pub fn is_supported(bundle_id: &str) -> bool {
    script(bundle_id).is_some()
}

pub fn front_tab_host(bundle_id: &str) -> TabHost {
    let Some(script) = script(bundle_id) else {
        return TabHost::Unsupported;
    };
    let Ok(out) = Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .output()
    else {
        return TabHost::Nothing;
    };
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return if err.contains("-1743") {
            TabHost::NotAllowed
        } else {
            TabHost::Nothing
        };
    }
    host_of(String::from_utf8_lossy(&out.stdout).trim()).map_or(TabHost::Nothing, TabHost::Host)
}

/// "https://www.tradingview.com/chart/x" → "www.tradingview.com".
pub fn host_of(url: &str) -> Option<String> {
    let rest = url.split_once("://")?;
    if !matches!(rest.0, "http" | "https") {
        return None;
    }
    let authority = rest.1.split(['/', '?', '#']).next()?;
    let host = authority.rsplit('@').next()?.split(':').next()?;
    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}

/// Opens System Settings at Privacy & Security → Automation.
pub fn open_automation_settings() -> std::io::Result<()> {
    Command::new("/usr/bin/open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Automation")
        .spawn()
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hosts_from_urls() {
        assert_eq!(
            host_of("https://www.tradingview.com/chart/abc?x=1").as_deref(),
            Some("www.tradingview.com")
        );
        assert_eq!(
            host_of("http://user@dexscreener.com:443/").as_deref(),
            Some("dexscreener.com")
        );
        assert_eq!(host_of("about:blank"), None);
        assert_eq!(host_of("file:///Users/me/a.html"), None);
        assert_eq!(host_of(""), None);
    }

    #[test]
    fn only_scriptable_browsers() {
        assert!(is_supported("com.google.Chrome"));
        assert!(is_supported("com.apple.Safari"));
        assert!(!is_supported("org.mozilla.firefox"));
    }
}
