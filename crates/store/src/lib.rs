//! Local SQLite storage. Every schema change is a numbered migration in
//! `migrations/`. Never edit an applied migration; add a new one.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::path::Path;

pub use rusqlite::Connection;

pub mod check_ins;
pub mod sessions;
pub mod settings;

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
