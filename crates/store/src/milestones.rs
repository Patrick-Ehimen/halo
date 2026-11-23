//! Milestones on the way to a goal.

use rusqlite::{Connection, params};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Milestone {
    pub id: String,
    pub goal_id: String,
    pub title: String,
    pub due_on: Option<String>,
    pub done_at: Option<i64>,
}

pub fn create(
    conn: &Connection,
    goal_id: &str,
    title: &str,
    due_on: Option<&str>,
    now: i64,
) -> Result<String> {
    Ok(conn.query_row(
        "INSERT INTO milestones (goal_id, title, due_on, position, updated_at)
         VALUES (?1, ?2, ?3,
                 (SELECT COALESCE(MAX(position), 0) + 1 FROM milestones WHERE goal_id = ?1), ?4)
         RETURNING id",
        params![goal_id, title, due_on, now],
        |r| r.get(0),
    )?)
}

pub fn update(
    conn: &Connection,
    id: &str,
    title: &str,
    due_on: Option<&str>,
    now: i64,
) -> Result<()> {
    conn.execute(
        "UPDATE milestones SET title = ?2, due_on = ?3, updated_at = ?4, synced_at = NULL WHERE id = ?1",
        params![id, title, due_on, now],
    )?;
    Ok(())
}

/// Marks done (`Some(time)`) or not done (`None`).
pub fn set_done(conn: &Connection, id: &str, done_at: Option<i64>, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE milestones SET done_at = ?2, updated_at = ?3, synced_at = NULL WHERE id = ?1",
        params![id, done_at, now],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE tasks SET milestone_id = NULL, updated_at = ?2, synced_at = NULL WHERE milestone_id = ?1",
        params![id, now],
    )?;
    crate::soft_or_hard_delete(conn, "milestones", id, now)
}

/// Every milestone, grouped by goal in order.
pub fn list(conn: &Connection) -> Result<Vec<Milestone>> {
    let mut stmt = conn.prepare(
        "SELECT id, goal_id, title, due_on, done_at FROM milestones
         WHERE deleted_at IS NULL ORDER BY goal_id, position",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Milestone {
            id: r.get(0)?,
            goal_id: r.get(1)?,
            title: r.get(2)?,
            due_on: r.get(3)?,
            done_at: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goals::{self, GoalInput};
    use crate::test_db;

    #[test]
    fn milestones_keep_their_order_and_state() {
        let conn = test_db();
        let g = goals::create(
            &conn,
            &GoalInput {
                title: "g",
                why: "",
                target_sessions: 10,
                due_on: None,
                color: "#fff",
            },
            1,
        )
        .unwrap();
        let a = create(&conn, &g, "Core", Some("2026-11-02"), 1).unwrap();
        let b = create(&conn, &g, "Benchmarks", None, 2).unwrap();
        set_done(&conn, &a, Some(9), 3).unwrap();
        update(&conn, &b, "Order book benchmarks", Some("2026-12-06"), 4).unwrap();

        let all = list(&conn).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].done_at, Some(9));
        assert_eq!(all[1].title, "Order book benchmarks");
        assert_eq!(all[1].due_on.as_deref(), Some("2026-12-06"));
    }

    #[test]
    fn deleting_a_milestone_unlinks_its_tasks() {
        let conn = test_db();
        let g = goals::create(
            &conn,
            &GoalInput {
                title: "g",
                why: "",
                target_sessions: 10,
                due_on: None,
                color: "#fff",
            },
            1,
        )
        .unwrap();
        let m = create(&conn, &g, "m", None, 1).unwrap();
        let mut input = crate::tasks::TaskInput::titled("t");
        input.milestone_id = Some(&m);
        let t = crate::tasks::create(&conn, &input, 1).unwrap();
        delete(&conn, &m, 2).unwrap();
        assert!(list(&conn).unwrap().is_empty());
        assert_eq!(
            crate::tasks::get(&conn, &t).unwrap().unwrap().milestone_id,
            None
        );
    }
}
