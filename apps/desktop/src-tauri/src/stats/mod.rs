//! Stats for the app: the Stats screen, today at a glance, and export. The
//! maths lives in `halo-stats`; this module reads history and formats labels
//! in the user's time zone.

mod dto;

use std::collections::{BTreeSet, HashMap};

use chrono::{Datelike, Days, Local, Months, NaiveDate, TimeZone};
use halo_plan::{day_bounds, local_minute};
use halo_stats::{Session, by_day, by_hour, focus_score, heat_level, streak, tree_stage};
use halo_store::Connection;
use halo_store::sessions::HistoryRow;
use tauri::{AppHandle, Manager};

use dto::{AchievementView, GroveDay, HeatDay, Kpi, ProjectTime};
pub use dto::{ReflectionView, StatsRange, StatsView, TodayFocus};

use crate::error::{Error, Result};
use crate::state::AppState;
use crate::timer::now_ms;

const HEATMAP_WEEKS: u64 = 24;
const GROVE_DAYS: u64 = 14;

fn now_i64() -> i64 {
    i64::try_from(now_ms()).unwrap_or(i64::MAX)
}

fn goal_secs(app: &AppHandle) -> u64 {
    u64::from(crate::prefs::get(app).daily_goal_minutes) * 60
}

fn to_session(r: &HistoryRow) -> Session {
    Session {
        started_at: r.started_at,
        ended_at: r.ended_at,
        focused_secs: r.focused_secs,
        planned_secs: r.planned_secs,
        rating: r.rating,
        distractions: r.distractions,
        strict: r.strict,
        project_id: r.project_id.clone(),
    }
}

fn today_in<Tz: TimeZone>(tz: &Tz, now: i64) -> Result<NaiveDate> {
    local_minute(tz, now)
        .map(|(d, _)| d)
        .ok_or(Error::Invalid("Bad clock time"))
}

// ---------------------------------------------------------------- today

pub fn today(app: &AppHandle) -> Result<TodayFocus> {
    let state = app.state::<AppState>();
    let conn = state.db()?;
    today_in_tz(&conn, &Local, now_i64(), goal_secs(app))
}

fn today_in_tz<Tz: TimeZone>(
    conn: &Connection,
    tz: &Tz,
    now: i64,
    goal: u64,
) -> Result<TodayFocus> {
    let today = today_in(tz, now)?;
    let history = halo_store::sessions::history(conn)?;
    let sessions: Vec<Session> = history.iter().map(to_session).collect();
    let focused = by_day(tz, &sessions, today, today)
        .get(&today)
        .copied()
        .unwrap_or(0);
    let count = history
        .iter()
        .filter(|r| local_date(tz, r.started_at) == Some(today))
        .count();
    let s = streak(&active_days(tz, &history), today);
    Ok(TodayFocus {
        focused_secs: u32::try_from(focused).unwrap_or(u32::MAX),
        sessions: u32::try_from(count).unwrap_or(u32::MAX),
        focused_label: duration(focused),
        goal_label: format!("of your {} goal", goal_duration(goal)),
        goal_permille: permille(focused, goal),
        tree: tree_stage(focused, goal).into(),
        streak: s.current,
        streak_longest: s.longest,
        streak_today: s.today_done,
    })
}

fn active_days<Tz: TimeZone>(tz: &Tz, history: &[HistoryRow]) -> BTreeSet<NaiveDate> {
    history
        .iter()
        .filter_map(|r| local_date(tz, r.started_at))
        .collect()
}

fn local_date<Tz: TimeZone>(tz: &Tz, ms: i64) -> Option<NaiveDate> {
    local_minute(tz, ms).map(|(d, _)| d)
}

// ---------------------------------------------------------------- stats view

pub fn view(app: &AppHandle, range: StatsRange) -> Result<StatsView> {
    let state = app.state::<AppState>();
    let conn = state.db()?;
    view_in(&conn, &Local, now_i64(), goal_secs(app), range)
}

