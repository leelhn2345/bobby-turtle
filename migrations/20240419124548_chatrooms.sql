CREATE TABLE chatrooms (
  id BIGINT NOT NULL,
  PRIMARY KEY (id),
  title TEXT,
  is_group BOOLEAN NOT NULL,
  joined_counter SMALLINT DEFAULT 1,
  joined_at timestamptz NOT NULL,
  left_at timestamptz
);
