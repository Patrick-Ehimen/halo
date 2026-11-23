//! Natural-language quick add:
//! "review PR tomorrow 10am ~2 sessions #meridian-rs".
//!
//! Understands:
//! - dates: today, tomorrow, weekday names ("fri" is the coming Friday, today
//!   included), "next fri" (Friday of next week), "next week" (next Monday),
//!   "21 nov", "nov 21", "21/11", "2026-11-21"
//! - times: 10am, 10:30am, 3pm, 14:00, noon, "at 3" (1 to 6 read as pm)
//! - estimates: ~2, ~2 sessions, ~2s
//! - project: #meridian-rs
//! - repeats: daily, weekdays, weekly, every day, every weekday,
//!   every mon and wed
//!
//! Whatever isn't understood is the title. Words like "at", "on" and "by"
//! are dropped only when they introduce a date or time.

use chrono::{Datelike, Days, NaiveDate, NaiveTime, Weekday};

use crate::recurrence::Recurrence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickAdd {
    pub title: String,
    pub planned_on: Option<NaiveDate>,
    pub due_time: Option<NaiveTime>,
    pub estimate: Option<u32>,
    /// The project name after `#`, as typed.
    pub project: Option<String>,
    pub recurrence: Option<Recurrence>,
}

/// Biggest estimate quick add accepts.
const MAX_ESTIMATE: u32 = 20;

pub fn parse(input: &str, today: NaiveDate) -> QuickAdd {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut used = vec![false; tokens.len()];
    let mut out = QuickAdd {
        title: String::new(),
        planned_on: None,
        due_time: None,
        estimate: None,
        project: None,
        recurrence: None,
    };
    let mut weekly_without_days = false;

    let word = |i: usize| -> String {
        tokens
            .get(i)
            .map(|t| t.trim_end_matches([',', '.', ';']).to_lowercase())
            .unwrap_or_default()
    };

    let mut i = 0;
    while i < tokens.len() {
        let w = word(i);

        // #project
        if let Some(name) = tokens[i].strip_prefix('#')
            && !name.is_empty()
            && out.project.is_none()
        {
            out.project = Some(name.trim_end_matches([',', '.']).to_string());
            used[i] = true;
            i += 1;
            continue;
        }

        // ~2, ~2s, ~2 sessions
        if let Some(rest) = w.strip_prefix('~') {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            let suffix = &rest[digits.len()..];
            if let Ok(n) = digits.parse::<u32>()
                && (1..=MAX_ESTIMATE).contains(&n)
                && matches!(suffix, "" | "s" | "session" | "sessions")
            {
                out.estimate = Some(n);
                used[i] = true;
                if suffix.is_empty() && matches!(word(i + 1).as_str(), "session" | "sessions") {
                    used[i + 1] = true;
                    i += 1;
                }
                i += 1;
                continue;
            }
        }

        // Repeats.
        match w.as_str() {
            "daily" => {
                out.recurrence = Some(Recurrence::Daily);
                used[i] = true;
                i += 1;
                continue;
            }
            "weekdays" => {
                out.recurrence = Some(Recurrence::Weekdays);
                used[i] = true;
                i += 1;
                continue;
            }
            "weekly" => {
                weekly_without_days = true;
                used[i] = true;
                i += 1;
                continue;
            }
            "every" => {
                let next = word(i + 1);
                if next == "day" {
                    out.recurrence = Some(Recurrence::Daily);
                    used[i] = true;
                    used[i + 1] = true;
                    i += 2;
                    continue;
                }
                if next == "weekday" || next == "weekdays" {
                    out.recurrence = Some(Recurrence::Weekdays);
                    used[i] = true;
                    used[i + 1] = true;
                    i += 2;
                    continue;
                }
                if next == "week" {
                    weekly_without_days = true;
                    used[i] = true;
                    used[i + 1] = true;
                    i += 2;
                    continue;
                }
                // every mon and wed / every mon, wed
                let mut days = Vec::new();
                let mut j = i + 1;
                while j < tokens.len() {
                    let t = word(j);
                    if let Some(d) = weekday(&t) {
                        days.push(d);
                        j += 1;
                    } else if (t == "and" || t == "&") && weekday(&word(j + 1)).is_some() {
                        j += 1;
                    } else {
                        break;
                    }
                }
                if let Some(r) = Recurrence::weekly(&days) {
                    out.recurrence = Some(r);
                    used[i..j].fill(true);
                    i = j;
                    continue;
                }
            }
            _ => {}
        }

        // Dates.
        if out.planned_on.is_none() {
            if let Some(date) = match w.as_str() {
                "today" | "tonight" => Some(today),
                "tomorrow" | "tmr" | "tmrw" => today.checked_add_days(Days::new(1)),
                _ => None,
            } {
                out.planned_on = Some(date);
                used[i] = true;
                i += 1;
                continue;
            }
            if w == "next" {
                let next = word(i + 1);
                let date = if next == "week" {
                    Some(next_monday(today))
                } else {
                    weekday(&next).and_then(|d| {
                        next_monday(today)
                            .checked_add_days(Days::new(u64::from(d.num_days_from_monday())))
                    })
                };
                if let Some(date) = date {
                    out.planned_on = Some(date);
                    used[i] = true;
                    used[i + 1] = true;
                    i += 2;
                    continue;
                }
            }
            if let Some(d) = weekday(&w) {
                out.planned_on = Some(coming(today, d));
                used[i] = true;
                i += 1;
                continue;
            }
            if let Some((date, len)) = calendar_date(&w, &word(i + 1), today) {
                out.planned_on = Some(date);
                used[i..i + len].fill(true);
                i += len;
                continue;
            }
        }

        // Times.
        if out.due_time.is_none() {
            if let Some((time, len)) = clock_time(&w, &word(i + 1)) {
                out.due_time = Some(time);
                used[i..i + len].fill(true);
                i += len;
                continue;
            }
            if w == "at"
                && let Ok(h) = word(i + 1).parse::<u32>()
                && (1..=12).contains(&h)
            {
                // "at 3": 1 to 6 are afternoon, the rest morning.
                let hour = if h <= 6 { h + 12 } else { h };
                out.due_time = NaiveTime::from_hms_opt(hour, 0, 0);
                used[i] = true;
                used[i + 1] = true;
                i += 2;
                continue;
            }
        }

        i += 1;
    }

    // Connectors go when they introduce something understood.
    for k in 0..tokens.len() {
        if !used[k]
            && matches!(word(k).as_str(), "at" | "on" | "by" | "for")
            && used.get(k + 1).copied().unwrap_or(false)
        {
            used[k] = true;
        }
    }

    if weekly_without_days && out.recurrence.is_none() {
        let day = out.planned_on.unwrap_or(today).weekday();
        out.recurrence = Recurrence::weekly(&[day]);
    }
    // A repeat with no date starts on its first matching day.
    if let (None, Some(r)) = (out.planned_on, &out.recurrence) {
        out.planned_on = Some(r.on_or_after(today));
    }
    // A time with no date is today.
    if out.due_time.is_some() && out.planned_on.is_none() {
        out.planned_on = Some(today);
    }

    let title: Vec<&str> = tokens
        .iter()
        .zip(&used)
        .filter(|(_, u)| !**u)
        .map(|(t, _)| *t)
        .collect();
    out.title = title.join(" ").trim().to_string();
    if out.title.is_empty() {
        out.title = input.trim().to_string();
    }
    out
}

