-- Descriptions on tasks and projects, and an end time on tasks. Mirrors
-- supabase/migrations/20261123010000_descriptions.sql.
-- A task's ends_at (unix ms) is on its planned day, after due_at; with none,
-- the timeline sizes it from its session estimate.

ALTER TABLE tasks ADD COLUMN description TEXT NOT NULL DEFAULT '';
ALTER TABLE tasks ADD COLUMN ends_at INTEGER;
ALTER TABLE projects ADD COLUMN description TEXT NOT NULL DEFAULT '';
