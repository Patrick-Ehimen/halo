//! Focus sessions.

use rusqlite::{Connection, params};

use crate::Result;

/// A finished session to save. Times are unix milliseconds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSession<'a> {
    /// `pomodoro`, `deep`, `flowtime` or `custom`.
    pub kind: &'a str,
    /// `timer` or `manual`.
    pub source: &'a str,
    pub planned_secs: u32,
    pub started_at: i64,
    pub ended_at: i64,
    pub focused_secs: u32,
    pub overflow_secs: u32,
    pub pauses: u32,
    pub intention: &'a str,
}

/// Saves a session and returns its id. It's marked as not yet synced.
pub fn insert(conn: &Connection, s: &NewSession<'_>, now: i64) -> Result<String> {
    let id = conn.query_row(
        "INSERT INTO sessions (kind, source, planned_secs, started_at, ended_at,
                               focused_secs, overflow_secs, pauses, intention, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         RETURNING id",
        params![
            s.kind,
            s.source,
            s.planned_secs,
            s.started_at,
            s.ended_at,
            s.focused_secs,
            s.overflow_secs,
            s.pauses,
            s.intention,
            now
        ],
        |row| row.get(0),
    )?;
    Ok(id)
}

/// Focused seconds and session count for sessions started in `[from, to)`.
pub fn totals(conn: &Connection, from: i64, to: i64) -> Result<(u32, u32)> {
    Ok(conn.query_row(
        "SELECT COALESCE(SUM(focused_secs), 0), COUNT(*)
         FROM sessions
         WHERE deleted_at IS NULL AND started_at >= ?1 AND started_at < ?2",
        params![from, to],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::migrate(&mut conn, crate::MIGRATIONS).unwrap();
        conn
    }

    fn session(started_at: i64, focused_secs: u32) -> NewSession<'static> {
        NewSession {
            kind: "deep",
            source: "timer",
            planned_secs: 3000,
            started_at,
            ended_at: started_at + i64::from(focused_secs) * 1000,
            focused_secs,
            overflow_secs: 0,
            pauses: 1,
            intention: "Write matching-engine tests",
        }
    }

    #[test]
    fn insert_returns_a_uuid_and_marks_unsynced() {
        let conn = db();
        let id = insert(&conn, &session(1_000, 1500), 5_000).unwrap();
        assert_eq!(id.len(), 36);
        assert_eq!(&id[14..15], "4");
        let (updated, synced): (i64, Option<i64>) = conn
            .query_row(
                "SELECT updated_at, synced_at FROM sessions WHERE id = ?1",
                [&id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(updated, 5_000);
        assert_eq!(synced, None);
    }

    #[test]
    fn ids_are_unique() {
        let conn = db();
        let a = insert(&conn, &session(1, 60), 1).unwrap();
        let b = insert(&conn, &session(1, 60), 1).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn rejects_an_unknown_kind() {
        let conn = db();
        let bad = NewSession {
            kind: "nap",
            ..session(1, 60)
        };
        assert!(insert(&conn, &bad, 1).is_err());
    }

    #[test]
    fn totals_cover_the_range_only() {
        let conn = db();
        insert(&conn, &session(100, 600), 1).unwrap();
        insert(&conn, &session(200, 900), 1).unwrap();
        insert(&conn, &session(300, 60), 1).unwrap();
        assert_eq!(totals(&conn, 100, 300).unwrap(), (1500, 2));
        assert_eq!(totals(&conn, 400, 500).unwrap(), (0, 0));
    }
}
