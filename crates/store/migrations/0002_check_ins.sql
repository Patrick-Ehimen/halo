-- "Still on it?" answers during focus sessions. Mirrors
-- supabase/migrations/20261122000000_check_ins.sql, minus user_id and plus synced_at.
-- From this migration on, a session's row is written when it starts
-- (ended_at NULL) so check-ins can point at it.

CREATE TABLE check_ins (
  id TEXT PRIMARY KEY NOT NULL DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    substr('89ab', 1 + (abs(random()) % 4), 1) ||
    substr(lower(hex(randomblob(2))), 2) || '-' ||
    lower(hex(randomblob(6)))
  ),
  session_id TEXT NOT NULL REFERENCES sessions (id) ON DELETE CASCADE,
  at INTEGER NOT NULL,
  response TEXT NOT NULL CHECK (response IN ('focused', 'drifted')),
  updated_at INTEGER NOT NULL,
  deleted_at INTEGER,
  synced_at INTEGER
);

CREATE INDEX check_ins_session ON check_ins (session_id);
CREATE INDEX check_ins_unsynced ON check_ins (synced_at) WHERE synced_at IS NULL;
