//! Planning for the app: builds the Today plan and Goals views, and applies
//! changes. Rules live in `halo-plan` and storage in `halo-store`; this module
//! joins them, in the user's local time zone.

mod dto;

use std::collections::HashMap;

use chrono::{Datelike, Days, Local, NaiveDate, NaiveTime, TimeZone};
use halo_plan::{Recurrence, at_local, day_bounds, goal_stats, local_minute};
use halo_store::Connection;
use halo_store::{goals, milestones, projects, tasks, time_blocks};
use tauri::AppHandle;
use tauri_specta::Event;

use dto::{BlockView, GoalView, MilestoneView, ProjectView, TaskView};
pub use dto::{GoalForm, PlanChanged, PlanView, ProjectForm, QuickAddPreview, TaskForm};

use crate::error::{Error, Result};
use crate::state::AppState;
use crate::timer::now_ms;

/// Colours offered for goals and projects, in the order new ones take them.
pub const COLORS: [&str; 9] = [
    "#F2B544", "#6CB6FF", "#C59CFF", "#5BD69A", "#FF9F7A", "#7FD1C9", "#FF8AC8", "#B5E06B",
    "#8E9BFF",
];

const DAY_MS: i64 = 86_400_000;
/// How long one estimated session takes on the timeline.
const TIMELINE_SESSION_MINUTES: u32 = 25;
const MAX_TARGET: u32 = 10_000;
const MAX_ESTIMATE: u32 = 50;

/// Tells every window the plan changed.
pub fn changed(app: &AppHandle) {
    if let Err(e) = PlanChanged.emit(app) {
        eprintln!("plan event: {e}");
    }
}

fn now_i64() -> i64 {
    i64::try_from(now_ms()).unwrap_or(i64::MAX)
}

pub fn view(state: &AppState) -> Result<PlanView> {
    let conn = state.db()?;
    view_in(&conn, &Local, now_i64())
}

/// The plan as seen at `now` in `tz`.
fn view_in<Tz: TimeZone>(conn: &Connection, tz: &Tz, now: i64) -> Result<PlanView> {
    let (today, now_minute) = local_minute(tz, now).ok_or(Error::Invalid("Bad clock time"))?;
    let today_s = today.format("%Y-%m-%d").to_string();
    let (day_start, day_end) = day_bounds(tz, today);
    let week_start = today
        .checked_sub_days(Days::new(u64::from(today.weekday().num_days_from_monday())))
        .map_or(day_start, |monday| day_bounds(tz, monday).0);

    let goal_rows = goals::list(conn)?;
    let project_rows = projects::list(conn)?;
    let milestone_rows = milestones::list(conn)?;
    let recent = goals::counts(conn, now - i64::from(halo_plan::PACE_WEEKS) * 7 * DAY_MS)?;
    let this_week = goals::counts(conn, week_start)?;
    let day_tasks = tasks::list_day(conn, &today_s, day_start, day_end)?;
    let open_tasks: Vec<tasks::Task> = tasks::list_open(conn)?
        .into_iter()
        .filter(|t| t.parent_id.is_none())
        .collect();

    let goal_of_project: HashMap<&str, Option<&str>> = project_rows
        .iter()
        .map(|p| (p.id.as_str(), p.goal_id.as_deref()))
        .collect();
    let goal_of_task = |t: &tasks::Task| -> Option<String> {
        t.project_id
            .as_deref()
            .and_then(|p| goal_of_project.get(p).copied().flatten())
            .map(str::to_string)
    };

    // Sessions still to do today, per goal.
    let mut today_left: HashMap<String, u32> = HashMap::new();
    for t in day_tasks
        .iter()
        .filter(|t| t.parent_id.is_none() && t.completed_at.is_none())
    {
        if let Some(g) = goal_of_task(t) {
            *today_left.entry(g).or_default() +=
                t.estimate_sessions.saturating_sub(t.done_sessions);
        }
    }

    let mut goals_out = Vec::with_capacity(goal_rows.len());
    for g in goal_rows {
        let due = g.due_on.as_deref().and_then(parse_date);
        let counts = recent.get(&g.id).copied().unwrap_or_default();
        let started = local_minute(tz, g.created_at).map_or(today, |(d, _)| d);
        let stats = goal_stats(
            counts.done,
            g.target_sessions,
            due,
            started,
            today,
            counts.recent,
        );
        let (estimate_total, _) = tasks::goal_estimates(conn, &g.id)?;
        goals_out.push(GoalView {
            milestones: milestone_rows
                .iter()
                .filter(|m| m.goal_id == g.id)
                .map(|m| milestone_view(tz, m))
                .collect(),
            due_label: due.map(|d| d.format("%-d %b %Y").to_string()),
            time_left: stats.days_left.map(time_left),
            done_sessions: stats.done,
            percent: stats.percent,
            pace: stats.pace.into(),
            needed_per_week_tenths: stats.needed_per_week.map(tenths),
            averaged_per_week_tenths: tenths(stats.averaged_per_week),
            this_week: this_week.get(&g.id).map_or(0, |c| c.recent),
            today_sessions: today_left.get(&g.id).copied().unwrap_or(0),
            estimate_total,
            id: g.id,
            title: g.title,
            why: g.why,
            color: g.color,
            status: g.status,
            target_sessions: g.target_sessions,
            due_on: g.due_on,
        });
    }

    let projects_out = project_rows
        .iter()
        .map(|p| {
            let open: Vec<&tasks::Task> = open_tasks
                .iter()
                .filter(|t| t.project_id.as_deref() == Some(p.id.as_str()))
                .collect();
            ProjectView {
                id: p.id.clone(),
                goal_id: p.goal_id.clone(),
                name: p.name.clone(),
                color: p.color.clone(),
                description: p.description.clone(),
                open_tasks: u32::try_from(open.len()).unwrap_or(u32::MAX),
                sessions_left: open
                    .iter()
                    .map(|t| t.estimate_sessions.saturating_sub(t.done_sessions))
                    .sum(),
            }
        })
        .collect();

    let blocks = time_blocks::list_between(conn, day_start, day_end)?
        .into_iter()
        .map(|b| {
            let minute = |ms: i64| -> u32 {
                if ms <= day_start {
                    0
                } else if ms >= day_end {
                    1440
                } else {
                    local_minute(tz, ms).map_or(0, |(_, m)| m)
                }
            };
            let (start, end) = (minute(b.starts_at), minute(b.ends_at));
            BlockView {
                label: format!("{}–{}", hhmm(start), hhmm(end)),
                start_time: hhmm(start),
                minutes: u32::try_from((b.ends_at - b.starts_at) / 60_000).unwrap_or(0),
                id: b.id,
                task_id: b.task_id,
                title: b.title,
                kind: b.kind,
                start_minute: start,
                end_minute: end,
            }
        })
        .collect();

    Ok(PlanView {
        date_label: today.format("%A, %-d %B").to_string(),
        date: today_s,
        now_minute,
        goals: goals_out,
        projects: projects_out,
        today: day_tasks.iter().map(|t| task_view(tz, t, today)).collect(),
        open: open_tasks.iter().map(|t| task_view(tz, t, today)).collect(),
        blocks,
    })
}