fn weekday(w: &str) -> Option<Weekday> {
    Some(match w {
        "mon" | "monday" => Weekday::Mon,
        "tue" | "tues" | "tuesday" => Weekday::Tue,
        "wed" | "wednesday" => Weekday::Wed,
        "thu" | "thur" | "thurs" | "thursday" => Weekday::Thu,
        "fri" | "friday" => Weekday::Fri,
        "sat" | "saturday" => Weekday::Sat,
        "sun" | "sunday" => Weekday::Sun,
        _ => return None,
    })
}

/// The coming `day`, today included.
fn coming(today: NaiveDate, day: Weekday) -> NaiveDate {
    let ahead = (7 + day.num_days_from_monday() - today.weekday().num_days_from_monday()) % 7;
    today
        .checked_add_days(Days::new(u64::from(ahead)))
        .unwrap_or(today)
}

/// The first Monday strictly after today.
fn next_monday(today: NaiveDate) -> NaiveDate {
    let ahead = 7 - today.weekday().num_days_from_monday();
    today
        .checked_add_days(Days::new(u64::from(ahead)))
        .unwrap_or(today)
}

fn month(w: &str) -> Option<u32> {
    let m = [
        "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
    ];
    let w = w.get(..3)?;
    m.iter().position(|x| *x == w).map(|i| i as u32 + 1)
}

