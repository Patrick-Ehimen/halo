//! Halo's menu bar item, as pure functions: what the title says and what the
//! icon looks like for a timer snapshot, and the icon pixels themselves.
//!
//! Icons are macOS template images: black with alpha, which the system tints
//! for light and dark menu bars. They're drawn here rather than loaded so the
//! ring can fill as a session goes.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use halo_timer::{Kind, Phase, Snapshot, Status};

/// Icon side in pixels. macOS shows tray icons at 18 pt, so this is @2x.
pub const ICON_SIZE: u32 = 36;

/// How finely the ring moves: 60 steps per lap, so the icon is redrawn at
/// most about once every 25 s of a 25 min session.
const RING_STEPS: u16 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    /// The Halo mark: ring with a gap and a dot. Shown while idle.
    Logo,
    /// A ring that fills clockwise from the top.
    Ring {
        /// 0..=RING_STEPS.
        step: u16,
        /// Paused: drawn fainter.
        dim: bool,
    },
}

/// What the menu bar item shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayView {
    /// Text next to the icon; `None` shows the icon alone.
    pub title: Option<String>,
    pub icon: Icon,
    /// Hover text.
    pub tooltip: String,
}

/// How the user wants the menu bar item to look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// Show the time next to the icon; off shows the icon alone.
    pub show_time: bool,
    /// `18:24` with seconds, or `18m` without.
    pub show_seconds: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            show_time: true,
            show_seconds: true,
        }
    }
}

/// The menu bar item for a timer snapshot.
pub fn view(s: &Snapshot, options: Options) -> TrayView {
    if s.status == Status::Idle {
        let tooltip = match s.phase {
            Phase::Focus => "Halo: ready to focus",
            Phase::ShortBreak | Phase::LongBreak => "Halo: break waiting",
        };
        return TrayView {
            title: None,
            icon: Icon::Logo,
            tooltip: tooltip.to_string(),
        };
    }

    let flowtime = s.phase == Phase::Focus && s.kind == Kind::Flowtime;
    let fmt = |ms: u64, round: Round| {
        if options.show_seconds {
            clock(ms, round)
        } else {
            minutes(ms, round)
        }
    };
    let title = if flowtime {
        fmt(s.elapsed_ms, Round::Down)
    } else if s.overflowing {
        format!("+{}", fmt(s.remaining_ms.unsigned_abs(), Round::Down))
    } else {
        fmt(s.remaining_ms.max(0).unsigned_abs(), Round::Up)
    };

    let permille = if flowtime {
        (s.elapsed_ms % HOUR_MS) * 1000 / HOUR_MS
    } else {
        (s.elapsed_ms.min(s.planned_ms) * 1000)
            .checked_div(s.planned_ms)
            .unwrap_or(0)
    };
    let step = u16::try_from(permille * u64::from(RING_STEPS) / 1000).unwrap_or(RING_STEPS);

    let what = match s.phase {
        Phase::Focus if s.overflowing => "overflow",
        Phase::Focus => "focusing",
        Phase::ShortBreak => "short break",
        Phase::LongBreak => "long break",
    };
    let paused = s.status == Status::Paused;
    TrayView {
        title: options.show_time.then_some(title),
        icon: Icon::Ring { step, dim: paused },
        tooltip: if paused {
            format!("Halo: {what}, paused")
        } else {
            format!("Halo: {what}")
        },
    }
}

const HOUR_MS: u64 = 3_600_000;

#[derive(Clone, Copy)]
enum Round {
    Up,
    Down,
}