fn task_view<Tz: TimeZone>(tz: &Tz, t: &tasks::Task, today: NaiveDate) -> TaskView {
    let planned = t.planned_on.as_deref().and_then(parse_date);
    let due = t.due_at.and_then(|ms| local_minute(tz, ms));
    let end = t.ends_at.and_then(|ms| local_minute(tz, ms));
    let completed = t.completed_at.is_some();
    let overdue = !completed && planned.is_some_and(|d| d < today);
    let when_label = match (due, planned) {
        (Some((date, minute)), _) => Some(match day_word(date, today) {
            Some(word) if word == "Today" => hhmm(minute),
            Some(word) => format!("{word} {}", hhmm(minute)),
            None => format!("{} {}", date.format("%a %-d %b"), hhmm(minute)),
        }),
        (None, Some(date)) if overdue => Some(format!("From {}", date.format("%a %-d %b"))),
        (None, Some(date)) if date != today => {
            day_word(date, today).or_else(|| Some(date.format("%a %-d %b").to_string()))
        }
        _ => None,
    };
    TaskView {
        id: t.id.clone(),
        title: t.title.clone(),
        project_id: t.project_id.clone(),
        milestone_id: t.milestone_id.clone(),
        parent_id: t.parent_id.clone(),
        estimate_sessions: t.estimate_sessions,
        done_sessions: t.done_sessions,
        planned_on: t.planned_on.clone(),
        due_minute: due.filter(|(d, _)| *d == today).map(|(_, m)| m),
        due_time: due.map(|(_, m)| hhmm(m)),
        due_end_minute: due.filter(|(d, _)| *d == today).map(|(d, m)| match end {
            Some((end_day, end_m)) if end_day == d => end_m.max(m),
            Some(_) => 1440,
            None => (m + t.estimate_sessions.max(1) * TIMELINE_SESSION_MINUTES).min(1440),
        }),
        end_time: end.map(|(_, m)| hhmm(m)),
        description: t.description.clone(),
        when_label,
        recurrence_label: t
            .recurrence
            .as_deref()
            .and_then(Recurrence::parse)
            .map(|r| r.describe()),
        recurrence: t.recurrence.clone(),
        completed,
        overdue,
    }
}

