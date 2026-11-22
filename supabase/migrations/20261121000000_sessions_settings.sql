-- Halo cloud schema: mirrors the local SQLite tables (crates/store/migrations)
-- so signed-in users' sessions and settings sync across devices.
-- Local SQLite stays the source of truth; rows are upserted by id, and the
-- newest updated_at wins. Timestamps are unix milliseconds, as locally.

create table public.sessions (
  id            uuid primary key,
  user_id       uuid not null default auth.uid() references auth.users (id) on delete cascade,
  kind          text not null check (kind in ('pomodoro', 'deep', 'flowtime', 'custom')),
  source        text not null default 'timer' check (source in ('timer', 'manual')),
  planned_secs  integer not null check (planned_secs >= 0),
  started_at    bigint not null,
  ended_at      bigint,
  focused_secs  integer not null default 0,
  overflow_secs integer not null default 0,
  pauses        integer not null default 0,
  intention     text not null default '',
  project_id    uuid,
  task_id       uuid,
  strict        boolean not null default false,
  rating        smallint,
  distractions  integer not null default 0,
  notes         text not null default '',
  updated_at    bigint not null,
  deleted_at    bigint
);

create index sessions_user_started_at on public.sessions (user_id, started_at);
create index sessions_user_updated_at on public.sessions (user_id, updated_at);

create table public.settings (
  user_id    uuid not null default auth.uid() references auth.users (id) on delete cascade,
  key        text not null,
  value      jsonb not null,
  updated_at bigint not null,
  primary key (user_id, key)
);

-- Row-level security: every user reads and writes only their own rows.
alter table public.sessions enable row level security;
alter table public.settings enable row level security;

create policy "own sessions" on public.sessions
  for all using (user_id = auth.uid()) with check (user_id = auth.uid());

create policy "own settings" on public.settings
  for all using (user_id = auth.uid()) with check (user_id = auth.uid());
