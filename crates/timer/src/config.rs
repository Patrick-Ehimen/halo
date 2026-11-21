/// One second, in the engine's unit (milliseconds).
pub const SECOND: u64 = 1_000;
/// One minute, in milliseconds.
pub const MINUTE: u64 = 60 * SECOND;

/// What kind of focus session this is. Stored on every session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Pomodoro,
    Deep,
    Flowtime,
    Custom,
}

impl Kind {
    /// Stable string used in storage and IPC.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Pomodoro => "pomodoro",
            Kind::Deep => "deep",
            Kind::Flowtime => "flowtime",
            Kind::Custom => "custom",
        }
    }
}

/// Lengths for one focus session and the breaks that follow it.
///
/// Flowtime has no goal: `focus_ms` is 0 and the break is earned from the
/// time actually focused (see [`flowtime_break_ms`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preset {
    pub kind: Kind,
    pub focus_ms: u64,
    pub short_break_ms: u64,
    pub long_break_ms: u64,
}

impl Preset {
    /// Pomodoro 25/5, 15 min long break.
    pub const POMODORO: Preset = Preset {
        kind: Kind::Pomodoro,
        focus_ms: 25 * MINUTE,
        short_break_ms: 5 * MINUTE,
        long_break_ms: 15 * MINUTE,
    };

    /// Deep 50/10, 20 min long break.
    pub const DEEP: Preset = Preset {
        kind: Kind::Deep,
        focus_ms: 50 * MINUTE,
        short_break_ms: 10 * MINUTE,
        long_break_ms: 20 * MINUTE,
    };

    /// Flowtime: a stopwatch; the break is earned from the time focused.
    pub const FLOWTIME: Preset = Preset {
        kind: Kind::Flowtime,
        focus_ms: 0,
        short_break_ms: 0,
        long_break_ms: 0,
    };

    /// A user-defined preset. Lengths are whole minutes.
    pub fn custom(focus_min: u64, short_break_min: u64, long_break_min: u64) -> Preset {
        Preset {
            kind: Kind::Custom,
            focus_ms: focus_min * MINUTE,
            short_break_ms: short_break_min * MINUTE,
            long_break_ms: long_break_min * MINUTE,
        }
    }

    pub fn is_flowtime(&self) -> bool {
        self.kind == Kind::Flowtime
    }
}

/// Timer behaviour the user can change in settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// A long break replaces the short one after every N completed sessions.
    pub long_break_every: u32,
    /// Keep counting past the goal instead of ending the session there.
    pub overflow: bool,
    /// Start the break as soon as a session completes.
    pub auto_start_breaks: bool,
    /// Start the next focus session as soon as a break ends.
    pub auto_start_focus: bool,
    /// Pauses allowed per focus session. `None` means unlimited.
    pub pause_limit: Option<u32>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            long_break_every: 4,
            overflow: true,
            auto_start_breaks: true,
            auto_start_focus: false,
            pause_limit: Some(2),
        }
    }
}

/// The break earned by a Flowtime session: about a fifth of the time focused,
/// rounded to the nearest minute, and at least one minute.
pub fn flowtime_break_ms(focused_ms: u64) -> u64 {
    let minutes = (focused_ms / 5 + MINUTE / 2) / MINUTE;
    minutes.max(1) * MINUTE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flowtime_break_is_a_fifth_rounded_to_minutes() {
        assert_eq!(flowtime_break_ms(50 * MINUTE), 10 * MINUTE);
        assert_eq!(flowtime_break_ms(32 * MINUTE), 6 * MINUTE); // 6.4 -> 6
        assert_eq!(flowtime_break_ms(33 * MINUTE), 7 * MINUTE); // 6.6 -> 7
    }

    #[test]
    fn flowtime_break_is_at_least_a_minute() {
        assert_eq!(flowtime_break_ms(0), MINUTE);
        assert_eq!(flowtime_break_ms(2 * MINUTE), MINUTE);
    }

    #[test]
    fn custom_preset_uses_minutes() {
        let p = Preset::custom(40, 8, 25);
        assert_eq!(p.kind, Kind::Custom);
        assert_eq!(p.focus_ms, 40 * MINUTE);
        assert_eq!(p.short_break_ms, 8 * MINUTE);
        assert_eq!(p.long_break_ms, 25 * MINUTE);
    }
}