fn milestone_view<Tz: TimeZone>(tz: &Tz, m: &milestones::Milestone) -> MilestoneView {
    let label = match (m.done_at, m.due_on.as_deref().and_then(parse_date)) {
        (Some(ms), _) => local_minute(tz, ms).map_or_else(
            || "Done".into(),
            |(d, _)| format!("Done {}", d.format("%-d %b")),
        ),
        (None, Some(d)) => format!("Due {}", d.format("%-d %b %Y")),
        (None, None) => "No date".into(),
    };
    MilestoneView {
        id: m.id.clone(),
        goal_id: m.goal_id.clone(),
        title: m.title.clone(),
        due_on: m.due_on.clone(),
        label,
        done: m.done_at.is_some(),
    }
}

fn day_word(date: NaiveDate, today: NaiveDate) -> Option<String> {
    match (date - today).num_days() {
        0 => Some("Today".into()),
        1 => Some("Tomorrow".into()),
        -1 => Some("Yesterday".into()),
        2..=6 => Some(date.format("%a").to_string()),
        _ => None,
    }
}

fn time_left(days: i64) -> String {
    match days {
        d if d < -13 => format!("{} weeks overdue", -d / 7),
        d if d < -1 => format!("{} days overdue", -d),
        -1 => "1 day overdue".into(),
        0 => "Due today".into(),
        1 => "1 day left".into(),
        d if d < 14 => format!("{d} days left"),
        d => format!("{} weeks left", d / 7),
    }
}

fn hhmm(minute: u32) -> String {
    format!("{:02}:{:02}", (minute / 60) % 24, minute % 60)
}

fn tenths(x: f64) -> u32 {
    let t = (x * 10.0).round();
    if t.is_finite() && t >= 0.0 {
        t.min(f64::from(u32::MAX)) as u32
    } else {
        0
    }
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

fn today_in<Tz: TimeZone>(tz: &Tz, now: i64) -> Result<NaiveDate> {
    local_minute(tz, now)
        .map(|(d, _)| d)
        .ok_or(Error::Invalid("Bad clock time"))
}

// ---------------------------------------------------------------- changes

fn valid_title(s: &str) -> Result<&str> {
    let t = s.trim();
    if t.is_empty() {
        return Err(Error::Invalid("Give it a name"));
    }
    if t.chars().count() > 200 {
        return Err(Error::Invalid("Keep the name under 200 characters"));
    }
    Ok(t)
}

/// The most a description holds, in characters.
const MAX_DESCRIPTION: usize = 2000;

fn valid_description(s: &str) -> Result<&str> {
    let t = s.trim();
    if t.chars().count() > MAX_DESCRIPTION {
        Err(Error::Invalid(
            "Keep the description under 2,000 characters",
        ))
    } else {
        Ok(t)
    }
}

fn valid_color(c: &str) -> Result<&str> {
    let hex = c.strip_prefix('#').unwrap_or("");
    if hex.len() == 6 && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Ok(c)
    } else {
        Err(Error::Invalid("Pick a colour"))
    }
}

fn valid_date(d: Option<&str>) -> Result<Option<&str>> {
    match d.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(s) if parse_date(s).is_some() => Ok(Some(s)),
        Some(_) => Err(Error::Invalid("Use a date like 2027-03-15")),
    }
}

fn valid_recurrence(r: Option<&str>) -> Result<Option<Recurrence>> {
    match r.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(s) => Recurrence::parse(s)
            .map(Some)
            .ok_or(Error::Invalid("That repeat isn't supported")),
    }
}

fn goal_input(f: &GoalForm) -> Result<goals::GoalInput<'_>> {
    if f.target_sessions == 0 || f.target_sessions > MAX_TARGET {
        return Err(Error::Invalid("Set a target between 1 and 10,000 sessions"));
    }
    Ok(goals::GoalInput {
        title: valid_title(&f.title)?,
        why: f.why.trim(),
        target_sessions: f.target_sessions,
        due_on: valid_date(f.due_on.as_deref())?,
        color: valid_color(&f.color)?,
    })
}

pub fn create_goal(state: &AppState, f: &GoalForm) -> Result<String> {
    let input = goal_input(f)?;
    Ok(goals::create(&*state.db()?, &input, now_i64())?)
}

pub fn update_goal(state: &AppState, id: &str, f: &GoalForm) -> Result<()> {
    let input = goal_input(f)?;
    Ok(goals::update(&*state.db()?, id, &input, now_i64())?)
}

