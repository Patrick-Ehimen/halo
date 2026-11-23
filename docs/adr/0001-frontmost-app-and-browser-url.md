# 0001: Frontmost app and browser URL for chart time

Status: accepted (2026-11-23)

## Context

The trading desk counts "chart time": minutes spent on charting apps and
sites (TradingView, Binance, DEX Screener; editable), and how much of it falls
outside the user's trading window. Halo needs two facts every few seconds:

1. Which app is in front.
2. If it's a browser, which site its front tab shows.

macOS gives the first freely. The second has no public API without a
permission.

## Options

**Frontmost app**

- `NSWorkspace.sharedWorkspace.frontmostApplication` via the `objc2` crates.
  No permission. Also needed later for app blocking (Shield).
- Shelling out to `lsappinfo front`. No new crates, but a process per poll and
  an undocumented output format.

**Browser URL**

- AppleScript through `osascript` ("URL of active tab of front window").
  Needs the Automation permission per browser, which macOS asks for the first
  time Halo sends the browser an Apple Event. Works for Safari and the
  Chromium family (Chrome, Arc, Brave, Edge, Vivaldi, Opera). Not Firefox.
- Accessibility API (read the address bar). Needs Accessibility, a broader
  and scarier permission, and breaks with browser UI changes.
- A browser extension. Most accurate, but another product to ship and install
  per browser. Revisit with Shield's website blocking.

## Decision

- Frontmost app from `NSWorkspace` (`objc2`, `objc2-app-kit`,
  `objc2-foundation`), polled every 5 seconds on a background thread.
- Browser URLs through `osascript`, only when the user turns on "Track
  charting sites in browsers". Off by default, so Halo never triggers an
  Automation prompt the user didn't ask for. Only the host is kept; the full
  URL and page title are never stored.
- `Info.plist` carries `NSAppleEventsUsageDescription` explaining why.

## Degrading without permission

- If a browser answers "not authorized" (error -1743), Halo stops asking that
  browser, keeps tracking apps, and the Trading screen says: "Browser sites
  aren't tracked: allow Halo under System Settings → Privacy & Security →
  Automation." with a button that opens that pane.
- Firefox and other browsers without AppleScript count by app only.
- Chart activity stays on the device. It is never synced to Supabase.

## Consequences

- No new privileged pieces; nothing runs when Halo quits.
- A `osascript` process every 5 s only while a supported browser is in front
  and the setting is on.
- Shield (phase 6) can reuse the frontmost-app watcher for app blocking.
