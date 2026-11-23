//! Long-term goals. Progress is counted from sessions on the goal's tasks.

use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension, params};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalInput<'a> {
    pub title: &'a str,
    pub why: &'a str,
    pub target_sessions: u32,
    /// Local date, YYYY-MM-DD.
    pub due_on: Option<&'a str>,
    pub color: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Goal {
    pub id: String,
    pub title: String,
    pub why: String,
    pub target_sessions: u32,
    pub due_on: Option<String>,
    pub color: String,
    /// `active` or `done`.
    pub status: String,
}

/// Sessions counted toward a goal.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GoalCounts {
    /// All saved sessions on the goal's tasks.
    pub done: u32,
    /// Those started since the `since` passed to [`counts`].
    pub recent: u32,
}

pub fn create(conn: &Connection, g: &GoalInput<'_>, now: i64) -> Result<String> {
    Ok(conn.query_row(
        "INSERT INTO goals (title, why, target_sessions, due_on, color, position, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, (SELECT COALESCE(MAX(position), 0) + 1 FROM goals), ?6, ?6)
         RETURNING id",
        params![g.title, g.why, g.target_sessions, g.due_on, g.color, now],
        |r| r.get(0),
    )?)
}

pub fn update(conn: &Connection, id: &str, g: &GoalInput<'_>, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE goals SET title = ?2, why = ?3, target_sessions = ?4, due_on = ?5, color = ?6,
                          updated_at = ?7, synced_at = NULL
         WHERE id = ?1",
        params![
            id,
            g.title,
            g.why,
            g.target_sessions,
            g.due_on,
            g.color,
            now
        ],
    )?;
    Ok(())
}

/// `active` or `done`.
pub fn set_status(conn: &Connection, id: &str, status: &str, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE goals SET status = ?2, updated_at = ?3, synced_at = NULL WHERE id = ?1",
        params![id, status, now],
    )?;
    Ok(())
}

/// Deletes a goal. Its projects stay, with no goal.
pub fn delete(conn: &Connection, id: &str, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE projects SET goal_id = NULL, updated_at = ?2, synced_at = NULL WHERE goal_id = ?1",
        params![id, now],
    )?;
    crate::soft_or_hard_delete(conn, "goals", id, now)
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<Goal>> {
    Ok(conn
        .query_row(
            &format!("{SELECT} WHERE id = ?1 AND deleted_at IS NULL"),
            [id],
            row,
        )
        .optional()?)
}

/// Active goals first, then done; each in the order they were made.
pub fn list(conn: &Connection) -> Result<Vec<Goal>> {
    let mut stmt = conn.prepare(&format!(
        "{SELECT} WHERE deleted_at IS NULL ORDER BY status = 'done', position"
    ))?;
    let rows = stmt
        .query_map([], row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Sessions per goal, through task → project → goal.
pub fn counts(conn: &Connection, since: i64) -> Result<HashMap<String, GoalCounts>> {
    let mut stmt = conn.prepare(
        "SELECT p.goal_id, COUNT(*), COALESCE(SUM(s.started_at >= ?1), 0)
         FROM sessions s
         JOIN tasks t ON t.id = s.task_id
         JOIN projects p ON p.id = t.project_id
         WHERE s.deleted_at IS NULL AND s.ended_at IS NOT NULL AND p.goal_id IS NOT NULL
         GROUP BY p.goal_id",
    )?;
    let rows = stmt.query_map([since], |r| {
        Ok((
            r.get::<_, String>(0)?,
            GoalCounts {
                done: r.get(1)?,
                recent: r.get(2)?,
            },
        ))
    })?;
    Ok(rows.collect::<rusqlite::Result<HashMap<_, _>>>()?)
}

/// When each finished goal was last updated (marked done), unix ms.
pub fn done_times(conn: &Connection) -> Result<Vec<i64>> {
    let mut stmt =
        conn.prepare("SELECT updated_at FROM goals WHERE status = 'done' AND deleted_at IS NULL")?;
    let rows = stmt.query_map([], |r| r.get(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

const SELECT: &str = "SELECT id, title, why, target_sessions, due_on, color, status FROM goals";

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Goal> {
    Ok(Goal {
        id: r.get(0)?,
        title: r.get(1)?,
        why: r.get(2)?,
        target_sessions: r.get(3)?,
        due_on: r.get(4)?,
        color: r.get(5)?,
        status: r.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_db;

    fn goal(title: &str) -> GoalInput<'_> {
        GoalInput {
            title,
            why: "because",
            target_sessions: 120,
            due_on: Some("2027-03-15"),
            color: "#F2B544",
        }
    }

    #[test]
    fn create_list_update_and_finish() {
        let conn = test_db();
        let a = create(&conn, &goal("Ship v1"), 1).unwrap();
        let b = create(&conn, &goal("Land a role"), 2).unwrap();
        set_status(&conn, &a, "done", 3).unwrap();
        update(
            &conn,
            &b,
            &GoalInput {
                target_sessions: 60,
                ..goal("Land a Rust role")
            },
            4,
        )
        .unwrap();

        let goals = list(&conn).unwrap();
        assert_eq!(
            goals.iter().map(|g| g.title.as_str()).collect::<Vec<_>>(),
            ["Land a Rust role", "Ship v1"]
        );
        assert_eq!(goals[0].target_sessions, 60);
        assert_eq!(goals[1].status, "done");
        assert_eq!(
            get(&conn, &a).unwrap().unwrap().due_on.as_deref(),
            Some("2027-03-15")
        );
    }

    #[test]
    fn rejects_an_unknown_status() {
        let conn = test_db();
        let a = create(&conn, &goal("x"), 1).unwrap();
        assert!(set_status(&conn, &a, "paused", 2).is_err());
    }

    #[test]
    fn deleting_a_goal_keeps_its_projects() {
        let conn = test_db();
        let g = create(&conn, &goal("x"), 1).unwrap();
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
        delete(&conn, &g, 2).unwrap();
        assert!(get(&conn, &g).unwrap().is_none());
        let project = crate::projects::list(&conn)
            .unwrap()
            .into_iter()
            .find(|x| x.id == p)
            .unwrap();
        assert_eq!(project.goal_id, None);
    }

    #[test]
    fn counts_sessions_through_tasks_and_projects() {
        let conn = test_db();
        let g = create(&conn, &goal("x"), 1).unwrap();
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
        let loose = crate::projects::create(
            &conn,
            &crate::projects::ProjectInput {
                name: "q",
                color: "#fff",
                goal_id: None,
            },
            1,
        )
        .unwrap();
        let t = crate::tasks::create(
            &conn,
            &crate::tasks::TaskInput::titled("a").in_project(&p),
            1,
        )
        .unwrap();
        let u = crate::tasks::create(
            &conn,
            &crate::tasks::TaskInput::titled("b").in_project(&loose),
            1,
        )
        .unwrap();
        for (task, started) in [(&t, 100), (&t, 5_000), (&u, 5_000)] {
            crate::test_session(&conn, task, started);
        }
        let c = counts(&conn, 1_000).unwrap();
        assert_eq!(c.get(&g), Some(&GoalCounts { done: 2, recent: 1 }));
        assert_eq!(
            c.len(),
            1,
            "sessions on goal-less projects count toward nothing"
        );
    }
}
