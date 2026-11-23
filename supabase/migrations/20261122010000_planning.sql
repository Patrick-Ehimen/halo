-- Planning: goals, milestones, projects, tasks, time blocks. Mirrors the
-- local crates/store/migrations/0003_planning.sql.

create table public.goals (
  id              uuid primary key,
  user_id         uuid not null default auth.uid() references auth.users (id) on delete cascade,
  title           text not null,
  why             text not null default '',
  target_sessions integer not null check (target_sessions >= 0),
  due_on          date,
  color           text not null,
  status          text not null default 'active' check (status in ('active', 'done')),
  position        integer not null default 0,
  created_at      bigint not null,
  updated_at      bigint not null,
  deleted_at      bigint
);

create table public.milestones (
  id         uuid primary key,
  user_id    uuid not null default auth.uid() references auth.users (id) on delete cascade,
  goal_id    uuid not null references public.goals (id) on delete cascade,
  title      text not null,
  due_on     date,
  done_at    bigint,
  position   integer not null default 0,
  updated_at bigint not null,
  deleted_at bigint
);

create table public.projects (
  id          uuid primary key,
  user_id     uuid not null default auth.uid() references auth.users (id) on delete cascade,
  goal_id     uuid references public.goals (id) on delete set null,
  name        text not null,
  color       text not null,
  archived_at bigint,
  position    integer not null default 0,
  updated_at  bigint not null,
  deleted_at  bigint
);

create table public.tasks (
  id                uuid primary key,
  user_id           uuid not null default auth.uid() references auth.users (id) on delete cascade,
  project_id        uuid references public.projects (id) on delete set null,
  milestone_id      uuid references public.milestones (id) on delete set null,
  parent_id         uuid references public.tasks (id) on delete cascade,
  title             text not null,
  estimate_sessions integer not null default 1 check (estimate_sessions >= 0),
  planned_on        date,
  due_at            bigint,
  recurrence        text,
  completed_at      bigint,
  position          integer not null default 0,
  created_at        bigint not null,
  updated_at        bigint not null,
  deleted_at        bigint
);

create table public.time_blocks (
  id         uuid primary key,
  user_id    uuid not null default auth.uid() references auth.users (id) on delete cascade,
  task_id    uuid references public.tasks (id) on delete cascade,
  title      text not null,
  starts_at  bigint not null,
  ends_at    bigint not null check (ends_at > starts_at),
  kind       text not null default 'focus' check (kind in ('focus', 'event', 'break')),
  updated_at bigint not null,
  deleted_at bigint
);

create index goals_user_updated_at on public.goals (user_id, updated_at);
create index milestones_user_updated_at on public.milestones (user_id, updated_at);
create index projects_user_updated_at on public.projects (user_id, updated_at);
create index tasks_user_updated_at on public.tasks (user_id, updated_at);
create index tasks_user_planned_on on public.tasks (user_id, planned_on);
create index time_blocks_user_starts_at on public.time_blocks (user_id, starts_at);

-- Sessions point at their task and project from now on.
alter table public.sessions
  add constraint sessions_task_fk foreign key (task_id) references public.tasks (id) on delete set null,
  add constraint sessions_project_fk foreign key (project_id) references public.projects (id) on delete set null;

alter table public.goals enable row level security;
create policy "own goals" on public.goals
  for all using (user_id = auth.uid()) with check (user_id = auth.uid());
grant select, insert, update, delete on public.goals to authenticated;
revoke all on public.goals from anon;

alter table public.milestones enable row level security;
create policy "own milestones" on public.milestones
  for all using (user_id = auth.uid()) with check (user_id = auth.uid());
grant select, insert, update, delete on public.milestones to authenticated;
revoke all on public.milestones from anon;

alter table public.projects enable row level security;
create policy "own projects" on public.projects
  for all using (user_id = auth.uid()) with check (user_id = auth.uid());
grant select, insert, update, delete on public.projects to authenticated;
revoke all on public.projects from anon;

alter table public.tasks enable row level security;
create policy "own tasks" on public.tasks
  for all using (user_id = auth.uid()) with check (user_id = auth.uid());
grant select, insert, update, delete on public.tasks to authenticated;
revoke all on public.tasks from anon;

alter table public.time_blocks enable row level security;
create policy "own time_blocks" on public.time_blocks
  for all using (user_id = auth.uid()) with check (user_id = auth.uid());
grant select, insert, update, delete on public.time_blocks to authenticated;
revoke all on public.time_blocks from anon;
