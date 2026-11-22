//! Key-value settings. Values are JSON text; the app owns their shape.

use rusqlite::{Connection, OptionalExtension, params};

use crate::Result;

pub fn get(conn: &Connection, key: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
            row.get(0)
        })
        .optional()?)
}

/// Inserts or replaces a setting and marks it as not yet synced.
pub fn set(conn: &Connection, key: &str, value: &str, now: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value, updated_at, synced_at)
         VALUES (?1, ?2, ?3, NULL)
         ON CONFLICT (key) DO UPDATE SET
           value = excluded.value, updated_at = excluded.updated_at, synced_at = NULL",
        params![key, value, now],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::migrate(&mut conn, crate::MIGRATIONS).unwrap();
        conn
    }

    #[test]
    fn missing_key_is_none() {
        assert_eq!(get(&db(), "timer").unwrap(), None);
    }

    #[test]
    fn set_then_get_and_overwrite() {
        let conn = db();
        set(&conn, "timer", r#"{"overflow":true}"#, 1).unwrap();
        conn.execute("UPDATE settings SET synced_at = 5", [])
            .unwrap();
        set(&conn, "timer", r#"{"overflow":false}"#, 2).unwrap();
        assert_eq!(
            get(&conn, "timer").unwrap().as_deref(),
            Some(r#"{"overflow":false}"#)
        );
        let (updated, synced): (i64, Option<i64>) = conn
            .query_row(
                "SELECT updated_at, synced_at FROM settings WHERE key = 'timer'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((updated, synced), (2, None));
    }
}
