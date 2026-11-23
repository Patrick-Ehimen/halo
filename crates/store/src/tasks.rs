//! Tasks: session estimates, subtasks, repeats, and the day they're planned
//! for. Sessions done is counted from saved sessions.

use rusqlite::{Connection, OptionalExtension, params};

use crate::Result;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaskInput<'a> {
    pub title: &'a str,
    pub project_id: Option<&'a str>,
    pub milestone_id: Option<&'a str>,
    pub parent_id: Option<&'a str>,
    pub estimate_sessions: u32,
    /// Local date, YYYY-MM-DD.
    pub planned_on: Option<&'a str>,
    /// Unix ms.
    pub due_at: Option<i64>,
    /// `daily`, `weekdays` or `weekly:MO,WE`.
    pub recurrence: Option<&'a str>,
}

impl<'a> TaskInput<'a> {
    /// A one-session task with nothing else set.
    pub fn titled(title: &'a str) -> Self {
        TaskInput {
            title,
            estimate_sessions: 1,
            ..TaskInput::default()
        }
    }

    pub fn in_project(mut self, project_id: &'a str) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub fn on(mut self, planned_on: &'a str) -> Self {
        self.planned_on = Some(planned_on);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub project_id: Option<String>,
    pub milestone_id: Option<String>,
    pub parent_id: Option<String>,
    pub title: String,
    pub estimate_sessions: u32,
    pub planned_on: Option<String>,
    pub due_at: Option<i64>,
    pub recurrence: Option<String>,
    pub completed_at: Option<i64>,
    /// Saved sessions started from this task.
    pub done_sessions: u32,
}

pub fn create(conn: &Connection, t: &TaskInput<'_>, now: i64) -> Result<String> {
    Ok(conn.query_row(
        "INSERT INTO tasks (project_id, milestone_id, parent_id, title, estimate_sessions,
                            planned_on, due_at, recurrence, position, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                 (SELECT COALESCE(MAX(position), 0) + 1 FROM tasks), ?9, ?9)
         RETURNING id",
        params![
            t.project_id,
            t.milestone_id,
            t.parent_id,
            t.title,
            t.estimate_sessions,
            t.planned_on,
            t.due_at,
            t.recurrence,
            now
        ],
        |r| r.get(0),
    )?)
}

/// Replaces a task's fields (not its parent).
pub fn update(conn: &Connection, id: &str, t: &TaskInput<'_>, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE tasks SET project_id = ?2, milestone_id = ?3, title = ?4, estimate_sessions = ?5,
                          planned_on = ?6, due_at = ?7, recurrence = ?8,
                          updated_at = ?9, synced_at = NULL
         WHERE id = ?1",
        params![
            id,
            t.project_id,
            t.milestone_id,
            t.title,
            t.estimate_sessions,
            t.planned_on,
            t.due_at,
            t.recurrence,
            now
        ],
    )?;
    Ok(())
}

pub fn complete(conn: &Connection, id: &str, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE tasks SET completed_at = ?2, updated_at = ?2, synced_at = NULL WHERE id = ?1",
        params![id, now],
    )?;
    Ok(())
}