/// A month and day with no year: this year, or next if it has passed.
fn upcoming(today: NaiveDate, m: u32, d: u32) -> Option<NaiveDate> {
    let this = NaiveDate::from_ymd_opt(today.year(), m, d)?;
    if this >= today {
        Some(this)
    } else {
        NaiveDate::from_ymd_opt(today.year() + 1, m, d)
    }
}

/// 2026-11-21, 21/11, 21/11/2026, "21 nov", "nov 21". Returns tokens used.
fn calendar_date(w: &str, next: &str, today: NaiveDate) -> Option<(NaiveDate, usize)> {
    if let Ok(d) = NaiveDate::parse_from_str(w, "%Y-%m-%d") {
        return Some((d, 1));
    }
    let parts: Vec<&str> = w.split('/').collect();
    if (2..=3).contains(&parts.len()) && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
    {
        let d: u32 = parts[0].parse().ok()?;
        let m: u32 = parts[1].parse().ok()?;
        let date = match parts.get(2) {
            Some(y) => {
                let y: i32 = y.parse().ok()?;
                let y = if y < 100 { 2000 + y } else { y };
                NaiveDate::from_ymd_opt(y, m, d)?
            }
            None => upcoming(today, m, d)?,
        };
        return Some((date, 1));
    }
    let day = |s: &str| -> Option<u32> {
        let digits = s.trim_end_matches(|c: char| c.is_ascii_alphabetic());
        digits.parse().ok().filter(|d| (1..=31).contains(d))
    };
    if let (Some(d), Some(m)) = (day(w), month(next)) {
        return upcoming(today, m, d).map(|date| (date, 2));
    }
    if let (Some(m), Some(d)) = (month(w), day(next))
        && w.len() >= 3
    {
        return upcoming(today, m, d).map(|date| (date, 2));
    }
    None
}