/// The first and last day of the range containing `today`, and of the one before.
fn bounds(range: StatsRange, today: NaiveDate) -> ((NaiveDate, NaiveDate), (NaiveDate, NaiveDate)) {
    let back = |d: NaiveDate, n: u64| d.checked_sub_days(Days::new(n)).unwrap_or(d);
    let ahead = |d: NaiveDate, n: u64| d.checked_add_days(Days::new(n)).unwrap_or(d);
    match range {
        StatsRange::Day => ((today, today), (back(today, 1), back(today, 1))),
        StatsRange::Week => {
            let monday = back(today, u64::from(today.weekday().num_days_from_monday()));
            (
                (monday, ahead(monday, 6)),
                (back(monday, 7), back(monday, 1)),
            )
        }
        StatsRange::Month => {
            let first = today.with_day(1).unwrap_or(today);
            let next = first.checked_add_months(Months::new(1)).unwrap_or(first);
            let prev = first.checked_sub_months(Months::new(1)).unwrap_or(first);
            ((first, back(next, 1)), (prev, back(first, 1)))
        }
        StatsRange::Year => {
            let y = today.year();
            let jan = |y| NaiveDate::from_ymd_opt(y, 1, 1).unwrap_or(today);
            let dec = |y| NaiveDate::from_ymd_opt(y, 12, 31).unwrap_or(today);
            ((jan(y), dec(y)), (jan(y - 1), dec(y - 1)))
        }
    }
}

struct Summary {
    secs: u64,
    sessions: Vec<Session>,
    score: u32,
    distractions: u32,
}

fn summarize<Tz: TimeZone>(
    conn: &Connection,
    tz: &Tz,
    all: &[Session],
    (from, to): (NaiveDate, NaiveDate),
    today: NaiveDate,
    goal: u64,
) -> Result<Summary> {
    let last = to.min(today);
    // Days still to come don't count against the score.
    let daily: Vec<u64> = if from > last {
        Vec::new()
    } else {
        by_day(tz, all, from, last).into_values().collect()
    };
    let in_range: Vec<Session> = all
        .iter()
        .filter(|s| local_date(tz, s.started_at).is_some_and(|d| d >= from && d <= to))
        .cloned()
        .collect();
    let (start_ms, _) = day_bounds(tz, from);
    let (_, end_ms) = day_bounds(tz, to);
    let (focused, total) = halo_store::check_ins::counts_between(conn, start_ms, end_ms)?;
    let (score, _) = focus_score(&in_range, &daily, goal, (focused, total));
    Ok(Summary {
        secs: daily.iter().sum(),
        distractions: in_range.iter().map(|s| s.distractions).sum(),
        sessions: in_range,
        score,
    })
}

