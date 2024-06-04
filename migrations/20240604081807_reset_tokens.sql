create table reset_password_tokens (
  reset_token text not null primary key,
  user_id uuid unique not null references users (user_id) on delete cascade,
  expires timestamptz not null
)