pub fn reopen(conn: &Connection, id: &str, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE tasks SET completed_at = NULL, updated_at = ?2, synced_at = NULL WHERE id = ?1",
        params![id, now],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str, now: i64) -> Result<()> {
    crate::soft_or_hard_delete(conn, "tasks", id, now)
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<Task>> {
    Ok(conn
        .query_row(
            &format!("{SELECT} WHERE t.id = ?1 AND t.deleted_at IS NULL"),
            [id],
            row,
        )
        .optional()?)
}

/// The day's plan: open tasks planned for `today` or earlier (overdue ones
/// roll forward), tasks finished during the day, and their subtasks.
pub fn list_day(conn: &Connection, today: &str, day_start: i64, day_end: i64) -> Result<Vec<Task>> {
    let on_the_day = "(t.completed_at IS NULL AND t.planned_on IS NOT NULL AND t.planned_on <= ?1)
                      OR (t.completed_at >= ?2 AND t.completed_at < ?3)";
    let mut stmt = conn.prepare(&format!(
        "{SELECT} WHERE t.deleted_at IS NULL AND (
            ({on_the_day})
            OR t.parent_id IN (SELECT t.id FROM tasks t WHERE t.deleted_at IS NULL AND ({on_the_day}))
         )
         ORDER BY t.completed_at IS NOT NULL, t.due_at IS NULL, t.due_at, t.position"
    ))?;
    let rows = stmt.query_map(params![today, day_start, day_end], row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Every open task, for search and the palette (subtasks included).
pub fn list_open(conn: &Connection) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(&format!(
        "{SELECT} WHERE t.deleted_at IS NULL AND t.completed_at IS NULL ORDER BY t.position"
    ))?;
    let rows = stmt.query_map([], row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// What a goal's open tasks add up to: (estimated sessions, done sessions).
pub fn goal_estimates(conn: &Connection, goal_id: &str) -> Result<(u32, u32)> {
    Ok(conn.query_row(
        "SELECT COALESCE(SUM(t.estimate_sessions), 0),
                COALESCE(SUM((SELECT COUNT(*) FROM sessions s
                              WHERE s.task_id = t.id AND s.ended_at IS NOT NULL
                                AND s.deleted_at IS NULL)), 0)
         FROM tasks t JOIN projects p ON p.id = t.project_id
         WHERE p.goal_id = ?1 AND t.deleted_at IS NULL AND t.parent_id IS NULL",
        [goal_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?)
}

const SELECT: &str = "SELECT t.id, t.project_id, t.milestone_id, t.parent_id, t.title,
        t.estimate_sessions, t.planned_on, t.due_at, t.recurrence, t.completed_at,
        (SELECT COUNT(*) FROM sessions s
         WHERE s.task_id = t.id AND s.ended_at IS NOT NULL AND s.deleted_at IS NULL)
    FROM tasks t";

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: r.get(0)?,
        project_id: r.get(1)?,
        milestone_id: r.get(2)?,
        parent_id: r.get(3)?,
        title: r.get(4)?,
        estimate_sessions: r.get(5)?,
        planned_on: r.get(6)?,
        due_at: r.get(7)?,
        recurrence: r.get(8)?,
        completed_at: r.get(9)?,
        done_sessions: r.get(10)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_db;

    // The day of 2026-11-21 in ms, for completion times.
    const DAY_START: i64 = 1_000_000;
    const DAY_END: i64 = 2_000_000;

    fn ids(tasks: &[Task]) -> Vec<&str> {
        tasks.iter().map(|t| t.title.as_str()).collect()
    }

    #[test]
    fn the_day_has_today_overdue_and_finished_today() {
        let conn = test_db();
        create(&conn, &TaskInput::titled("today").on("2026-11-21"), 1).unwrap();
        create(&conn, &TaskInput::titled("overdue").on("2026-11-19"), 1).unwrap();
        create(&conn, &TaskInput::titled("tomorrow").on("2026-11-22"), 1).unwrap();
        create(&conn, &TaskInput::titled("someday"), 1).unwrap();
        let done = create(&conn, &TaskInput::titled("done today").on("2026-11-21"), 1).unwrap();
        complete(&conn, &done, DAY_START + 5).unwrap();
        let old = create(&conn, &TaskInput::titled("done before").on("2026-11-18"), 1).unwrap();
        complete(&conn, &old, DAY_START - 5).unwrap();

        let day = list_day(&conn, "2026-11-21", DAY_START, DAY_END).unwrap();
        assert_eq!(ids(&day), ["today", "overdue", "done today"]);
    }

    #[test]
    fn timed_tasks_come_first_in_time_order() {
        let conn = test_db();
        create(&conn, &TaskInput::titled("untimed").on("2026-11-21"), 1).unwrap();
        create(
            &conn,
            &TaskInput {
                due_at: Some(DAY_START + 900),
                ..TaskInput::titled("later").on("2026-11-21")
            },
            1,
        )
        .unwrap();
        create(
            &conn,
            &TaskInput {
                due_at: Some(DAY_START + 100),
                ..TaskInput::titled("sooner").on("2026-11-21")
            },
            1,
        )
        .unwrap();
        let day = list_day(&conn, "2026-11-21", DAY_START, DAY_END).unwrap();
        assert_eq!(ids(&day), ["sooner", "later", "untimed"]);
    }

    #[test]
    fn subtasks_come_with_their_parent() {
        let conn = test_db();
        let parent = create(&conn, &TaskInput::titled("parent").on("2026-11-21"), 1).unwrap();
        create(
            &conn,
            &TaskInput {
                parent_id: Some(&parent),
                ..TaskInput::titled("child")
            },
            1,
        )
        .unwrap();
        let day = list_day(&conn, "2026-11-21", DAY_START, DAY_END).unwrap();
        assert_eq!(ids(&day), ["parent", "child"]);
        assert_eq!(day[1].parent_id.as_deref(), Some(parent.as_str()));
    }

    #[test]
    fn done_sessions_are_counted_from_saved_sessions() {
        let conn = test_db();
        let t = create(
            &conn,
            &TaskInput {
                estimate_sessions: 4,
                ..TaskInput::titled("t").on("2026-11-21")
            },
            1,
        )
        .unwrap();
        crate::test_session(&conn, &t, 10);
        crate::test_session(&conn, &t, 20);
        let task = get(&conn, &t).unwrap().unwrap();
        assert_eq!((task.done_sessions, task.estimate_sessions), (2, 4));
    }

    #[test]
    fn update_reopen_and_delete() {
        let conn = test_db();
        let t = create(&conn, &TaskInput::titled("t"), 1).unwrap();
        update(
            &conn,
            &t,
            &TaskInput {
                recurrence: Some("daily"),
                ..TaskInput::titled("renamed").on("2026-11-21")
            },
            2,
        )
        .unwrap();
        complete(&conn, &t, 3).unwrap();
        reopen(&conn, &t, 4).unwrap();
        let task = get(&conn, &t).unwrap().unwrap();
        assert_eq!(
            (
                task.title.as_str(),
                task.completed_at,
                task.recurrence.as_deref()
            ),
            ("renamed", None, Some("daily"))
        );
        assert_eq!(list_open(&conn).unwrap().len(), 1);
        delete(&conn, &t, 5).unwrap();
        assert!(get(&conn, &t).unwrap().is_none());
    }

    #[test]
    fn goal_estimates_add_up_open_and_done() {
        let conn = test_db();
        let g = crate::goals::create(
            &conn,
            &crate::goals::GoalInput {
                title: "g",
                why: "",
                target_sessions: 100,
                due_on: None,
                color: "#fff",
            },
            1,
        )
        .unwrap();
        let p = crate::projects::create(
            &conn,
            &crate::projects::ProjectInput {
                name: "p",
                color: "#fff",
                goal_id: Some(&g),
            },
            1,
        )
        .unwrap();
        let a = create(
            &conn,
            &TaskInput {
                estimate_sessions: 4,
                ..TaskInput::titled("a").in_project(&p)
            },
            1,
        )
        .unwrap();
        create(
            &conn,
            &TaskInput {
                estimate_sessions: 6,
                ..TaskInput::titled("b").in_project(&p)
            },
            1,
        )
        .unwrap();
        crate::test_session(&conn, &a, 1);
        assert_eq!(goal_estimates(&conn, &g).unwrap(), (10, 1));
    }
}
