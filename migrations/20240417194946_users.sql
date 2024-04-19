-- Add migration script here
CREATE TABLE users (
  id BIGINT NOT NULL,
  PRIMARY KEY (id),
  first_name TEXT NOT NULL,
  last_name TEXT,
  username TEXT,
  role TEXT NOT NULL,
  joined_at timestamptz NOT NULL
);