pub fn set_goal_done(state: &AppState, id: &str, done: bool) -> Result<()> {
    let status = if done { "done" } else { "active" };
    Ok(goals::set_status(&*state.db()?, id, status, now_i64())?)
}

pub fn delete_goal(state: &AppState, id: &str) -> Result<()> {
    Ok(goals::delete(&*state.db()?, id, now_i64())?)
}

pub fn create_milestone(
    state: &AppState,
    goal_id: &str,
    title: &str,
    due_on: Option<&str>,
) -> Result<String> {
    let (title, due_on) = (valid_title(title)?, valid_date(due_on)?);
    Ok(milestones::create(
        &*state.db()?,
        goal_id,
        title,
        due_on,
        now_i64(),
    )?)
}

pub fn set_milestone_done(state: &AppState, id: &str, done: bool) -> Result<()> {
    let now = now_i64();
    Ok(milestones::set_done(
        &*state.db()?,
        id,
        done.then_some(now),
        now,
    )?)
}

pub fn delete_milestone(state: &AppState, id: &str) -> Result<()> {
    Ok(milestones::delete(&*state.db()?, id, now_i64())?)
}

pub fn create_project(state: &AppState, f: &ProjectForm) -> Result<String> {
    let input = projects::ProjectInput {
        name: valid_title(&f.name)?,
        color: valid_color(&f.color)?,
        goal_id: f.goal_id.as_deref().filter(|g| !g.is_empty()),
        description: valid_description(&f.description)?,
    };
    Ok(projects::create(&*state.db()?, &input, now_i64())?)
}

pub fn update_project(state: &AppState, id: &str, f: &ProjectForm) -> Result<()> {
    let input = projects::ProjectInput {
        name: valid_title(&f.name)?,
        color: valid_color(&f.color)?,
        goal_id: f.goal_id.as_deref().filter(|g| !g.is_empty()),
        description: valid_description(&f.description)?,
    };
    Ok(projects::update(&*state.db()?, id, &input, now_i64())?)
}

pub fn delete_project(state: &AppState, id: &str) -> Result<()> {
    Ok(projects::delete(&*state.db()?, id, now_i64())?)
}

/// A task form as stored values: the due time becomes a timestamp on the
/// planned day, and a repeat with no day starts on its first matching day.
struct Resolved {
    planned_on: Option<String>,
    due_at: Option<i64>,
    ends_at: Option<i64>,
    recurrence: Option<String>,
}

fn resolve<Tz: TimeZone>(tz: &Tz, f: &TaskForm, now: i64) -> Result<Resolved> {
    if f.estimate_sessions > MAX_ESTIMATE {
        return Err(Error::Invalid("Estimate 50 sessions or fewer"));
    }
    let rule = valid_recurrence(f.recurrence.as_deref())?;
    let mut planned = valid_date(f.planned_on.as_deref())?.and_then(parse_date);
    let time = match f
        .due_time
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        None => None,
        Some(s) => Some(
            NaiveTime::parse_from_str(s, "%H:%M")
                .map_err(|_| Error::Invalid("Use a time like 09:30"))?,
        ),
    };
    let end = match f
        .end_time
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        None => None,
        Some(s) => Some(
            NaiveTime::parse_from_str(s, "%H:%M")
                .map_err(|_| Error::Invalid("Use an end time like 10:30"))?,
        ),
    };
    match (time, end) {
        (None, Some(_)) => return Err(Error::Invalid("Set a start time before an end time")),
        (Some(t), Some(e)) if e <= t => return Err(Error::Invalid("End it after it starts")),
        _ => {}
    }
    let today = today_in(tz, now)?;
    if planned.is_none() {
        planned = match (&rule, time) {
            (Some(r), _) => Some(r.on_or_after(today)),
            (None, Some(_)) => Some(today),
            _ => None,
        };
    }
    Ok(Resolved {
        due_at: match (planned, time) {
            (Some(d), Some(t)) => Some(at_local(tz, d, t)),
            _ => None,
        },
        ends_at: match (planned, end) {
            (Some(d), Some(e)) => Some(at_local(tz, d, e)),
            _ => None,
        },
        planned_on: planned.map(|d| d.format("%Y-%m-%d").to_string()),
        recurrence: rule.map(|r| r.to_string()),
    })
}

