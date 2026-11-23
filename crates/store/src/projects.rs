//! Projects, each under a goal or standalone.

use rusqlite::{Connection, OptionalExtension, params};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectInput<'a> {
    pub name: &'a str,
    pub color: &'a str,
    pub goal_id: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: String,
    pub goal_id: Option<String>,
    pub name: String,
    pub color: String,
}

pub fn create(conn: &Connection, p: &ProjectInput<'_>, now: i64) -> Result<String> {
    Ok(conn.query_row(
        "INSERT INTO projects (goal_id, name, color, position, updated_at)
         VALUES (?1, ?2, ?3, (SELECT COALESCE(MAX(position), 0) + 1 FROM projects), ?4)
         RETURNING id",
        params![p.goal_id, p.name, p.color, now],
        |r| r.get(0),
    )?)
}

pub fn update(conn: &Connection, id: &str, p: &ProjectInput<'_>, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE projects SET goal_id = ?2, name = ?3, color = ?4, updated_at = ?5, synced_at = NULL
         WHERE id = ?1",
        params![id, p.goal_id, p.name, p.color, now],
    )?;
    Ok(())
}

/// Hides a project; its tasks and sessions stay.
pub fn archive(conn: &Connection, id: &str, now: i64) -> Result<()> {
    conn.execute(
        "UPDATE projects SET archived_at = ?2, updated_at = ?2, synced_at = NULL WHERE id = ?1",
        params![id, now],
    )?;
    Ok(())
}

/// Active projects, in order.
pub fn list(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, goal_id, name, color FROM projects
         WHERE deleted_at IS NULL AND archived_at IS NULL ORDER BY position",
    )?;
    let rows = stmt.query_map([], row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// An active project by name, ignoring case and spaces vs hyphens
/// (`#job-hunt` finds "Job hunt").
pub fn find_by_name(conn: &Connection, name: &str) -> Result<Option<Project>> {
    let key = name.trim().to_lowercase().replace([' ', '_'], "-");
    Ok(conn
        .query_row(
            "SELECT id, goal_id, name, color FROM projects
             WHERE deleted_at IS NULL AND archived_at IS NULL
               AND replace(replace(lower(name), ' ', '-'), '_', '-') = ?1
             ORDER BY position LIMIT 1",
            [key],
            row,
        )
        .optional()?)
}

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: r.get(0)?,
        goal_id: r.get(1)?,
        name: r.get(2)?,
        color: r.get(3)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_db;

    #[test]
    fn create_find_and_archive() {
        let conn = test_db();
        let a = create(
            &conn,
            &ProjectInput {
                name: "Job hunt",
                color: "#C59CFF",
                goal_id: None,
            },
            1,
        )
        .unwrap();
        create(
            &conn,
            &ProjectInput {
                name: "meridian-rs",
                color: "#F2B544",
                goal_id: None,
            },
            2,
        )
        .unwrap();

        assert_eq!(
            find_by_name(&conn, "job-hunt").unwrap().map(|p| p.id),
            Some(a.clone())
        );
        assert_eq!(
            find_by_name(&conn, "MERIDIAN-RS").unwrap().map(|p| p.name),
            Some("meridian-rs".into())
        );
        assert_eq!(find_by_name(&conn, "nope").unwrap(), None);

        archive(&conn, &a, 3).unwrap();
        assert_eq!(list(&conn).unwrap().len(), 1);
        assert_eq!(find_by_name(&conn, "job hunt").unwrap(), None);
    }
}
