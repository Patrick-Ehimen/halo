//! macOS APIs (NSWorkspace, idle time, Shortcuts). Each capability degrades
//! cleanly when a permission is missing.

pub mod browser;

use objc2_app_kit::NSWorkspace;

/// The app in front: its name and bundle id. No permission needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontApp {
    pub name: String,
    pub bundle_id: String,
}

pub fn front_app() -> Option<FrontApp> {
    let workspace = NSWorkspace::sharedWorkspace();
    let app = workspace.frontmostApplication()?;
    Some(FrontApp {
        name: app
            .localizedName()
            .map(|s| s.to_string())
            .unwrap_or_default(),
        bundle_id: app
            .bundleIdentifier()
            .map(|s| s.to_string())
            .unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    /// Run by hand: `cargo test -p halo front_app_is_read -- --ignored --nocapture`.
    #[test]
    #[ignore = "reads the real frontmost app"]
    fn front_app_is_read() {
        let app = super::front_app().expect("a frontmost app");
        println!("front app: {app:?}");
        assert!(!app.name.is_empty());
    }
}
