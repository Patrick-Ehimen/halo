//! Trading desk storage: the pre-trade checklist and its daily ticks, head
//! checks, and chart activity (local only).

use rusqlite::{Connection, OptionalExtension, params};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecklistItem {
    pub id: String,
    pub label: String,
    pub hint: String,
    /// Ticked on the day asked for.
    pub checked: bool,
}

/// The checklist in order, with each item's tick for `date` (YYYY-MM-DD).
pub fn checklist(conn: &Connection, date: &str) -> Result<Vec<ChecklistItem>> {
    let mut stmt = conn.prepare(
        "SELECT i.id, i.label, i.hint,
                EXISTS (SELECT 1 FROM checklist_runs r
                        WHERE r.item_id = i.id AND r.date = ?1 AND r.deleted_at IS NULL)
         FROM checklist_items i
         WHERE i.deleted_at IS NULL
         ORDER BY i.position, i.created_at",
    )?;
    let rows = stmt.query_map([date], |r| {
        Ok(ChecklistItem {
            id: r.get(0)?,
            label: r.get(1)?,
            hint: r.get(2)?,
            checked: r.get(3)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Whether the checklist has ever had items (deleted ones included), so the
/// defaults are only added once.
pub fn checklist_started(conn: &Connection) -> Result<bool> {
    Ok(
        conn.query_row("SELECT EXISTS (SELECT 1 FROM checklist_items)", [], |r| {
            r.get(0)
        })?,
    )
}

pub fn add_item(conn: &Connection, label: &str, hint: &str, now: i64) -> Result<String> {
    Ok(conn.query_row(
        "INSERT INTO checklist_items (label, hint, position, created_at, updated_at)
         VALUES (?1, ?2, (SELECT COALESCE(MAX(position), 0) + 1 FROM checklist_items), ?3, ?3)
         RETURNING id",
        params![label, hint, now],
        |r| r.get(0),
    )?)
}

pub fn remove_item(conn: &Connection, id: &str, now: i64) -> Result<()> {
    crate::soft_or_hard_delete(conn, "checklist_items", id, now)
}

/// Ticks or unticks an item for `date`.
pub fn set_checked(conn: &Connection, item_id: &str, date: &str, on: bool, now: i64) -> Result<()> {
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM checklist_runs WHERE item_id = ?1 AND date = ?2 AND deleted_at IS NULL",
            params![item_id, date],
            |r| r.get(0),
        )
        .optional()?;
    match (existing, on) {
        (None, true) => {
            conn.execute(
                "INSERT INTO checklist_runs (item_id, date, checked_at, updated_at)
                 VALUES (?1, ?2, ?3, ?3)",
                params![item_id, date, now],
            )?;
        }
        (Some(id), false) => crate::soft_or_hard_delete(conn, "checklist_runs", &id, now)?,
        _ => {}
    }
    Ok(())
}

/// Unticks everything for `date`.
pub fn reset_checklist(conn: &Connection, date: &str, now: i64) -> Result<()> {
    let ids: Vec<String> = conn
        .prepare("SELECT id FROM checklist_runs WHERE date = ?1 AND deleted_at IS NULL")?
        .query_map([date], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for id in ids {
        crate::soft_or_hard_delete(conn, "checklist_runs", &id, now)?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadCheck {
    pub at: i64,
    pub score: u8,
    /// A JSON array of factor labels.
    pub factors_json: String,
}

pub fn record_head_check(
    conn: &Connection,
    date: &str,
    at: i64,
    score: u8,
    factors_json: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO head_checks (date, at, score, factors, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?2)",
        params![date, at, score, factors_json],
    )?;
    Ok(())
}

/// The latest head check on `date`.
pub fn latest_head_check(conn: &Connection, date: &str) -> Result<Option<HeadCheck>> {
    Ok(conn
        .query_row(
            "SELECT at, score, factors FROM head_checks
             WHERE date = ?1 AND deleted_at IS NULL ORDER BY at DESC LIMIT 1",
            [date],
            |r| {
                Ok(HeadCheck {
                    at: r.get(0)?,
                    score: r.get(1)?,
                    factors_json: r.get(2)?,
                })
            },
        )
        .optional()?)
}

/// Takes back the head checks logged on `date`.
pub fn clear_head_checks(conn: &Connection, date: &str, now: i64) -> Result<()> {
    let ids: Vec<String> = conn
        .prepare("SELECT id FROM head_checks WHERE date = ?1 AND deleted_at IS NULL")?
        .query_map([date], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for id in ids {
        crate::soft_or_hard_delete(conn, "head_checks", &id, now)?;
    }
    Ok(())
}

/// The lowest head check on `date`: a 1 or 2 locks the rest of the day.
pub fn lowest_head_check(conn: &Connection, date: &str) -> Result<Option<u8>> {
    Ok(conn.query_row(
        "SELECT MIN(score) FROM head_checks WHERE date = ?1 AND deleted_at IS NULL",
        [date],
        |r| r.get(0),
    )?)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartActivity {
    pub label: String,
    pub app: String,
    pub host: Option<String>,
    pub started_at: i64,
    pub ended_at: i64,
}

/// Starts a stretch of chart time. Returns its row id, to extend it.
pub fn start_chart(
    conn: &Connection,
    label: &str,
    app: &str,
    host: Option<&str>,
    at: i64,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO chart_activity (label, app, host, started_at, ended_at) VALUES (?1, ?2, ?3, ?4, ?4)",
        params![label, app, host, at],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn extend_chart(conn: &Connection, id: i64, ended_at: i64) -> Result<()> {
    conn.execute(
        "UPDATE chart_activity SET ended_at = MAX(ended_at, ?2) WHERE id = ?1",
        params![id, ended_at],
    )?;
    Ok(())
}

/// Chart time overlapping `[from, to)`, oldest first.
pub fn chart_between(conn: &Connection, from: i64, to: i64) -> Result<Vec<ChartActivity>> {
    let mut stmt = conn.prepare(
        "SELECT label, app, host, started_at, ended_at FROM chart_activity
         WHERE ended_at > ?1 AND started_at < ?2 ORDER BY started_at",
    )?;
    let rows = stmt.query_map(params![from, to], |r| {
        Ok(ChartActivity {
            label: r.get(0)?,
            app: r.get(1)?,
            host: r.get(2)?,
            started_at: r.get(3)?,
            ended_at: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_checklist_starts_fresh_each_day() {
        let conn = crate::test_db();
        assert!(!checklist_started(&conn).unwrap());
        let a = add_item(&conn, "Key levels marked", "", 1).unwrap();
        let b = add_item(&conn, "Risk set", "Max 0.5%", 2).unwrap();
        assert!(checklist_started(&conn).unwrap());

        set_checked(&conn, &a, "2026-11-23", true, 10).unwrap();
        set_checked(&conn, &a, "2026-11-23", true, 11).unwrap(); // twice is fine
        let today = checklist(&conn, "2026-11-23").unwrap();
        assert_eq!(
            today.iter().map(|i| i.checked).collect::<Vec<_>>(),
            [true, false]
        );
        assert_eq!(today[1].hint, "Max 0.5%");
        // Next morning: nothing ticked.
        assert!(
            checklist(&conn, "2026-11-24")
                .unwrap()
                .iter()
                .all(|i| !i.checked)
        );

        set_checked(&conn, &b, "2026-11-23", true, 12).unwrap();
        set_checked(&conn, &a, "2026-11-23", false, 13).unwrap();
        let today = checklist(&conn, "2026-11-23").unwrap();
        assert_eq!(
            today.iter().map(|i| i.checked).collect::<Vec<_>>(),
            [false, true]
        );

        reset_checklist(&conn, "2026-11-23", 14).unwrap();
        assert!(
            checklist(&conn, "2026-11-23")
                .unwrap()
                .iter()
                .all(|i| !i.checked)
        );

        remove_item(&conn, &a, 15).unwrap();
        assert_eq!(checklist(&conn, "2026-11-23").unwrap().len(), 1);
        assert!(checklist_started(&conn).unwrap());
    }

    #[test]
    fn the_latest_head_check_counts() {
        let conn = crate::test_db();
        assert_eq!(latest_head_check(&conn, "2026-11-23").unwrap(), None);
        record_head_check(&conn, "2026-11-23", 100, 2, "[\"Took a loss today\"]").unwrap();
        record_head_check(&conn, "2026-11-23", 200, 4, "[]").unwrap();
        let h = latest_head_check(&conn, "2026-11-23").unwrap().unwrap();
        assert_eq!((h.score, h.at), (4, 200));
        assert_eq!(lowest_head_check(&conn, "2026-11-23").unwrap(), Some(2));
        assert_eq!(lowest_head_check(&conn, "2026-11-24").unwrap(), None);
        clear_head_checks(&conn, "2026-11-23", 400).unwrap();
        assert_eq!(latest_head_check(&conn, "2026-11-23").unwrap(), None);
        assert_eq!(latest_head_check(&conn, "2026-11-24").unwrap(), None);
        assert!(record_head_check(&conn, "2026-11-23", 300, 6, "[]").is_err());
    }

    #[test]
    fn chart_activity_extends_and_queries() {
        let conn = crate::test_db();
        let id = start_chart(
            &conn,
            "TradingView",
            "Google Chrome",
            Some("www.tradingview.com"),
            1_000,
        )
        .unwrap();
        extend_chart(&conn, id, 61_000).unwrap();
        extend_chart(&conn, id, 30_000).unwrap(); // never shrinks
        let rows = chart_between(&conn, 0, 100_000).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!((rows[0].started_at, rows[0].ended_at), (1_000, 61_000));
        assert!(chart_between(&conn, 61_000, 100_000).unwrap().is_empty());
    }
}