fn task_input<'a>(f: &'a TaskForm, r: &'a Resolved) -> Result<tasks::TaskInput<'a>> {
    Ok(tasks::TaskInput {
        title: valid_title(&f.title)?,
        project_id: f.project_id.as_deref().filter(|s| !s.is_empty()),
        milestone_id: f.milestone_id.as_deref().filter(|s| !s.is_empty()),
        parent_id: f.parent_id.as_deref().filter(|s| !s.is_empty()),
        estimate_sessions: f.estimate_sessions,
        planned_on: r.planned_on.as_deref(),
        due_at: r.due_at,
        recurrence: r.recurrence.as_deref(),
        description: valid_description(&f.description)?,
        ends_at: r.ends_at,
    })
}

pub fn create_task(state: &AppState, f: &TaskForm) -> Result<String> {
    create_task_in(&*state.db()?, &Local, f, now_i64())
}

fn create_task_in<Tz: TimeZone>(
    conn: &Connection,
    tz: &Tz,
    f: &TaskForm,
    now: i64,
) -> Result<String> {
    let r = resolve(tz, f, now)?;
    Ok(tasks::create(conn, &task_input(f, &r)?, now)?)
}

pub fn update_task(state: &AppState, id: &str, f: &TaskForm) -> Result<()> {
    update_task_in(&*state.db()?, &Local, id, f, now_i64())
}

/// Saves a task. When its time moves within a day it has a block on, the
/// block moves with it, keeping its length, so the timeline shows one item.
fn update_task_in<Tz: TimeZone>(
    conn: &Connection,
    tz: &Tz,
    id: &str,
    f: &TaskForm,
    now: i64,
) -> Result<()> {
    let r = resolve(tz, f, now)?;
    tasks::update(conn, id, &task_input(f, &r)?, now)?;
    let Some(due) = r.due_at else {
        return Ok(());
    };
    let Some((day, _)) = local_minute(tz, due) else {
        return Ok(());
    };
    let (from, to) = day_bounds(tz, day);
    if let Some(b) = time_blocks::list_for_task(conn, id, from, to)?.first()
        && b.starts_at != due
    {
        let ends = r.ends_at.unwrap_or(due + (b.ends_at - b.starts_at));
        time_blocks::reschedule(conn, &b.id, due, ends, now)?;
    }
    Ok(())
}

pub fn complete_task(state: &AppState, id: &str) -> Result<()> {
    complete_task_in(&*state.db()?, &Local, id, now_i64())
}

/// Finishes a task. A repeating task comes back on its next day, with its
/// time of day kept.
fn complete_task_in<Tz: TimeZone>(conn: &Connection, tz: &Tz, id: &str, now: i64) -> Result<()> {
    let task = tasks::get(conn, id)?.ok_or(Error::Invalid("That task no longer exists"))?;
    tasks::complete(conn, id, now)?;
    let Some(rule) = task.recurrence.as_deref().and_then(Recurrence::parse) else {
        return Ok(());
    };
    let today = today_in(tz, now)?;
    let from = task
        .planned_on
        .as_deref()
        .and_then(parse_date)
        .map_or(today, |d| d.max(today));
    let next = rule.next_after(from);
    let next_s = next.format("%Y-%m-%d").to_string();
    let on_next = |ms: i64| {
        tz.timestamp_millis_opt(ms)
            .single()
            .map(|t| at_local(tz, next, t.naive_local().time()))
    };
    let due_at = task.due_at.and_then(on_next);
    let ends_at = task.ends_at.and_then(on_next);
    tasks::create(
        conn,
        &tasks::TaskInput {
            title: &task.title,
            project_id: task.project_id.as_deref(),
            milestone_id: task.milestone_id.as_deref(),
            parent_id: task.parent_id.as_deref(),
            estimate_sessions: task.estimate_sessions,
            planned_on: Some(&next_s),
            due_at,
            recurrence: task.recurrence.as_deref(),
            description: &task.description,
            ends_at,
        },
        now,
    )?;
    Ok(())
}

pub fn reopen_task(state: &AppState, id: &str) -> Result<()> {
    Ok(tasks::reopen(&*state.db()?, id, now_i64())?)
}

pub fn delete_task(state: &AppState, id: &str) -> Result<()> {
    Ok(tasks::delete(&*state.db()?, id, now_i64())?)
}

/// Puts a block on today's timeline, `minutes` long from `start` (local `HH:MM`).
pub fn create_block(
    state: &AppState,
    task_id: Option<&str>,
    title: &str,
    start: &str,
    minutes: u32,
) -> Result<String> {
    let (title, now) = (valid_title(title)?, now_i64());
    let (start, end) = block_span(&Local, now, start, minutes)?;
    Ok(time_blocks::create(
        &*state.db()?,
        &time_blocks::BlockInput {
            task_id,
            title,
            starts_at: start,
            ends_at: end,
            kind: "focus",
        },
        now,
    )?)
}

