//! Repeating tasks: daily, weekdays, or weekly on chosen days.

use std::fmt;

use chrono::{Datelike, Days, NaiveDate, Weekday};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recurrence {
    Daily,
    /// Monday to Friday.
    Weekdays,
    /// Every week on these days (sorted, no duplicates).
    Weekly(Vec<Weekday>),
}

impl Recurrence {
    /// Weekly on the given days, sorted Monday first. `None` if empty.
    pub fn weekly(days: &[Weekday]) -> Option<Self> {
        let mut days: Vec<Weekday> = days.to_vec();
        days.sort_by_key(|d| d.num_days_from_monday());
        days.dedup();
        (!days.is_empty()).then_some(Recurrence::Weekly(days))
    }

    fn matches(&self, date: NaiveDate) -> bool {
        match self {
            Recurrence::Daily => true,
            Recurrence::Weekdays => date.weekday().num_days_from_monday() < 5,
            Recurrence::Weekly(days) => days.contains(&date.weekday()),
        }
    }

    /// The first matching date on or after `date`.
    pub fn on_or_after(&self, date: NaiveDate) -> NaiveDate {
        (0..7)
            .filter_map(|n| date.checked_add_days(Days::new(n)))
            .find(|d| self.matches(*d))
            .unwrap_or(date)
    }

    /// The next matching date strictly after `date`: where a finished
    /// repeating task comes back.
    pub fn next_after(&self, date: NaiveDate) -> NaiveDate {
        date.checked_add_days(Days::new(1))
            .map_or(date, |d| self.on_or_after(d))
    }

    /// Parses the stored form (`daily`, `weekdays`, `weekly:MO,WE`).
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "daily" => Some(Recurrence::Daily),
            "weekdays" => Some(Recurrence::Weekdays),
            _ => {
                let days = s.strip_prefix("weekly:")?;
                let parsed: Option<Vec<Weekday>> = days.split(',').map(code_day).collect();
                Recurrence::weekly(&parsed?)
            }
        }
    }

    /// Short words for the UI: "Repeats daily", "Repeats Mon, Wed".
    pub fn describe(&self) -> String {
        match self {
            Recurrence::Daily => "Repeats daily".into(),
            Recurrence::Weekdays => "Repeats weekdays".into(),
            Recurrence::Weekly(days) => {
                let names: Vec<&str> = days.iter().map(|d| short_name(*d)).collect();
                format!("Repeats {}", names.join(", "))
            }
        }
    }
}

/// The stored form: `daily`, `weekdays`, `weekly:MO,WE`.
impl fmt::Display for Recurrence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Recurrence::Daily => f.write_str("daily"),
            Recurrence::Weekdays => f.write_str("weekdays"),
            Recurrence::Weekly(days) => {
                let codes: Vec<&str> = days.iter().map(|d| day_code(*d)).collect();
                write!(f, "weekly:{}", codes.join(","))
            }
        }
    }
}

fn day_code(d: Weekday) -> &'static str {
    match d {
        Weekday::Mon => "MO",
        Weekday::Tue => "TU",
        Weekday::Wed => "WE",
        Weekday::Thu => "TH",
        Weekday::Fri => "FR",
        Weekday::Sat => "SA",
        Weekday::Sun => "SU",
    }
}

fn code_day(code: &str) -> Option<Weekday> {
    Some(match code {
        "MO" => Weekday::Mon,
        "TU" => Weekday::Tue,
        "WE" => Weekday::Wed,
        "TH" => Weekday::Thu,
        "FR" => Weekday::Fri,
        "SA" => Weekday::Sat,
        "SU" => Weekday::Sun,
        _ => return None,
    })
}

pub(crate) fn short_name(d: Weekday) -> &'static str {
    match d {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    // 2026-11-20 is a Friday.
    const FRI: (i32, u32, u32) = (2026, 11, 20);

    #[test]
    fn daily_comes_back_tomorrow() {
        let fri = d(FRI.0, FRI.1, FRI.2);
        assert_eq!(Recurrence::Daily.next_after(fri), d(2026, 11, 21));
    }

    #[test]
    fn weekdays_skip_the_weekend() {
        let fri = d(FRI.0, FRI.1, FRI.2);
        assert_eq!(Recurrence::Weekdays.next_after(fri), d(2026, 11, 23));
        assert_eq!(
            Recurrence::Weekdays.on_or_after(d(2026, 11, 21)),
            d(2026, 11, 23)
        );
    }

    #[test]
    fn weekly_finds_the_next_chosen_day() {
        let r = Recurrence::weekly(&[Weekday::Wed, Weekday::Mon]).unwrap();
        assert_eq!(r.next_after(d(2026, 11, 23)), d(2026, 11, 25)); // Mon -> Wed
        assert_eq!(r.next_after(d(2026, 11, 25)), d(2026, 11, 30)); // Wed -> Mon
    }

    #[test]
    fn crosses_month_and_year_ends() {
        assert_eq!(Recurrence::Daily.next_after(d(2026, 12, 31)), d(2027, 1, 1));
        let fri = Recurrence::weekly(&[Weekday::Fri]).unwrap();
        assert_eq!(fri.next_after(d(2027, 2, 26)), d(2027, 3, 5));
    }

    #[test]
    fn stored_form_round_trips() {
        for r in [
            Recurrence::Daily,
            Recurrence::Weekdays,
            Recurrence::weekly(&[Weekday::Sun, Weekday::Mon, Weekday::Mon]).unwrap(),
        ] {
            assert_eq!(Recurrence::parse(&r.to_string()), Some(r));
        }
        assert_eq!(
            Recurrence::weekly(&[Weekday::Wed, Weekday::Mon])
                .unwrap()
                .to_string(),
            "weekly:MO,WE"
        );
        assert_eq!(Recurrence::parse("weekly:XX"), None);
        assert_eq!(Recurrence::parse("weekly:"), None);
        assert_eq!(Recurrence::parse("hourly"), None);
    }

    #[test]
    fn describes_itself() {
        assert_eq!(Recurrence::Weekdays.describe(), "Repeats weekdays");
        let r = Recurrence::weekly(&[Weekday::Mon, Weekday::Thu]).unwrap();
        assert_eq!(r.describe(), "Repeats Mon, Thu");
    }
}