/// mm:ss, or h:mm:ss from an hour.
fn clock(ms: u64, round: Round) -> String {
    let secs = match round {
        Round::Up => ms.div_ceil(1000),
        Round::Down => ms / 1000,
    };
    let (h, m, s) = (secs / 3600, secs % 3600 / 60, secs % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

/// `18m`, or `1h 05m` from an hour. Counting down rounds up, so the last
/// minute shows `1m` until the end.
fn minutes(ms: u64, round: Round) -> String {
    let mins = match round {
        Round::Up => ms.div_ceil(60_000),
        Round::Down => ms / 60_000,
    };
    if mins >= 60 {
        format!("{}h {:02}m", mins / 60, mins % 60)
    } else {
        format!("{mins}m")
    }
}

/// RGBA pixels, `ICON_SIZE` × `ICON_SIZE`. With no `tint` it's black with
/// alpha (a template image macOS tints); with a tint, it's that colour.
pub fn render(icon: Icon, tint: Option<[u8; 3]>) -> Vec<u8> {
    // Geometry of the menu bar mark (design/logo/menubar-template.svg):
    // r = 22 and stroke 8 on a 64 grid, dot r = 4.5 at (37.1, 10.6).
    let scale = ICON_SIZE as f32 / 64.0;
    let radius = 22.0 * scale;
    let half_stroke = 4.0 * scale;
    let center = ICON_SIZE as f32 / 2.0;

    let layers: Vec<(Arc, f32)> = match icon {
        Icon::Logo => vec![(
            // The SVG arc starts 40° clockwise from the top and covers
            // 118 / 138.23 of the circle, leaving a gap for the dot.
            Arc::new(40.0, 40.0 + 360.0 * 118.0 / 138.23),
            1.0,
        )],
        Icon::Ring { step, dim } => {
            let end = 360.0 * f32::from(step.min(RING_STEPS)) / f32::from(RING_STEPS);
            let mut layers = vec![(Arc::full(), 0.32)];
            if step > 0 {
                layers.push((Arc::new(0.0, end), if dim { 0.55 } else { 1.0 }));
            }
            layers
        }
    };
    let dot = matches!(icon, Icon::Logo).then(|| (37.1 * scale, 10.6 * scale, 4.5 * scale));

    let mut rgba = vec![0u8; (ICON_SIZE * ICON_SIZE * 4) as usize];
    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let mut alpha: f32 = 0.0;
            for (arc, strength) in &layers {
                let dist = arc.distance(px - center, py - center, radius);
                alpha = alpha.max(coverage(dist, half_stroke) * strength);
            }
            if let Some((dx, dy, r)) = dot {
                let dist = ((px - dx).powi(2) + (py - dy).powi(2)).sqrt();
                alpha = alpha.max(coverage(dist, r));
            }
            let i = ((y * ICON_SIZE + x) * 4) as usize;
            if let Some([r, g, b]) = tint {
                rgba[i] = r;
                rgba[i + 1] = g;
                rgba[i + 2] = b;
            }
            rgba[i + 3] = (alpha * 255.0).round() as u8;
        }
    }
    rgba
}

/// Anti-aliased coverage of a pixel `dist` from a shape edge of half-width `half`.
fn coverage(dist: f32, half: f32) -> f32 {
    (half + 0.5 - dist).clamp(0.0, 1.0)
}

/// An arc of the ring, in degrees clockwise from the top, with round caps.
struct Arc {
    start: f32,
    end: f32,
}

impl Arc {
    fn new(start: f32, end: f32) -> Self {
        Arc { start, end }
    }

    fn full() -> Self {
        Arc {
            start: 0.0,
            end: 360.0,
        }
    }