fn view_in<Tz: TimeZone>(
    conn: &Connection,
    tz: &Tz,
    now: i64,
    goal: u64,
    range: StatsRange,
) -> Result<StatsView> {
    let today = today_in(tz, now)?;
    let history = halo_store::sessions::history(conn)?;
    let all: Vec<Session> = history.iter().map(to_session).collect();
    let (cur_range, prev_range) = bounds(range, today);
    let cur = summarize(conn, tz, &all, cur_range, today, goal)?;
    let prev = summarize(conn, tz, &all, prev_range, today, goal)?;
    let vs = match range {
        StatsRange::Day => "vs yesterday",
        StatsRange::Week => "vs last week",
        StatsRange::Month => "vs last month",
        StatsRange::Year => "vs last year",
    };

    let rated: Vec<u8> = cur.sessions.iter().filter_map(|s| s.rating).collect();
    let kpis = vec![
        Kpi {
            label: "Focus score".into(),
            value: cur.score.to_string(),
            detail: count_delta(i64::from(cur.score) - i64::from(prev.score), vs),
            good: cur.score > prev.score,
        },
        Kpi {
            label: match range {
                StatsRange::Day => "Focused today",
                StatsRange::Week => "Focused this week",
                StatsRange::Month => "Focused this month",
                StatsRange::Year => "Focused this year",
            }
            .into(),
            value: duration(cur.secs),
            detail: duration_delta(cur.secs, prev.secs, vs),
            good: cur.secs > prev.secs,
        },
        Kpi {
            label: "Sessions".into(),
            value: cur.sessions.len().to_string(),
            detail: if rated.is_empty() {
                "Not rated yet".into()
            } else {
                let avg = f64::from(rated.iter().map(|&r| u32::from(r)).sum::<u32>())
                    / rated.len() as f64;
                format!("Average rating {} ({avg:.1} of 4)", rating_word(avg))
            },
            good: false,
        },
        Kpi {
            label: "Distractions".into(),
            value: cur.distractions.to_string(),
            detail: count_delta(
                i64::from(cur.distractions) - i64::from(prev.distractions),
                vs,
            ),
            good: cur.distractions < prev.distractions,
        },
    ];

    // Heatmap: 24 weeks ending with this week.
    let this_monday = today
        .checked_sub_days(Days::new(u64::from(today.weekday().num_days_from_monday())))
        .unwrap_or(today);
    let first = this_monday
        .checked_sub_days(Days::new((HEATMAP_WEEKS - 1) * 7))
        .unwrap_or(this_monday);
    let heat_days = by_day(tz, &all, first, today);
    let heatmap = (0..HEATMAP_WEEKS)
        .map(|w| {
            (0..7)
                .map(|d| {
                    let date = first
                        .checked_add_days(Days::new(w * 7 + d))
                        .unwrap_or(first);
                    let secs = heat_days.get(&date).copied();
                    HeatDay {
                        label: format!(
                            "{} · {}",
                            date.format("%a %-d %b"),
                            duration(secs.unwrap_or(0))
                        ),
                        level: secs.map(|s| heat_level(s, goal)),
                    }
                })
                .collect()
        })
        .collect();

    // Peak hours over the range.
    let hours_secs = by_hour(tz, &cur.sessions);
    let hours: Vec<u32> = hours_secs
        .iter()
        .map(|s| u32::try_from(s / 60).unwrap_or(u32::MAX))
        .collect();
    let best_hours = (0..23)
        .map(|h| (h, hours_secs[h] + hours_secs[h + 1]))
        .max_by_key(|&(h, s)| (s, std::cmp::Reverse(h)))
        .filter(|&(_, s)| s > 0)
        .map(|(h, _)| format!("{h:02}:00–{:02}:00", h + 2));

    // Time by project.
    let projects = halo_store::projects::list(conn)?;
    let names: HashMap<&str, (&str, &str)> = projects
        .iter()
        .map(|p| (p.id.as_str(), (p.name.as_str(), p.color.as_str())))
        .collect();
    let mut by_project: HashMap<Option<String>, u64> = HashMap::new();
    for s in &cur.sessions {
        *by_project.entry(s.project_id.clone()).or_default() += u64::from(s.focused_secs);
    }
    let project_total: u64 = by_project.values().sum();
    let mut project_times: Vec<(u64, ProjectTime)> = by_project
        .into_iter()
        .map(|(id, secs)| {
            let (name, color) = id
                .as_deref()
                .and_then(|i| names.get(i).copied())
                .unwrap_or(("No project", "#8A8E9E"));
            (
                secs,
                ProjectTime {
                    name: name.to_string(),
                    color: color.to_string(),
                    label: duration(secs),
                    permille: permille(secs, project_total),
                },
            )
        })
        .collect();
    project_times.sort_by_key(|p| std::cmp::Reverse(p.0));

    // Achievements over all of history.
    let goals_done: Vec<NaiveDate> = halo_store::goals::done_times(conn)?
        .into_iter()
        .filter_map(|ms| local_date(tz, ms))
        .collect();
    let achievements = halo_stats::achievements(tz, &all, &goals_done)
        .into_iter()
        .map(|a| AchievementView {
            status: match a.earned_on {
                Some(d) => format!("Earned {}", d.format("%-d %b %Y")),
                None if a.id == "deep-diver" => {
                    format!("Best day {}", duration(u64::from(a.progress) * 60))
                }
                None => format!("{} of {}", a.progress, a.target),
            },
            earned: a.earned_on.is_some(),
            id: a.id.to_string(),
            name: a.name.to_string(),
            description: a.description.to_string(),
            progress: a.progress,
            target: a.target,
        })
        .collect();

    // The grove: a tree for each of the last 14 days.
    let grove_from = today
        .checked_sub_days(Days::new(GROVE_DAYS - 1))
        .unwrap_or(today);
    let grove = by_day(tz, &all, grove_from, today)
        .into_iter()
        .map(|(date, secs)| GroveDay {
            label: date.format("%a").to_string(),
            stage: tree_stage(secs, goal).into(),
            today: date == today,
        })
        .collect();

    Ok(StatsView {
        range_label: range_label(range, cur_range, today),
        kpis,
        heatmap,
        hours,
        best_hours,
        projects: project_times.into_iter().map(|(_, p)| p).collect(),
        achievements,
        grove,
    })
}

