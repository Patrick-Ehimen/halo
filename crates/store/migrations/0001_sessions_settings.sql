-- Focus sessions and key-value settings. Mirrors supabase/migrations/
-- 20261121000000_sessions_settings.sql, minus user_id and plus synced_at.
-- Timestamps are unix milliseconds (UTC). synced_at IS NULL means the row
-- still needs uploading.

CREATE TABLE sessions (
  -- UUID v4, generated here so rows can sync without an id clash.
  id TEXT PRIMARY KEY NOT NULL DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    substr('89ab', 1 + (abs(random()) % 4), 1) ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    lower(hex(randomblob(6)))
  ),
  kind TEXT NOT NULL CHECK (kind IN ('pomodoro', 'deep', 'flowtime', 'custom')),
  source TEXT NOT NULL DEFAULT 'timer' CHECK (source IN ('timer', 'manual')),
  planned_secs INTEGER NOT NULL CHECK (planned_secs >= 0),
  started_at INTEGER NOT NULL,
  ended_at INTEGER,
  focused_secs INTEGER NOT NULL DEFAULT 0,
  overflow_secs INTEGER NOT NULL DEFAULT 0,
  pauses INTEGER NOT NULL DEFAULT 0,
  intention TEXT NOT NULL DEFAULT '',
  project_id TEXT,
  task_id TEXT,
  strict INTEGER NOT NULL DEFAULT 0,
  rating INTEGER,
  distractions INTEGER NOT NULL DEFAULT 0,
  notes TEXT NOT NULL DEFAULT '',
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER,
  synced_at INTEGER
);

CREATE INDEX sessions_started_at ON sessions (started_at);
CREATE INDEX sessions_unsynced ON sessions (synced_at) WHERE synced_at IS NULL;

CREATE TABLE settings (
  key TEXT PRIMARY KEY NOT NULL,
  value TEXT NOT NULL, -- JSON
  updated_at INTEGER NOT NULL,
  synced_at INTEGER
);
