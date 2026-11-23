-- Trading desk: the pre-trade checklist, its daily ticks, head checks, and
-- chart activity. The first three mirror supabase/migrations/
-- 20261123000000_trading.sql (minus user_id, plus synced_at). Chart activity
-- is browsing history, so it stays on this device and never syncs.
-- Dates are the user's local calendar day, YYYY-MM-DD. Times are unix ms.

CREATE TABLE checklist_items (
  id TEXT PRIMARY KEY NOT NULL DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    substr('89ab', 1 + (abs(random()) % 4), 1) ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    lower(hex(randomblob(6)))
  ),
  label TEXT NOT NULL,
  hint TEXT NOT NULL DEFAULT '',
  position INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER,
  synced_at INTEGER
);

-- One row per item ticked on a day. Unticking deletes it, so the checklist
-- starts empty every morning without a reset job.
CREATE TABLE checklist_runs (
  id TEXT PRIMARY KEY NOT NULL DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    substr('89ab', 1 + (abs(random()) % 4), 1) ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    lower(hex(randomblob(6)))
  ),
  item_id TEXT NOT NULL REFERENCES checklist_items (id) ON DELETE CASCADE,
  date TEXT NOT NULL,
  checked_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER,
  synced_at INTEGER
);
CREATE UNIQUE INDEX checklist_runs_item_date ON checklist_runs (item_id, date)
  WHERE deleted_at IS NULL;

CREATE TABLE head_checks (
  id TEXT PRIMARY KEY NOT NULL DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    substr('89ab', 1 + (abs(random()) % 4), 1) ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    lower(hex(randomblob(6)))
  ),
  date TEXT NOT NULL,
  at INTEGER NOT NULL,
  score INTEGER NOT NULL CHECK (score BETWEEN 1 AND 5),
  -- JSON array of factor labels, e.g. ["Slept well","FOMO"].
  factors TEXT NOT NULL DEFAULT '[]',
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER,
  synced_at INTEGER
);
CREATE INDEX head_checks_date ON head_checks (date, at);

-- Local only.
CREATE TABLE chart_activity (
  id INTEGER PRIMARY KEY,
  -- The rule it matched: "TradingView".
  label TEXT NOT NULL,
  app TEXT NOT NULL,
  host TEXT,
  started_at INTEGER NOT NULL,
  ended_at INTEGER NOT NULL CHECK (ended_at >= started_at)
);
CREATE INDEX chart_activity_started ON chart_activity (started_at);
