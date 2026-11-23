# Halo design

Live canvas (view, click through, comment): https://claude.ai/artifact/6KUBniDhU2wna5WumwFeqf

The canvas is the source of truth. This folder is a snapshot of it for the repo.

## Screens

| File | Screen | Interactive on the canvas |
| --- | --- | --- |
| `canvas/Brand.dc.html` | Logo, app icon, colors, type | — |
| `canvas/Main.dc.html` | Focus: live timer, intention, up next, Shield, ambience | Timer, modes, sounds |
| `canvas/Plan.dc.html` | Today plan: quick add, AI-drafted day, tasks, time blocks | — |
| `canvas/Stats.dc.html` | Focus score, heatmap, peak hours, projects, weekly review | — |
| `canvas/Trading.dc.html` | Trading desk: session clocks, pre-trade checklist, head check, chart time outside window | Clocks, checklist, head check |
| `canvas/Blocker.dc.html` | Shield: strict mode, block/allow lists, apps, schedules | Switches, mode |
| `canvas/Overlays.dc.html` | Mini timer, check-in, reflection, break overlay, ⌘K palette | — |
| `canvas/MenuBar.dc.html` | Menu bar popover (Focus / Trading), item states, right-click menu, settings | Tabs, timer, checklist, head check, settings |
| `canvas/Goals.dc.html` | Goals: long-term goals with pace, milestones, projects, and what today's plan contributes | — |
| `canvas/PlanGoals.dc.html` | Today plan linked to goals: goal progress strip, tasks grouped by goal, time blocks coloured by goal | — |

The `.dc.html` files are the canvas's design-component source. They need the canvas runtime, so they won't render by opening them in a browser. Use them as a reference for layout, copy, spacing, and colors, or open the canvas link to see them live.

## Assets

- `logo/halo-mark.svg`: the mark (amber ring with a spark in the gap)
- `logo/app-icon-dark.svg`: 1024 × 1024 macOS app icon, on Apple's 824 px icon grid
- `logo/app-icon-light.svg`: light variant
- `logo/app-icon-dark-1024.png`, `logo/app-icon-light-1024.png`: PNG exports of the icons
- `logo/menubar-template.svg`: black-on-transparent template image for the tray icon
- `logo/menubarTemplate.png`, `logo/menubarTemplate@2x.png`: 18 px and 36 px exports. The `Template` suffix tells macOS to tint it for light and dark menu bars.
- `tokens.css`: colors, type, radii, and shadows as CSS variables

To generate the Tauri icon set: `pnpm tauri icon design/logo/app-icon-dark-1024.png`.
