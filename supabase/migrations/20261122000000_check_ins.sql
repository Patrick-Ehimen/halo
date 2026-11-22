-- "Still on it?" answers during focus sessions. Mirrors the local
-- crates/store/migrations/0002_check_ins.sql.

create table public.check_ins (
  id         uuid primary key,
  user_id    uuid not null default auth.uid() references auth.users (id) on delete cascade,
  session_id uuid not null references public.sessions (id) on delete cascade,
  at         bigint not null,
  response   text not null check (response in ('focused', 'drifted')),
  updated_at bigint not null,
  deleted_at bigint
);

create index check_ins_user_session on public.check_ins (user_id, session_id);
create index check_ins_user_updated_at on public.check_ins (user_id, updated_at);

alter table public.check_ins enable row level security;

create policy "own check-ins" on public.check_ins
  for all using (user_id = auth.uid()) with check (user_id = auth.uid());

grant select, insert, update, delete on public.check_ins to authenticated;
revoke all on public.check_ins from anon;