pub fn move_block(state: &AppState, id: &str, start: &str, minutes: u32) -> Result<()> {
    let now = now_i64();
    let (start, end) = block_span(&Local, now, start, minutes)?;
    Ok(time_blocks::reschedule(&*state.db()?, id, start, end, now)?)
}

pub fn delete_block(state: &AppState, id: &str) -> Result<()> {
    Ok(time_blocks::delete(&*state.db()?, id, now_i64())?)
}

fn block_span<Tz: TimeZone>(tz: &Tz, now: i64, start: &str, minutes: u32) -> Result<(i64, i64)> {
    if minutes == 0 || minutes > 12 * 60 {
        return Err(Error::Invalid("Pick a length up to 12 hours"));
    }
    let today = today_in(tz, now)?;
    let start_t = NaiveTime::parse_from_str(start.trim(), "%H:%M")
        .map_err(|_| Error::Invalid("Use a start time like 09:30"))?;
    let start = at_local(tz, today, start_t);
    Ok((start, start + i64::from(minutes) * 60_000))
}

// ---------------------------------------------------------------- quick add

pub fn preview_quick_add(state: &AppState, text: &str) -> Result<QuickAddPreview> {
    let conn = state.db()?;
    preview_in(&conn, &Local, text, now_i64())
}

fn preview_in<Tz: TimeZone>(
    conn: &Connection,
    tz: &Tz,
    text: &str,
    now: i64,
) -> Result<QuickAddPreview> {
    let today = today_in(tz, now)?;
    let q = halo_plan::parse_quick_add(text, today);
    let project = match q.project.as_deref() {
        Some(name) => projects::find_by_name(conn, name)?,
        None => None,
    };
    let goal_title = match project.as_ref().and_then(|p| p.goal_id.as_deref()) {
        Some(g) => goals::get(conn, g)?.map(|g| g.title),
        None => None,
    };
    Ok(QuickAddPreview {
        date_label: q
            .planned_on
            .map(|d| day_word(d, today).unwrap_or_else(|| d.format("%a %-d %b").to_string())),
        time_label: q.due_time.map(|t| t.format("%H:%M").to_string()),
        estimate: q.estimate,
        project_exists: project.is_some(),
        project_name: project.map(|p| p.name).or(q.project),
        goal_title,
        recurrence_label: q.recurrence.as_ref().map(Recurrence::describe),
        title: q.title,
    })
}

/// Adds a task from quick add. An unknown `#project` is created (with no goal).
pub fn quick_add(state: &AppState, text: &str) -> Result<String> {
    quick_add_in(&*state.db()?, &Local, text, now_i64())
}

