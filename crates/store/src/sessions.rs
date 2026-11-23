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

/// A session as it starts: written right away (with `ended_at` NULL) so
/// check-ins can point at it and a crash doesn't lose it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartedSession<'a> {
    pub kind: &'a str,
    pub planned_secs: u32,
    pub started_at: i64,
    pub intention: &'a str,
    /// The task it was started from, if any, and that task's project.
    pub task_id: Option<&'a str>,
    pub project_id: Option<&'a str>,
}

/// How far a session got: written as it runs and when it ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub planned_secs: u32,
    pub focused_secs: u32,
    pub overflow_secs: u32,
    pub pauses: u32,
}

pub fn start(conn: &Connection, s: &StartedSession<'_>, now: i64) -> Result<String> {
    let id = conn.query_row(
        "INSERT INTO sessions (kind, source, planned_secs, started_at, intention, task_id,
                               project_id, updated_at)
         VALUES (?1, 'timer', ?2, ?3, ?4, ?5, ?6, ?7)
         RETURNING id",
        params![
            s.kind,
            s.planned_secs,
            s.started_at,
            s.intention,
            s.task_id,
            s.project_id,
            now
        ],
        |row| row.get(0),
    )?;
    Ok(id)
}

/// Records progress of a running session.
pub fn update_progress(conn: &Connection, id: &str, p: &Progress, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET planned_secs = ?2, focused_secs = ?3, overflow_secs = ?4,
                             pauses = ?5, updated_at = ?6, synced_at = NULL
         WHERE id = ?1 AND ended_at IS NULL",
        params![
            id,
            p.planned_secs,
            p.focused_secs,
            p.overflow_secs,
            p.pauses,
            now
        ],
    )?;
    Ok(())
}

pub fn finish(conn: &Connection, id: &str, ended_at: i64, p: &Progress, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET ended_at = ?2, planned_secs = ?3, focused_secs = ?4,
                             overflow_secs = ?5, pauses = ?6, updated_at = ?7, synced_at = NULL
         WHERE id = ?1",
        params![
            id,
            ended_at,
            p.planned_secs,
            p.focused_secs,
            p.overflow_secs,
            p.pauses,
            now
        ],
    )?;
    Ok(())
}

/// Drops a session that shouldn't be kept (an accidental start). Rows never
/// synced are deleted; synced ones are marked deleted so the deletion syncs.
pub fn discard(conn: &Connection, id: &str, now: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM sessions WHERE id = ?1 AND synced_at IS NULL",
        [id],
    )?;
    conn.execute(
        "UPDATE sessions SET deleted_at = ?2, updated_at = ?2, synced_at = NULL WHERE id = ?1",
        params![id, now],
    )?;
    Ok(())
}

/// A note on a session, written during or after it.
pub fn set_notes(conn: &Connection, id: &str, notes: &str, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET notes = ?2, updated_at = ?3, synced_at = NULL WHERE id = ?1",
        params![id, notes, now],
    )?;
    Ok(())
}

/// "I drifted": one more distraction on this session.
pub fn add_distraction(conn: &Connection, id: &str, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET distractions = distractions + 1, updated_at = ?2, synced_at = NULL
         WHERE id = ?1",
        params![id, now],
    )?;
    Ok(())
}

/// Closes sessions left open by a crash or a forced quit: ends them where
/// their last recorded progress stopped, and drops ones under a minute.
/// Returns how many were kept.
pub fn close_abandoned(conn: &Connection, now: i64) -> Result<usize> {
    conn.execute(
        "DELETE FROM sessions WHERE ended_at IS NULL AND focused_secs < 60 AND synced_at IS NULL",
        [],
    )?;
    let kept = conn.execute(
        "UPDATE sessions SET ended_at = started_at + focused_secs * 1000,
                             updated_at = ?1, synced_at = NULL
         WHERE ended_at IS NULL",
        [now],
    )?;
    Ok(kept)
}

/// Focused seconds and finished-session count for sessions started in `[from, to)`.
pub fn totals(conn: &Connection, from: i64, to: i64) -> Result<(u32, u32)> {
    Ok(conn.query_row(
        "SELECT COALESCE(SUM(focused_secs), 0), COUNT(*)
         FROM sessions
         WHERE deleted_at IS NULL AND ended_at IS NOT NULL
           AND started_at >= ?1 AND started_at < ?2",
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

    fn started(started_at: i64) -> StartedSession<'static> {
        StartedSession {
            kind: "deep",
            planned_secs: 3000,
            started_at,
            intention: "Write tests",
            task_id: None,
            project_id: None,
        }
    }

    fn progress(focused_secs: u32) -> Progress {
        Progress {
            planned_secs: 3000,
            focused_secs,
            overflow_secs: 0,
            pauses: 0,
        }
    }

    fn ended_at(conn: &Connection, id: &str) -> Option<i64> {
        conn.query_row("SELECT ended_at FROM sessions WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .unwrap()
    }

    #[test]
    fn a_session_runs_then_finishes() {
        let conn = db();
        let id = start(&conn, &started(1_000), 1_000).unwrap();
        assert_eq!(ended_at(&conn, &id), None);
        // Running sessions don't count toward today yet.
        assert_eq!(totals(&conn, 0, 10_000_000).unwrap(), (0, 0));

        update_progress(&conn, &id, &progress(600), 2_000).unwrap();
        finish(&conn, &id, 3_001_000, &progress(3000), 3_001_000).unwrap();
        assert_eq!(ended_at(&conn, &id), Some(3_001_000));
        assert_eq!(totals(&conn, 0, 10_000_000).unwrap(), (3000, 1));
    }

    #[test]
    fn discard_deletes_unsynced_and_soft_deletes_synced() {
        let conn = db();
        let a = start(&conn, &started(1), 1).unwrap();
        discard(&conn, &a, 2).unwrap();
        let n: u32 = conn
            .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0);

        let b = start(&conn, &started(1), 1).unwrap();
        conn.execute("UPDATE sessions SET synced_at = 5", [])
            .unwrap();
        discard(&conn, &b, 9).unwrap();
        let (deleted, synced): (Option<i64>, Option<i64>) = conn
            .query_row(
                "SELECT deleted_at, synced_at FROM sessions WHERE id = ?1",
                [&b],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((deleted, synced), (Some(9), None));
    }

    #[test]
    fn distractions_add_up() {
        let conn = db();
        let id = start(&conn, &started(1), 1).unwrap();
        add_distraction(&conn, &id, 2).unwrap();
        add_distraction(&conn, &id, 3).unwrap();
        let n: u32 = conn
            .query_row(
                "SELECT distractions FROM sessions WHERE id = ?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn close_abandoned_keeps_real_progress_and_drops_blips() {
        let conn = db();
        let real = start(&conn, &started(1_000), 1_000).unwrap();
        update_progress(&conn, &real, &progress(1200), 2_000).unwrap();
        let blip = start(&conn, &started(5_000), 5_000).unwrap();
        update_progress(&conn, &blip, &progress(20), 6_000).unwrap();

        assert_eq!(close_abandoned(&conn, 9_000).unwrap(), 1);
        assert_eq!(ended_at(&conn, &real), Some(1_000 + 1_200_000));
        let blips: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE id = ?1",
                [&blip],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(blips, 0);
    }
}
