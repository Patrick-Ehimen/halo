-- Signed-in users reach their own rows through the Data API; row-level
-- security (previous migration) limits them to user_id = auth.uid().
-- Anonymous requests get no access at all.
grant select, insert, update, delete on public.sessions to authenticated;
grant select, insert, update, delete on public.settings to authenticated;

revoke all on public.sessions from anon;
revoke all on public.settings from anon;
