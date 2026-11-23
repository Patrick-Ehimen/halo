//! Time blocks on the day's timeline.

use rusqlite::{Connection, params};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockInput<'a> {
    pub task_id: Option<&'a str>,
    pub title: &'a str,
    pub starts_at: i64,
    pub ends_at: i64,
    /// `focus`, `event` or `break`.
    pub kind: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeBlock {
    pub id: String,
    pub task_id: Option<String>,
    pub title: String,
    pub starts_at: i64,
    pub ends_at: i64,
    pub kind: String,
}

pub fn create(conn: &Connection, b: &BlockInput<'_>, now: i64) -> Result<String> {
    Ok(conn.query_row(
        "INSERT INTO time_blocks (task_id, title, starts_at, ends_at, kind, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6) RETURNING id",
        params![b.task_id, b.title, b.starts_at, b.ends_at, b.kind, now],
        |r| r.get(0),
    )?)
}

/// Moves or resizes a block.
pub fn reschedule(
    conn: &Connection,
    id: &str,
    starts_at: i64,
    ends_at: i64,
    now: i64,
) -> Result<()> {
    conn.execute(
        "UPDATE time_blocks SET starts_at = ?2, ends_at = ?3, updated_at = ?4, synced_at = NULL
         WHERE id = ?1",
        params![id, starts_at, ends_at, now],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str, now: i64) -> Result<()> {
    crate::soft_or_hard_delete(conn, "time_blocks", id, now)
}

/// Blocks overlapping `[from, to)`, in time order.
pub fn list_between(conn: &Connection, from: i64, to: i64) -> Result<Vec<TimeBlock>> {
    let mut stmt = conn.prepare(
        "SELECT id, task_id, title, starts_at, ends_at, kind FROM time_blocks
         WHERE deleted_at IS NULL AND starts_at < ?2 AND ends_at > ?1
         ORDER BY starts_at",
    )?;
    let rows = stmt.query_map(params![from, to], |r| {
        Ok(TimeBlock {
            id: r.get(0)?,
            task_id: r.get(1)?,
            title: r.get(2)?,
            starts_at: r.get(3)?,
            ends_at: r.get(4)?,
            kind: r.get(5)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_db;

    fn block(title: &str, starts_at: i64, ends_at: i64) -> BlockInput<'_> {
        BlockInput {
            task_id: None,
            title,
            starts_at,
            ends_at,
            kind: "focus",
        }
    }

    #[test]
    fn lists_blocks_overlapping_the_range() {
        let conn = test_db();
        create(&conn, &block("before", 0, 100), 1).unwrap();
        create(&conn, &block("spans in", 50, 150), 1).unwrap();
        let inside = create(&conn, &block("inside", 120, 180), 1).unwrap();
        create(&conn, &block("after", 200, 300), 1).unwrap();
        let titles: Vec<String> = list_between(&conn, 100, 200)
            .unwrap()
            .into_iter()
            .map(|b| b.title)
            .collect();
        assert_eq!(titles, ["spans in", "inside"]);

        reschedule(&conn, &inside, 250, 280, 2).unwrap();
        assert_eq!(list_between(&conn, 100, 200).unwrap().len(), 1);
        delete(&conn, &inside, 3).unwrap();
        assert_eq!(list_between(&conn, 0, 1000).unwrap().len(), 3);
    }

    #[test]
    fn rejects_an_empty_block() {
        let conn = test_db();
        assert!(create(&conn, &block("x", 10, 10), 1).is_err());
    }
}
