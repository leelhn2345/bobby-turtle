BEGIN;

delete from users
where
  username = 'admin';

ALTER TABLE users
ADD COLUMN first_name TEXT not NULL,
ADD COLUMN last_name TEXT not NULL,
add column joined_at timestamptz not null,
add column last_updated timestamptz not null,
add column permission_level text default 'member' not null check (permission_level in ('alpha', 'admin', 'member'));

COMMIT;
