-- Trading desk: the pre-trade checklist, its daily ticks and head checks.
-- Mirrors the local crates/store/migrations/0004_trading.sql. Chart activity
-- is browsing history and stays on the device; it has no table here.

create table public.checklist_items (
  id         uuid primary key,
  user_id    uuid not null default auth.uid() references auth.users (id) on delete cascade,
  label      text not null,
  hint       text not null default '',
  position   integer not null default 0,
  created_at bigint not null,
  updated_at bigint not null,
  deleted_at bigint
);

create table public.checklist_runs (
  id         uuid primary key,
  user_id    uuid not null default auth.uid() references auth.users (id) on delete cascade,
  item_id    uuid not null references public.checklist_items (id) on delete cascade,
  date       date not null,
  checked_at bigint not null,
  updated_at bigint not null,
  deleted_at bigint
);

create table public.head_checks (
  id         uuid primary key,
  user_id    uuid not null default auth.uid() references auth.users (id) on delete cascade,
  date       date not null,
  at         bigint not null,
  score      integer not null check (score between 1 and 5),
  factors    jsonb not null default '[]'::jsonb,
  updated_at bigint not null,
  deleted_at bigint
);

create index checklist_items_user_updated_at on public.checklist_items (user_id, updated_at);
create index checklist_runs_user_updated_at on public.checklist_runs (user_id, updated_at);
create index checklist_runs_user_date on public.checklist_runs (user_id, date);
create index head_checks_user_updated_at on public.head_checks (user_id, updated_at);
create index head_checks_user_date on public.head_checks (user_id, date);

alter table public.checklist_items enable row level security;
create policy "own checklist_items" on public.checklist_items
  for all using (user_id = auth.uid()) with check (user_id = auth.uid());
grant select, insert, update, delete on public.checklist_items to authenticated;
revoke all on public.checklist_items from anon;

alter table public.checklist_runs enable row level security;
create policy "own checklist_runs" on public.checklist_runs
  for all using (user_id = auth.uid()) with check (user_id = auth.uid());
grant select, insert, update, delete on public.checklist_runs to authenticated;
revoke all on public.checklist_runs from anon;

alter table public.head_checks enable row level security;
create policy "own head_checks" on public.head_checks
  for all using (user_id = auth.uid()) with check (user_id = auth.uid());
grant select, insert, update, delete on public.head_checks to authenticated;
revoke all on public.head_checks from anon;
