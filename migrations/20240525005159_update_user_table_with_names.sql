BEGIN;

ALTER TABLE users
ADD COLUMN first_name TEXT NULL;

update users
set
  first_name = 'beta-users'
where
  first_name IS NULL;

ALTER TABLE users
ALTER COLUMN first_name
SET NOT NULL;

alter table users
add column last_name text;

COMMIT;