fn range_label(range: StatsRange, (from, to): (NaiveDate, NaiveDate), today: NaiveDate) -> String {
    match range {
        StatsRange::Day => format!("Today, {}", today.format("%-d %b")),
        StatsRange::Week if from.month() == to.month() => {
            format!("{}–{}", from.format("%-d"), to.format("%-d %b"))
        }
        StatsRange::Week => format!("{} – {}", from.format("%-d %b"), to.format("%-d %b")),
        StatsRange::Month => from.format("%B %Y").to_string(),
        StatsRange::Year => from.format("%Y").to_string(),
    }
}

fn rating_word(avg: f64) -> &'static str {
    match avg {
        a if a >= 3.5 => "in flow",
        a if a >= 2.5 => "good",
        a if a >= 1.5 => "okay",
        _ => "rough",
    }
}

/// "21h 40m", "45m", "0m".
pub fn duration(secs: u64) -> String {
    let mins = secs / 60;
    if mins >= 60 {
        format!("{}h {:02}m", mins / 60, mins % 60)
    } else {
        format!("{mins}m")
    }
}

/// A daily goal: "5h", "4h 30m", "45m".
fn goal_duration(secs: u64) -> String {
    let mins = secs / 60;
    match (mins / 60, mins % 60) {
        (0, m) => format!("{m}m"),
        (h, 0) => format!("{h}h"),
        (h, m) => format!("{h}h {m:02}m"),
    }
}

/// "+3h 05m vs last week", "−20m vs yesterday", "Same as last week".
fn duration_delta(cur: u64, prev: u64, vs: &str) -> String {
    match cur.cmp(&prev) {
        std::cmp::Ordering::Equal => format!("Same as {}", vs.trim_start_matches("vs ")),
        std::cmp::Ordering::Greater => format!("+{} {vs}", duration(cur - prev)),
        std::cmp::Ordering::Less => format!("−{} {vs}", duration(prev - cur)),
    }
}

/// "+6 vs last week", "−12 vs last week".
fn count_delta(d: i64, vs: &str) -> String {
    match d {
        0 => format!("Same as {}", vs.trim_start_matches("vs ")),
        d if d > 0 => format!("+{d} {vs}"),
        d => format!("−{} {vs}", -d),
    }
}

fn permille(part: u64, whole: u64) -> u32 {
    (part.min(whole) * 1000)
        .checked_div(whole)
        .map_or(0, |p| u32::try_from(p).unwrap_or(1000))
}

// ---------------------------------------------------------------- export