fn quick_add_in<Tz: TimeZone>(conn: &Connection, tz: &Tz, text: &str, now: i64) -> Result<String> {
    let today = today_in(tz, now)?;
    let q = halo_plan::parse_quick_add(text, today);
    valid_title(&q.title)?;
    let project_id = match q.project.as_deref() {
        None => None,
        Some(name) => Some(match projects::find_by_name(conn, name)? {
            Some(p) => p.id,
            None => {
                let used = projects::list(conn)?.len();
                let color = COLORS[used % COLORS.len()];
                projects::create(
                    conn,
                    &projects::ProjectInput {
                        name,
                        color,
                        goal_id: None,
                        description: "",
                    },
                    now,
                )?
            }
        }),
    };
    let form = TaskForm {
        title: q.title,
        project_id,
        milestone_id: None,
        parent_id: None,
        estimate_sessions: q.estimate.unwrap_or(1),
        // Quick add without a date plans the task for today.
        planned_on: Some(q.planned_on.unwrap_or(today).format("%Y-%m-%d").to_string()),
        due_time: q.due_time.map(|t| t.format("%H:%M").to_string()),
        recurrence: q.recurrence.map(|r| r.to_string()),
        end_time: None,
        description: String::new(),
    };
    create_task_in(conn, tz, &form, now)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use chrono_tz::Africa::Lagos;

    use super::*;

    /// Saturday 21 Nov 2026, 10:28 in Lagos.
    fn now() -> i64 {
        at_local(
            &Lagos,
            NaiveDate::from_ymd_opt(2026, 11, 21).unwrap(),
            NaiveTime::from_hms_opt(10, 28, 0).unwrap(),
        )
    }

    fn db() -> Connection {
        halo_store::open(Path::new(":memory:")).unwrap()
    }

    fn form(title: &str) -> TaskForm {
        TaskForm {
            title: title.into(),
            project_id: None,
            milestone_id: None,
            parent_id: None,
            estimate_sessions: 1,
            planned_on: None,
            due_time: None,
            recurrence: None,
            end_time: None,
            description: String::new(),
        }
    }

    #[test]
    fn quick_add_creates_the_task_and_an_unknown_project() {
        let conn = db();
        quick_add_in(
            &conn,
            &Lagos,
            "review PR tomorrow 10am ~2 sessions #meridian-rs",
            now(),
        )
        .unwrap();
        let open = tasks::list_open(&conn).unwrap();
        assert_eq!(open.len(), 1);
        let t = &open[0];
        assert_eq!((t.title.as_str(), t.estimate_sessions), ("review PR", 2));
        assert_eq!(t.planned_on.as_deref(), Some("2026-11-22"));
        assert_eq!(local_minute(&Lagos, t.due_at.unwrap()).unwrap().1, 600);
        let project = projects::find_by_name(&conn, "meridian-rs")
            .unwrap()
            .unwrap();
        assert_eq!(t.project_id.as_deref(), Some(project.id.as_str()));
    }

    #[test]
    fn quick_add_preview_names_the_goal() {
        let conn = db();
        let g = goals::create(
            &conn,
            &goals::GoalInput {
                title: "Ship v1",
                why: "",
                target_sessions: 120,
                due_on: None,
                color: "#F2B544",
            },
            1,
        )
        .unwrap();
        projects::create(
            &conn,
            &projects::ProjectInput {
                name: "meridian-rs",
                color: "#F2B544",
                goal_id: Some(&g),
                description: "",
            },
            1,
        )
        .unwrap();
        let p = preview_in(
            &conn,
            &Lagos,
            "bench cancel path fri 3pm ~2 #meridian-rs",
            now(),
        )
        .unwrap();
        assert_eq!(p.title, "bench cancel path");
        assert_eq!(p.date_label.as_deref(), Some("Fri"));
        assert_eq!(p.time_label.as_deref(), Some("15:00"));
        assert_eq!(p.goal_title.as_deref(), Some("Ship v1"));
        assert!(p.project_exists);
    }

    #[test]
    fn a_repeating_task_comes_back_on_its_next_day_at_the_same_time() {
        let conn = db();
        let mut f = form("journal trades");
        f.recurrence = Some("weekdays".into());
        f.due_time = Some("08:30".into());
        let id = create_task_in(&conn, &Lagos, &f, now()).unwrap();
        // Saturday: weekdays start on Monday.
        assert_eq!(
            tasks::get(&conn, &id)
                .unwrap()
                .unwrap()
                .planned_on
                .as_deref(),
            Some("2026-11-23")
        );

        complete_task_in(&conn, &Lagos, &id, now()).unwrap();
        let open = tasks::list_open(&conn).unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].planned_on.as_deref(), Some("2026-11-24"));
        let (date, minute) = local_minute(&Lagos, open[0].due_at.unwrap()).unwrap();
        assert_eq!((date.to_string().as_str(), minute), ("2026-11-24", 510));
    }

    #[test]
    fn an_overdue_repeat_rolls_forward_from_today() {
        let conn = db();
        let mut f = form("standup");
        f.recurrence = Some("daily".into());
        f.planned_on = Some("2026-11-10".into());
        let id = create_task_in(&conn, &Lagos, &f, now()).unwrap();
        complete_task_in(&conn, &Lagos, &id, now()).unwrap();
        let open = tasks::list_open(&conn).unwrap();
        assert_eq!(open[0].planned_on.as_deref(), Some("2026-11-22"));
    }

    #[test]
    fn the_view_groups_today_and_counts_goal_progress() {
        let conn = db();
        let g = goals::create(
            &conn,
            &goals::GoalInput {
                title: "Ship v1",
                why: "",
                target_sessions: 10,
                due_on: Some("2026-12-19"),
                color: "#F2B544",
            },
            1,
        )
        .unwrap();
        let p = projects::create(
            &conn,
            &projects::ProjectInput {
                name: "meridian-rs",
                color: "#F2B544",
                goal_id: Some(&g),
                description: "",
            },
            1,
        )
        .unwrap();
        let mut f = form("write tests");
        f.project_id = Some(p.clone());
        f.estimate_sessions = 3;
        f.planned_on = Some("2026-11-21".into());
        f.due_time = Some("09:30".into());
        create_task_in(&conn, &Lagos, &f, now()).unwrap();
        let mut late = form("overdue thing");
        late.planned_on = Some("2026-11-19".into());
        create_task_in(&conn, &Lagos, &late, now()).unwrap();

        let v = view_in(&conn, &Lagos, now()).unwrap();
        assert_eq!(v.date_label, "Saturday, 21 November");
        assert_eq!(v.now_minute, 628);
        assert_eq!(v.today.len(), 2);
        let timed = v.today.iter().find(|t| t.title == "write tests").unwrap();
        assert_eq!(
            (timed.due_minute, timed.when_label.as_deref()),
            (Some(570), Some("09:30"))
        );
        assert_eq!(timed.due_end_minute, Some(570 + 3 * 25));
        let old = v.today.iter().find(|t| t.title == "overdue thing").unwrap();
        assert!(old.overdue);
        assert_eq!(old.when_label.as_deref(), Some("From Thu 19 Nov"));

        let goal = &v.goals[0];
        assert_eq!(
            (
                goal.today_sessions,
                goal.estimate_total,
                goal.target_sessions
            ),
            (3, 3, 10)
        );
        assert_eq!(goal.time_left.as_deref(), Some("4 weeks left"));
        assert_eq!(v.projects[0].sessions_left, 3);
    }

    #[test]
    fn validation_messages() {
        let conn = db();
        let err = |f: TaskForm| {
            create_task_in(&conn, &Lagos, &f, now())
                .unwrap_err()
                .to_string()
        };
        assert_eq!(err(form("  ")), "Give it a name");
        let mut f = form("x");
        f.due_time = Some("25:00".into());
        assert_eq!(err(f), "Use a time like 09:30");
        let mut f = form("x");
        f.recurrence = Some("hourly".into());
        assert_eq!(err(f), "That repeat isn't supported");
        assert_eq!(time_left(0), "Due today");
        assert_eq!(time_left(-20), "2 weeks overdue");
        assert_eq!(time_left(114), "16 weeks left");
    }

    #[test]
    fn blocks_land_on_the_local_day() {
        let (start, end) = block_span(&Lagos, now(), "09:30", 100).unwrap();
        assert_eq!(local_minute(&Lagos, start).unwrap().1, 570);
        assert_eq!(end - start, 100 * 60_000);
        assert!(block_span(&Lagos, now(), "9.30", 50).is_err());
        assert!(block_span(&Lagos, now(), "09:30", 0).is_err());
    }

    #[test]
    fn a_task_keeps_its_description_and_end_time() {
        let conn = db();
        let mut f = form("Review PR");
        f.description = "  Check the matching engine changes  ".into();
        f.due_time = Some("15:00".into());
        f.end_time = Some("15:50".into());
        create_task_in(&conn, &Lagos, &f, now()).unwrap();

        let view = view_in(&conn, &Lagos, now()).unwrap();
        let t = &view.today[0];
        assert_eq!(t.description, "Check the matching engine changes");
        assert_eq!(t.due_minute, Some(15 * 60));
        assert_eq!(t.end_time.as_deref(), Some("15:50"));
        // The end time sizes it on the timeline, not the session estimate.
        assert_eq!(t.due_end_minute, Some(15 * 60 + 50));
    }

    #[test]
    fn an_end_time_needs_a_start_before_it() {
        let conn = db();
        let mut f = form("Review PR");
        f.end_time = Some("15:50".into());
        assert!(create_task_in(&conn, &Lagos, &f, now()).is_err());
        f.due_time = Some("16:00".into());
        assert!(create_task_in(&conn, &Lagos, &f, now()).is_err());
        f.due_time = Some("15:50".into());
        assert!(create_task_in(&conn, &Lagos, &f, now()).is_err());
    }

    #[test]
    fn editing_a_tasks_time_moves_its_block() {
        let conn = db();
        let mut f = form("Write tests");
        f.due_time = Some("15:00".into());
        let id = create_task_in(&conn, &Lagos, &f, now()).unwrap();
        let (start, end) = block_span(&Lagos, now(), "15:00", 50).unwrap();
        let input = time_blocks::BlockInput {
            task_id: Some(&id),
            title: "Write tests",
            starts_at: start,
            ends_at: end,
            kind: "focus",
        };
        time_blocks::create(&conn, &input, now()).unwrap();

        f.due_time = Some("16:00".into());
        update_task_in(&conn, &Lagos, &id, &f, now()).unwrap();

        let (from, to) = day_bounds(&Lagos, NaiveDate::from_ymd_opt(2026, 11, 21).unwrap());
        let blocks = time_blocks::list_between(&conn, from, to).unwrap();
        assert_eq!(blocks.len(), 1, "moved, not copied");
        let (moved, _) = block_span(&Lagos, now(), "16:00", 50).unwrap();
        assert_eq!(blocks[0].starts_at, moved);
        assert_eq!(blocks[0].ends_at - blocks[0].starts_at, 50 * 60_000);
    }
}
