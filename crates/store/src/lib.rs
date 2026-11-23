//! Local SQLite storage. Every schema change is a numbered migration in
//! `migrations/`. Never edit an applied migration; add a new one.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::path::Path;

pub use rusqlite::Connection;

pub mod check_ins;
pub mod goals;
pub mod milestones;
pub mod projects;
pub mod sessions;
pub mod settings;
pub mod tasks;
pub mod time_blocks;
pub mod trading;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Ordered migrations. Index + 1 is the schema version stored in `PRAGMA user_version`.
/// Add `migrations/NNNN_name.sql` and append `include_str!("../migrations/NNNN_name.sql")` here.
const MIGRATIONS: &[&str] = &[
    include_str!("../migrations/0001_sessions_settings.sql"),
    include_str!("../migrations/0002_check_ins.sql"),
    include_str!("../migrations/0003_planning.sql"),
    include_str!("../migrations/0004_trading.sql"),
];

pub fn open(path: &Path) -> Result<Connection> {
    let mut conn = Connection::open(path)?;
    configure(&conn)?;
    migrate(&mut conn, MIGRATIONS)?;
    Ok(conn)
}

fn configure(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}

/// Deletes a row that never synced; marks a synced one deleted so the
/// deletion syncs too. `table` is one of this crate's own table names.
pub(crate) fn soft_or_hard_delete(
    conn: &Connection,
    table: &str,
    id: &str,
    now: i64,
) -> Result<()> {
    conn.execute(
        &format!("DELETE FROM {table} WHERE id = ?1 AND synced_at IS NULL"),
        [id],
    )?;
    conn.execute(
        &format!(
            "UPDATE {table} SET deleted_at = ?2, updated_at = ?2, synced_at = NULL WHERE id = ?1"
        ),
        rusqlite::params![id, now],
    )?;
    Ok(())
}

/// A migrated in-memory database with foreign keys on, for tests.
#[cfg(test)]
pub(crate) fn test_db() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    configure(&conn).unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    conn
}

/// A finished one-minute session started from `task_id`, for tests.
#[cfg(test)]
pub(crate) fn test_session(conn: &Connection, task_id: &str, started_at: i64) {
    let id = sessions::start(
        conn,
        &sessions::StartedSession {
            kind: "deep",
            planned_secs: 3000,
            started_at,
            intention: "",
            task_id: Some(task_id),
            project_id: None,
        },
        started_at,
    )
    .unwrap();
    let p = sessions::Progress {
        planned_secs: 3000,
        focused_secs: 60,
        overflow_secs: 0,
        pauses: 0,
    };
    sessions::finish(conn, &id, started_at + 60_000, &p, started_at).unwrap();
}

pub fn schema_version(conn: &Connection) -> Result<u32> {
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

/// Applies every migration newer than the stored version, each in its own transaction.
fn migrate(conn: &mut Connection, migrations: &[&str]) -> Result<()> {
    let current = schema_version(conn)? as usize;
    for (index, sql) in migrations.iter().enumerate().skip(current) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        let version = i64::try_from(index + 1).unwrap_or(i64::MAX);
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MIGRATIONS: &[&str] = &[
        "CREATE TABLE a (id INTEGER PRIMARY KEY);",
        "CREATE TABLE b (id INTEGER PRIMARY KEY);",
    ];

    #[test]
    fn applies_migrations_in_order() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn, TEST_MIGRATIONS).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 2);
    }

    #[test]
    fn is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn, TEST_MIGRATIONS).unwrap();
        migrate(&mut conn, TEST_MIGRATIONS).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 2);
    }

    #[test]
    fn applies_only_new_migrations() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn, &TEST_MIGRATIONS[..1]).unwrap();
        migrate(&mut conn, TEST_MIGRATIONS).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 2);
    }

    #[test]
    fn failed_migration_rolls_back() {
        let mut conn = Connection::open_in_memory().unwrap();
        let bad = &[TEST_MIGRATIONS[0], "CREATE TABLE oops (;"];
        assert!(migrate(&mut conn, bad).is_err());
        assert_eq!(schema_version(&conn).unwrap(), 1);
    }
}