/// Writes every session to Downloads as CSV or JSON. Returns the path.
pub fn export(app: &AppHandle, json: bool) -> Result<String> {
    let dir = app.path().download_dir()?;
    let history = {
        let state = app.state::<AppState>();
        let conn = state.db()?;
        let projects = halo_store::projects::list(&conn)?;
        (halo_store::sessions::history(&conn)?, projects)
    };
    let today = Local::now().date_naive();
    let name = format!(
        "halo-sessions-{}.{}",
        today.format("%Y-%m-%d"),
        if json { "json" } else { "csv" }
    );
    let path = dir.join(name);
    let body = render_export(&Local, &history.0, &history.1, json)?;
    std::fs::write(&path, body)?;
    Ok(path.display().to_string())
}

const EXPORT_COLUMNS: [&str; 15] = [
    "date",
    "start",
    "end",
    "kind",
    "source",
    "intention",
    "project",
    "focused_minutes",
    "planned_minutes",
    "overflow_minutes",
    "pauses",
    "rating",
    "distractions",
    "notes",
    "id",
];

fn render_export<Tz: TimeZone>(
    tz: &Tz,
    history: &[HistoryRow],
    projects: &[halo_store::projects::Project],
    json: bool,
) -> Result<String> {
    let names: HashMap<&str, &str> = projects
        .iter()
        .map(|p| (p.id.as_str(), p.name.as_str()))
        .collect();
    let local = |ms: i64| {
        tz.timestamp_millis_opt(ms)
            .single()
            .map(|t| t.naive_local())
    };
    let rows: Vec<Vec<String>> = history
        .iter()
        .map(|r| {
            let start = local(r.started_at);
            vec![
                start
                    .map(|t| t.format("%Y-%m-%d").to_string())
                    .unwrap_or_default(),
                start
                    .map(|t| t.format("%H:%M").to_string())
                    .unwrap_or_default(),
                local(r.ended_at)
                    .map(|t| t.format("%H:%M").to_string())
                    .unwrap_or_default(),
                r.kind.clone(),
                r.source.clone(),
                r.intention.clone(),
                r.project_id
                    .as_deref()
                    .and_then(|p| names.get(p).copied())
                    .unwrap_or("")
                    .to_string(),
                (r.focused_secs / 60).to_string(),
                (r.planned_secs / 60).to_string(),
                (r.overflow_secs / 60).to_string(),
                r.pauses.to_string(),
                r.rating.map(|x| x.to_string()).unwrap_or_default(),
                r.distractions.to_string(),
                r.notes.clone(),
                r.id.clone(),
            ]
        })
        .collect();
    if !json {
        return Ok(halo_stats::csv(&EXPORT_COLUMNS, &rows));
    }
    let objects: Vec<serde_json::Map<String, serde_json::Value>> = rows
        .into_iter()
        .map(|row| {
            EXPORT_COLUMNS
                .iter()
                .zip(row)
                .map(|(k, v)| {
                    let value = match *k {
                        "focused_minutes" | "planned_minutes" | "overflow_minutes" | "pauses"
                        | "distractions" | "rating" => v
                            .parse::<u64>()
                            .map_or(serde_json::Value::Null, serde_json::Value::from),
                        _ => serde_json::Value::from(v),
                    };
                    ((*k).to_string(), value)
                })
                .collect()
        })
        .collect();
    Ok(serde_json::to_string_pretty(&objects)?)
}

// ---------------------------------------------------------------- reflection

/// The reflection card's content for the last saved session, if there is one.
pub fn reflection(app: &AppHandle) -> Result<Option<ReflectionView>> {
    let state = app.state::<AppState>();
    let last = crate::timer::last_session(&state)?;
    let break_label = last.pending_break;
    let Some((id, intention)) = last.session else {
        return Ok(None);
    };
    let conn = state.db()?;
    Ok(
        halo_store::sessions::reflection_basis(&conn, &id)?.map(|(secs, distractions, notes)| {
            ReflectionView {
                focused_label: clock(u64::from(secs)),
                intention,
                distractions,
                notes,
                break_label,
            }
        }),
    )
}

