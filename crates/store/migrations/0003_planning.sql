-- Planning: goals, their milestones, projects (under a goal or standalone),
-- tasks, and time blocks. Mirrors supabase/migrations/
-- 20261122010000_planning.sql, minus user_id and plus synced_at.
-- Dates (due_on, planned_on) are the user's local calendar day, YYYY-MM-DD.
-- Times are unix milliseconds. Sessions done per task or goal are counted
-- from the sessions table, never stored.

CREATE TABLE goals (
  id TEXT PRIMARY KEY NOT NULL DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    substr('89ab', 1 + (abs(random()) % 4), 1) ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    lower(hex(randomblob(6)))
  ),
  title TEXT NOT NULL,
  why TEXT NOT NULL DEFAULT '',
  target_sessions INTEGER NOT NULL CHECK (target_sessions >= 0),
  due_on TEXT,
  color TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'done')),
  position INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER,
  synced_at INTEGER
);

CREATE TABLE milestones (
  id TEXT PRIMARY KEY NOT NULL DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    substr('89ab', 1 + (abs(random()) % 4), 1) ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    lower(hex(randomblob(6)))
  ),
  goal_id TEXT NOT NULL REFERENCES goals (id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  due_on TEXT,
  done_at INTEGER,
  position INTEGER NOT NULL DEFAULT 0,
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER,
  synced_at INTEGER
);
CREATE INDEX milestones_goal ON milestones (goal_id);

CREATE TABLE projects (
  id TEXT PRIMARY KEY NOT NULL DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    substr('89ab', 1 + (abs(random()) % 4), 1) ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    lower(hex(randomblob(6)))
  ),
  goal_id TEXT REFERENCES goals (id) ON DELETE SET NULL,
  name TEXT NOT NULL,
  color TEXT NOT NULL,
  archived_at INTEGER,
  position INTEGER NOT NULL DEFAULT 0,
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER,
  synced_at INTEGER
);
CREATE INDEX projects_goal ON projects (goal_id);

CREATE TABLE tasks (
  id TEXT PRIMARY KEY NOT NULL DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    substr('89ab', 1 + (abs(random()) % 4), 1) ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    lower(hex(randomblob(6)))
  ),
  project_id TEXT REFERENCES projects (id) ON DELETE SET NULL,
  milestone_id TEXT REFERENCES milestones (id) ON DELETE SET NULL,
  parent_id TEXT REFERENCES tasks (id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  estimate_sessions INTEGER NOT NULL DEFAULT 1 CHECK (estimate_sessions >= 0),
  planned_on TEXT,
  due_at INTEGER,
  recurrence TEXT,
  completed_at INTEGER,
  position INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER,
  synced_at INTEGER
);
CREATE INDEX tasks_planned_on ON tasks (planned_on);
CREATE INDEX tasks_project ON tasks (project_id);
CREATE INDEX tasks_parent ON tasks (parent_id);

CREATE TABLE time_blocks (
  id TEXT PRIMARY KEY NOT NULL DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    substr('89ab', 1 + (abs(random()) % 4), 1) ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    lower(hex(randomblob(6)))
  ),
  task_id TEXT REFERENCES tasks (id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  starts_at INTEGER NOT NULL,
  ends_at INTEGER NOT NULL CHECK (ends_at > starts_at),
  kind TEXT NOT NULL DEFAULT 'focus' CHECK (kind IN ('focus', 'event', 'break')),
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER,
  synced_at INTEGER
);
CREATE INDEX time_blocks_starts_at ON time_blocks (starts_at);

CREATE INDEX sessions_task ON sessions (task_id);
