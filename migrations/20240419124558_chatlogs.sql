CREATE TABLE chatlogs (
  id SERIAL PRIMARY KEY,
  message_id BIGINT REFERENCES chatrooms (id),
  name TEXT,
  role TEXT NOT NULL,
  content TEXT NOT NULL,
  datetime timestamptz NOT NULL
);
