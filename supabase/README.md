# Supabase

Halo is offline-first: everything is saved to a local SQLite file first
(`crates/store`). When the user signs in, sessions and settings sync to this
Supabase project so they follow the user across Macs.

- `migrations/`: the Postgres schema, in Supabase CLI format. It mirrors the
  local tables in `crates/store/migrations/`, plus `user_id` and row-level
  security, so each user only ever sees their own rows.
- Sync rules: rows are upserted by `id`; the newest `updated_at` wins;
  deletes are soft (`deleted_at`) so they sync too.

## Setup

1. Create a Supabase project.
2. Apply the migrations: `supabase link --project-ref <ref>` then `supabase db push`.
3. Copy `.env.cloud.example` to `.env.cloud` in the repo root and fill in the
   project URL and anon key. They're read at build time. Never commit
   `.env.cloud`, and never put the service-role key in the app.

## Changing the schema

Every change is a new migration here **and** a matching local migration in
`crates/store/migrations/`. Never edit an applied migration.