/// 10am, 10:30am, 10.30pm, 14:00, noon, "10 am". Returns tokens used.
fn clock_time(w: &str, next: &str) -> Option<(NaiveTime, usize)> {
    if w == "noon" {
        return NaiveTime::from_hms_opt(12, 0, 0).map(|t| (t, 1));
    }
    let (body, meridiem, len) = if let Some(b) = w.strip_suffix("am") {
        (b, Some(false), 1)
    } else if let Some(b) = w.strip_suffix("pm") {
        (b, Some(true), 1)
    } else if next == "am" || next == "pm" {
        (w, Some(next == "pm"), 2)
    } else {
        (w, None, 1)
    };
    if body.is_empty() {
        return None;
    }
    let (h, m) = match body.split_once([':', '.']) {
        Some((h, m)) => (h.parse::<u32>().ok()?, m.parse::<u32>().ok()?),
        None if meridiem.is_some() => (body.parse::<u32>().ok()?, 0),
        // A bare number isn't a time ("buy 2 books").
        None => return None,
    };
    let hour = match meridiem {
        Some(pm) => {
            if !(1..=12).contains(&h) {
                return None;
            }
            match (h, pm) {
                (12, false) => 0,
                (12, true) => 12,
                (h, true) => h + 12,
                (h, false) => h,
            }
        }
        None => h,
    };
    NaiveTime::from_hms_opt(hour, m, 0).map(|t| (t, len))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Saturday 21 November 2026.
    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 21).unwrap()
    }

    fn d(m: u32, day: u32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(2026, m, day)
    }

    fn t(h: u32, m: u32) -> Option<NaiveTime> {
        NaiveTime::from_hms_opt(h, m, 0)
    }

    #[test]
    fn the_example_from_the_spec() {
        let q = parse("review PR tomorrow 10am ~2 sessions #meridian-rs", today());
        assert_eq!(q.title, "review PR");
        assert_eq!(q.planned_on, d(11, 22));
        assert_eq!(q.due_time, t(10, 0));
        assert_eq!(q.estimate, Some(2));
        assert_eq!(q.project.as_deref(), Some("meridian-rs"));
        assert_eq!(q.recurrence, None);
    }

    #[test]
    fn dates() {
        let cases: &[(&str, Option<NaiveDate>)] = &[
            ("a today", d(11, 21)),
            ("a tmr", d(11, 22)),
            ("a sat", d(11, 21)), // the coming Saturday is today
            ("a mon", d(11, 23)),
            ("a friday", d(11, 27)),
            ("a next week", d(11, 23)), // next Monday
            ("a next fri", d(11, 27)),  // Friday of next week
            ("a on 25 nov", d(11, 25)),
            ("a nov 25", d(11, 25)),
            ("a 3rd dec", d(12, 3)),
            ("a 25/11", d(11, 25)),
            ("a 2026-12-01", d(12, 1)),
            ("a 1 jan", NaiveDate::from_ymd_opt(2027, 1, 1)), // passed: next year
            ("a 10/11", NaiveDate::from_ymd_opt(2027, 11, 10)),
            ("a 1/2/27", NaiveDate::from_ymd_opt(2027, 2, 1)),
        ];
        for (input, want) in cases {
            let q = parse(input, today());
            assert_eq!(q.planned_on, *want, "{input}");
            assert_eq!(q.title, "a", "{input}");
        }
    }

    #[test]
    fn times() {
        let cases: &[(&str, Option<NaiveTime>)] = &[
            ("a 10am", t(10, 0)),
            ("a 10:30am", t(10, 30)),
            ("a 3pm", t(15, 0)),
            ("a 3 pm", t(15, 0)),
            ("a 10.15pm", t(22, 15)),
            ("a 12am", t(0, 0)),
            ("a 12pm", t(12, 0)),
            ("a 14:00", t(14, 0)),
            ("a at noon", t(12, 0)),
            ("a at 3", t(15, 0)),
            ("a at 9", t(9, 0)),
        ];
        for (input, want) in cases {
            let q = parse(input, today());
            assert_eq!(q.due_time, *want, "{input}");
            assert_eq!(q.planned_on, d(11, 21), "a time alone means today: {input}");
            assert_eq!(q.title, "a", "{input}");
        }
    }

    #[test]
    fn estimates_and_projects() {
        let cases: &[(&str, Option<u32>, Option<&str>, &str)] = &[
            ("write ~3", Some(3), None, "write"),
            ("write ~3s", Some(3), None, "write"),
            ("write ~1 session", Some(1), None, "write"),
            ("write ~0", None, None, "write ~0"),
            ("write ~99", None, None, "write ~99"),
            ("write #Job-hunt", None, Some("Job-hunt"), "write"),
            (
                "#open-source triage issues",
                None,
                Some("open-source"),
                "triage issues",
            ),
        ];
        for (input, est, proj, title) in cases {
            let q = parse(input, today());
            assert_eq!(q.estimate, *est, "{input}");
            assert_eq!(q.project.as_deref(), *proj, "{input}");
            assert_eq!(q.title, *title, "{input}");
        }
    }

    #[test]
    fn repeats() {
        let q = parse("journal trades daily 8:30", today());
        assert_eq!(q.recurrence, Some(Recurrence::Daily));
        assert_eq!(q.due_time, t(8, 30));
        assert_eq!(q.planned_on, d(11, 21));
        assert_eq!(q.title, "journal trades");

        let q = parse("send DMs every weekday", today());
        assert_eq!(q.recurrence, Some(Recurrence::Weekdays));
        assert_eq!(q.planned_on, d(11, 23), "starts on the first weekday");
        assert_eq!(q.title, "send DMs");

        let q = parse("gym every mon and thu", today());
        assert_eq!(
            q.recurrence,
            Recurrence::weekly(&[Weekday::Mon, Weekday::Thu])
        );
        assert_eq!(q.planned_on, d(11, 23));
        assert_eq!(q.title, "gym");

        let q = parse("review week weekly fri", today());
        assert_eq!(q.recurrence, Recurrence::weekly(&[Weekday::Fri]));
        assert_eq!(q.planned_on, d(11, 27));
        assert_eq!(q.title, "review week");

        assert_eq!(
            parse("plan every day", today()).recurrence,
            Some(Recurrence::Daily)
        );
    }

    #[test]
    fn leaves_ordinary_words_alone() {
        let q = parse("buy 2 books for the meetup", today());
        assert_eq!(q.title, "buy 2 books for the meetup");
        assert_eq!(q.planned_on, None);
        assert_eq!(q.due_time, None);

        let q = parse("call mom on the way home", today());
        assert_eq!(q.title, "call mom on the way home");

        let q = parse("read every chapter", today());
        assert_eq!(q.title, "read every chapter");
        assert_eq!(q.recurrence, None);
    }

    #[test]
    fn empty_and_nothing_left() {
        assert_eq!(parse("   ", today()).title, "");
        let q = parse("tomorrow", today());
        assert_eq!(q.title, "tomorrow");
        assert_eq!(q.planned_on, d(11, 22));
    }
}
