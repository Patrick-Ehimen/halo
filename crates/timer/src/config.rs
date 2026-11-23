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
    /// Ask "Still on it?" every this many ms of focus. `None` turns it off.
    pub check_in_every_ms: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            long_break_every: 4,
            overflow: true,
            auto_start_breaks: true,
            auto_start_focus: false,
            pause_limit: Some(2),
            check_in_every_ms: Some(15 * MINUTE),
        }
    }
}

/// Allowed range for [`Config::long_break_every`].
pub const LONG_BREAK_EVERY: std::ops::RangeInclusive<u32> = 2..=8;
/// Allowed check-in interval, in whole minutes.
pub const CHECK_IN_EVERY_MIN: std::ops::RangeInclusive<u64> = 5..=60;
/// Highest pause limit; above it, use unlimited (`None`).
pub const MAX_PAUSE_LIMIT: u32 = 5;

impl Config {
    /// Checks a config coming from settings before the engine uses it.
    pub fn validate(&self) -> crate::Result<()> {
        if !LONG_BREAK_EVERY.contains(&self.long_break_every) {
            return Err(crate::Error::Invalid(
                "Long break interval must be between 2 and 8 sessions",
            ));
        }
        if self
            .check_in_every_ms
            .is_some_and(|ms| !CHECK_IN_EVERY_MIN.contains(&(ms / MINUTE)) || ms % MINUTE != 0)
        {
            return Err(crate::Error::Invalid(
                "Check-ins must be every 5 to 60 minutes, or off",
            ));
        }
        if self.pause_limit.is_some_and(|n| n > MAX_PAUSE_LIMIT) {
            return Err(crate::Error::Invalid(
                "Pause limit must be 5 or less, or unlimited",
            ));
        }
        Ok(())
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
    fn default_config_is_valid() {
        assert_eq!(Config::default().validate(), Ok(()));
    }

    #[test]
    fn validate_rejects_out_of_range_settings() {
        let bad_interval = Config {
            long_break_every: 1,
            ..Config::default()
        };
        assert!(bad_interval.validate().is_err());
        let too_many_pauses = Config {
            pause_limit: Some(6),
            ..Config::default()
        };
        assert!(too_many_pauses.validate().is_err());
        for bad in [MINUTE, 61 * MINUTE, 15 * MINUTE + 1] {
            let c = Config {
                check_in_every_ms: Some(bad),
                ..Config::default()
            };
            assert!(c.validate().is_err(), "{bad}");
        }
        let off = Config {
            check_in_every_ms: None,
            ..Config::default()
        };
        assert_eq!(off.validate(), Ok(()));
        let unlimited = Config {
            pause_limit: None,
            long_break_every: 8,
            ..Config::default()
        };
        assert_eq!(unlimited.validate(), Ok(()));
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
