BEGIN;

ALTER TABLE users
ADD COLUMN first_name TEXT NULL;

alter table users
add column joined_at timestamptz null;

alter table users
add column last_updated timestamptz null;

update users
set
  first_name = 'beta-users',
  joined_at = NOW(),
  last_updated = NOW()
where
  first_name IS NULL;

ALTER TABLE users
ALTER COLUMN first_name
SET NOT NULL;

alter table users
add column last_name text;

alter table users
alter column joined_at
set not null;

alter table users
alter column last_updated
set not null;

COMMIT;