    /// Distance from a point (relative to the center, y down) to the arc's centreline.
    fn distance(&self, dx: f32, dy: f32, radius: f32) -> f32 {
        let d = (dx * dx + dy * dy).sqrt();
        let angle = dx.atan2(-dy).to_degrees().rem_euclid(360.0);
        let span = self.end - self.start;
        if span >= 360.0 || (angle - self.start).rem_euclid(360.0) <= span {
            return (d - radius).abs();
        }
        // Outside the sweep: distance to the nearer round cap.
        let cap = |deg: f32| {
            let t = deg.to_radians();
            let (cx, cy) = (radius * t.sin(), -radius * t.cos());
            ((dx - cx).powi(2) + (dy - cy).powi(2)).sqrt()
        };
        cap(self.start).min(cap(self.end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo_timer::MINUTE;

    fn snap(phase: Phase, status: Status) -> Snapshot {
        Snapshot {
            phase,
            status,
            kind: Kind::Deep,
            planned_ms: 50 * MINUTE,
            elapsed_ms: 0,
            remaining_ms: (50 * MINUTE) as i64,
            overflowing: false,
            completed_in_cycle: 0,
            long_break_every: 4,
            pauses_left: Some(2),
            pauses: 0,
            started_at: None,
        }
    }

    fn view_default(s: &Snapshot) -> TrayView {
        view(s, Options::default())
    }

    fn alpha(rgba: &[u8], x: u32, y: u32) -> u8 {
        rgba[((y * ICON_SIZE + x) * 4 + 3) as usize]
    }

    #[test]
    fn idle_shows_the_logo_alone() {
        let v = view_default(&snap(Phase::Focus, Status::Idle));
        assert_eq!(v.title, None);
        assert_eq!(v.icon, Icon::Logo);
    }

    #[test]
    fn focus_counts_down_rounding_up() {
        let mut s = snap(Phase::Focus, Status::Running);
        s.elapsed_ms = 31 * MINUTE + 35_500;
        s.remaining_ms = (18 * MINUTE + 24_500) as i64;
        let v = view(&s, Options::default());
        assert_eq!(v.title.as_deref(), Some("18:25"));
        assert_eq!(
            v.icon,
            Icon::Ring {
                step: 37,
                dim: false
            }
        );
        assert_eq!(v.tooltip, "Halo: focusing");
    }

    #[test]
    fn overflow_counts_up_with_a_plus_and_a_full_ring() {
        let mut s = snap(Phase::Focus, Status::Running);
        s.elapsed_ms = 53 * MINUTE + 12_000;
        s.remaining_ms = -((3 * MINUTE + 12_000) as i64);
        s.overflowing = true;
        let v = view(&s, Options::default());
        assert_eq!(v.title.as_deref(), Some("+03:12"));
        assert_eq!(
            v.icon,
            Icon::Ring {
                step: 60,
                dim: false
            }
        );
    }

    #[test]
    fn flowtime_counts_up_and_hours_show() {
        let mut s = snap(Phase::Focus, Status::Running);
        s.kind = Kind::Flowtime;
        s.planned_ms = 0;
        s.elapsed_ms = 75 * MINUTE + 9_900;
        let v = view(&s, Options::default());
        assert_eq!(v.title.as_deref(), Some("1:15:09"));
        assert_eq!(
            v.icon,
            Icon::Ring {
                step: 15,
                dim: false
            }
        );
    }

    #[test]
    fn break_counts_down_and_pause_dims_the_ring() {
        let mut s = snap(Phase::ShortBreak, Status::Paused);
        s.planned_ms = 10 * MINUTE;
        s.elapsed_ms = 3 * MINUTE;
        s.remaining_ms = (7 * MINUTE) as i64;
        let v = view(&s, Options::default());
        assert_eq!(v.title.as_deref(), Some("07:00"));
        assert_eq!(
            v.icon,
            Icon::Ring {
                step: 18,
                dim: true
            }
        );
        assert_eq!(v.tooltip, "Halo: short break, paused");
    }

    #[test]
    fn icons_have_the_right_size() {
        for icon in [
            Icon::Logo,
            Icon::Ring {
                step: 30,
                dim: false,
            },
        ] {
            assert_eq!(
                render(icon, None).len(),
                (ICON_SIZE * ICON_SIZE * 4) as usize
            );
        }
    }

    #[test]
    fn ring_fills_clockwise_from_the_top() {
        // A quarter lap: the right side is solid, the bottom is only the faint track.
        let rgba = render(
            Icon::Ring {
                step: 15,
                dim: false,
            },
            None,
        );
        let right = alpha(&rgba, 18 + 12, 18);
        let bottom = alpha(&rgba, 18, 18 + 12);
        assert!(right > 200, "right {right}");
        assert!(bottom > 40 && bottom < 120, "bottom {bottom}");
        // Centre and corners stay clear.
        assert_eq!(alpha(&rgba, 18, 18), 0);
        assert_eq!(alpha(&rgba, 0, 0), 0);
    }

    #[test]
    fn logo_has_a_gap_and_a_dot_near_the_top() {
        let rgba = render(Icon::Logo, None);
        // Ring at the left and bottom.
        assert!(alpha(&rgba, 18 - 12, 18) > 200);
        assert!(alpha(&rgba, 18, 18 + 12) > 200);
        // The dot, inside the gap at the top right.
        assert!(alpha(&rgba, 21, 6) > 200);
        // Going right from the dot along that row: a clear gap, then the ring.
        let row: Vec<u8> = (21..ICON_SIZE).map(|x| alpha(&rgba, x, 6)).collect();
        let gap = row
            .iter()
            .position(|&a| a < 81)
            .expect("a gap after the dot");
        assert!(
            row[gap..].iter().any(|&a| a > 150),
            "the ring resumes after the gap: {row:?}"
        );
    }

    #[test]
    fn without_seconds_shows_minutes() {
        let mut s = snap(Phase::Focus, Status::Running);
        s.elapsed_ms = 31 * MINUTE + 36_000;
        s.remaining_ms = (18 * MINUTE + 24_000) as i64;
        let opts = Options {
            show_time: true,
            show_seconds: false,
        };
        assert_eq!(view(&s, opts).title.as_deref(), Some("19m"));

        s.overflowing = true;
        s.remaining_ms = -((3 * MINUTE + 40_000) as i64);
        assert_eq!(view(&s, opts).title.as_deref(), Some("+3m"));

        s.overflowing = false;
        s.kind = Kind::Flowtime;
        s.elapsed_ms = 65 * MINUTE;
        assert_eq!(view(&s, opts).title.as_deref(), Some("1h 05m"));
    }

    #[test]
    fn icon_only_hides_the_time_but_keeps_the_ring() {
        let mut s = snap(Phase::Focus, Status::Running);
        s.elapsed_ms = 25 * MINUTE;
        s.remaining_ms = (25 * MINUTE) as i64;
        let v = view(
            &s,
            Options {
                show_time: false,
                show_seconds: true,
            },
        );
        assert_eq!(v.title, None);
        assert_eq!(
            v.icon,
            Icon::Ring {
                step: 30,
                dim: false
            }
        );
    }

    #[test]
    fn a_tint_colours_the_icon() {
        let rgba = render(Icon::Logo, Some([242, 181, 68]));
        let i = ((18 * ICON_SIZE + 6) * 4) as usize;
        assert_eq!(&rgba[i..i + 4], &[242, 181, 68, 255]);
    }
}