/// Saves the reflection on the last session, closes the card, and starts the
/// waiting break if asked.
pub fn reflect(
    app: &AppHandle,
    rating: Option<u8>,
    distractions: u32,
    notes: &str,
    start_break: bool,
) -> Result<()> {
    if rating.is_some_and(|r| !(1..=4).contains(&r)) {
        return Err(Error::Invalid("Pick Rough, Okay, Good or In flow"));
    }
    if distractions > 999 {
        return Err(Error::Invalid("Count up to 999 distractions"));
    }
    let notes: String = notes.trim().chars().take(2000).collect();
    {
        let state = app.state::<AppState>();
        if let Some((id, _)) = crate::timer::last_session(&state)?.session {
            let conn = state.db()?;
            halo_store::sessions::reflect(&conn, &id, rating, distractions, &notes, now_i64())?;
        }
    }
    crate::windows::hide_reflection(app)?;
    if start_break {
        let last = crate::timer::last_session(&app.state::<AppState>())?;
        if last.pending_break.is_some() {
            crate::timer::start_pending(app)?;
        }
    }
    // Stats and today's ring include the rating now.
    crate::plan::changed(app);
    Ok(())
}

/// "50:00", "1:05:00".
fn clock(secs: u64) -> String {
    let (h, m, s) = (secs / 3600, secs % 3600 / 60, secs % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use chrono::NaiveTime;
    use chrono_tz::Africa::Lagos;
    use halo_plan::at_local;

    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    /// Monday 23 Nov 2026, 18:00 in Lagos.
    fn now() -> i64 {
        at_local(
            &Lagos,
            d(2026, 11, 23),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
    }

    fn db() -> Connection {
        halo_store::open(Path::new(":memory:")).unwrap()
    }

    /// A finished session at `h:00` local on `date`, `mins` long.
    fn session(conn: &Connection, date: NaiveDate, h: u32, mins: u32, task: Option<&str>) {
        let start = at_local(&Lagos, date, NaiveTime::from_hms_opt(h, 0, 0).unwrap());
        let id = halo_store::sessions::start(
            conn,
            &halo_store::sessions::StartedSession {
                kind: "deep",
                planned_secs: 3000,
                started_at: start,
                intention: "Write tests",
                task_id: task,
                project_id: None,
            },
            start,
        )
        .unwrap();
        let progress = halo_store::sessions::Progress {
            planned_secs: 3000,
            focused_secs: mins * 60,
            overflow_secs: 0,
            pauses: 0,
        };
        halo_store::sessions::finish(
            conn,
            &id,
            start + i64::from(mins) * 60_000,
            &progress,
            start,
        )
        .unwrap();
    }

    const GOAL: u64 = 4 * 3600;

    #[test]
    fn today_has_the_goal_ring_tree_and_streak() {
        let conn = db();
        session(&conn, d(2026, 11, 21), 9, 50, None);
        session(&conn, d(2026, 11, 22), 9, 50, None);
        session(&conn, d(2026, 11, 23), 9, 100, None);
        session(&conn, d(2026, 11, 23), 14, 50, None);
        let t = today_in_tz(&conn, &Lagos, now(), GOAL).unwrap();
        assert_eq!((t.focused_label.as_str(), t.sessions), ("2h 30m", 2));
        assert_eq!(t.goal_label, "of your 4h goal");
        assert_eq!(t.goal_permille, 625);
        assert_eq!(t.tree, dto::TreeStageView::YoungTree);
        assert_eq!((t.streak, t.streak_today), (3, true));
    }

    #[test]
    fn the_week_view_compares_with_last_week() {
        let conn = db();
        session(&conn, d(2026, 11, 17), 9, 60, None); // last week
        session(&conn, d(2026, 11, 23), 9, 120, None); // this week, Monday
        let v = view_in(&conn, &Lagos, now(), GOAL, StatsRange::Week).unwrap();
        assert_eq!(v.range_label, "23–29 Nov");
        let focused = &v.kpis[1];
        assert_eq!(
            (focused.value.as_str(), focused.detail.as_str()),
            ("2h 00m", "+1h 00m vs last week")
        );
        assert_eq!(v.kpis[2].value, "1");
        assert_eq!(v.best_hours.as_deref(), Some("09:00–11:00"));
        assert_eq!(v.heatmap.len(), 24);
        assert!(v.heatmap.iter().all(|w| w.len() == 7));
        // Sunday 29 Nov hasn't happened yet.
        assert_eq!(v.heatmap[23][6].level, None);
        assert_eq!(v.heatmap[23][0].level, Some(3));
        assert_eq!(v.grove.len(), 14);
        assert!(v.grove.last().unwrap().today);
    }

    #[test]
    fn time_by_project_comes_through_tasks() {
        let conn = db();
        let p = halo_store::projects::create(
            &conn,
            &halo_store::projects::ProjectInput {
                name: "meridian-rs",
                color: "#F2B544",
                goal_id: None,
            },
            1,
        )
        .unwrap();
        let t = halo_store::tasks::create(
            &conn,
            &halo_store::tasks::TaskInput::titled("t").in_project(&p),
            1,
        )
        .unwrap();
        session(&conn, d(2026, 11, 23), 9, 90, Some(&t));
        session(&conn, d(2026, 11, 23), 13, 30, None);
        let v = view_in(&conn, &Lagos, now(), GOAL, StatsRange::Day).unwrap();
        let names: Vec<(&str, u32)> = v
            .projects
            .iter()
            .map(|p| (p.name.as_str(), p.permille))
            .collect();
        assert_eq!(names, [("meridian-rs", 750), ("No project", 250)]);
    }

    #[test]
    fn ranges_and_labels() {
        let mon = d(2026, 11, 23);
        assert_eq!(
            bounds(StatsRange::Week, mon).0,
            (d(2026, 11, 23), d(2026, 11, 29))
        );
        assert_eq!(
            bounds(StatsRange::Week, d(2026, 11, 29)).1,
            (d(2026, 11, 16), d(2026, 11, 22))
        );
        assert_eq!(
            bounds(StatsRange::Month, mon),
            (
                (d(2026, 11, 1), d(2026, 11, 30)),
                (d(2026, 10, 1), d(2026, 10, 31))
            )
        );
        assert_eq!(
            bounds(StatsRange::Year, mon).1,
            (d(2025, 1, 1), d(2025, 12, 31))
        );
        assert_eq!(
            range_label(StatsRange::Week, (d(2026, 11, 30), d(2026, 12, 6)), mon),
            "30 Nov – 6 Dec"
        );
        assert_eq!(duration(78_000), "21h 40m");
        assert_eq!(
            duration_delta(3600, 4800, "vs yesterday"),
            "−20m vs yesterday"
        );
        assert_eq!(count_delta(0, "vs last week"), "Same as last week");
        assert_eq!(clock(3000), "50:00");
        assert_eq!(goal_duration(270 * 60), "4h 30m");
    }

    #[test]
    fn export_renders_csv_and_json() {
        let conn = db();
        session(&conn, d(2026, 11, 23), 9, 50, None);
        let history = halo_store::sessions::history(&conn).unwrap();
        let csv = render_export(&Lagos, &history, &[], false).unwrap();
        let mut lines = csv.lines();
        assert_eq!(lines.next(), Some(EXPORT_COLUMNS.join(",").as_str()));
        assert!(
            lines
                .next()
                .unwrap()
                .starts_with("2026-11-23,09:00,09:50,deep,timer,Write tests,,50,50,0,0,,0,,")
        );
        let json: serde_json::Value =
            serde_json::from_str(&render_export(&Lagos, &history, &[], true).unwrap()).unwrap();
        assert_eq!(json[0]["focused_minutes"], 50);
        assert_eq!(json[0]["rating"], serde_json::Value::Null);
    }
}
