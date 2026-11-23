//! "Still on it?" answers during focus sessions.

use rusqlite::{Connection, params};

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Response {
    Focused,
    Drifted,
}

impl Response {
    pub fn as_str(self) -> &'static str {
        match self {
            Response::Focused => "focused",
            Response::Drifted => "drifted",
        }
    }
}

/// Saves an answer. "Drifted" also adds a distraction to the session.
pub fn record(
    conn: &Connection,
    session_id: &str,
    at: i64,
    response: Response,
    now: i64,
) -> Result<String> {
    let tx = conn.unchecked_transaction()?;
    let id = tx.query_row(
        "INSERT INTO check_ins (session_id, at, response, updated_at)
         VALUES (?1, ?2, ?3, ?4) RETURNING id",
        params![session_id, at, response.as_str(), now],
        |row| row.get(0),
    )?;
    if response == Response::Drifted {
        crate::sessions::add_distraction(&tx, session_id, now)?;
    }
    tx.commit()?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions::{self, StartedSession};

    fn db_with_session() -> (Connection, String) {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::migrate(&mut conn, crate::MIGRATIONS).unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        let id = sessions::start(
            &conn,
            &StartedSession {
                kind: "deep",
                planned_secs: 3000,
                started_at: 1,
                intention: "",
                task_id: None,
                project_id: None,
            },
            1,
        )
        .unwrap();
        (conn, id)
    }

    fn count(conn: &Connection, sql: &str) -> u32 {
        conn.query_row(sql, [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn drifted_adds_a_distraction_and_focused_does_not() {
        let (conn, session) = db_with_session();
        record(&conn, &session, 900_000, Response::Focused, 900_000).unwrap();
        record(&conn, &session, 1_800_000, Response::Drifted, 1_800_000).unwrap();
        assert_eq!(count(&conn, "SELECT COUNT(*) FROM check_ins"), 2);
        assert_eq!(count(&conn, "SELECT distractions FROM sessions"), 1);
    }

    #[test]
    fn check_ins_go_with_their_session() {
        let (conn, session) = db_with_session();
        record(&conn, &session, 1, Response::Focused, 1).unwrap();
        sessions::discard(&conn, &session, 2).unwrap();
        assert_eq!(count(&conn, "SELECT COUNT(*) FROM check_ins"), 0);
    }

    #[test]
    fn refuses_an_unknown_session() {
        let (conn, _) = db_with_session();
        assert!(record(&conn, "nope", 1, Response::Focused, 1).is_err());
    }
}
