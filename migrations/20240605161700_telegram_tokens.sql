create table if not exists telegram_tokens (
  telegram_token text not null primary key,
  telegram_user_id bigint unique not null,
  telegram_username text,
  expiry timestamptz not null
);

create table telegram_users (
  telegram_user_id bigint primary key,
  telegram_username text,
  user_id uuid not null references users (user_id) on delete cascade,
  joined_at timestamptz not null
);

create table if not exists telegram_whisperers (
  id serial primary key,
  telegram_user_id bigint not null references telegram_users (telegram_user_id) on delete cascade,
  telegram_chat_id bigint not null,
  unique (telegram_user_id, telegram_chat_id),
  registered timestamptz not null
);

alter table about_me
alter column about_me type text[] using array[about_me];
