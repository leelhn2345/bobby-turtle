begin;

alter table users
add column verified boolean default false;

update users
set
  verified = true;

alter table users
alter column verified
set not null;

commit;

create table verification_tokens (
  verification_token text not null primary key,
  user_id uuid not null references users (user_id)
);
