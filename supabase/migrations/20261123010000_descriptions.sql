-- Descriptions on tasks and projects, and an end time on tasks. Mirrors the
-- local crates/store/migrations/0005_descriptions.sql. The tables' row-level
-- security policies cover the new columns.

alter table public.tasks add column description text not null default '';
alter table public.tasks add column ends_at bigint;
alter table public.projects add column description text not null default '';
