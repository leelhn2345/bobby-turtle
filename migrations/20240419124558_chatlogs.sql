CREATE TABLE chatlogs (
  id UUID NOT NULL,
  PRIMARY KEY (id),
  message_id BIGINT NOT NULL,
  is_group BOOLEAN NOT NULL,
  joined_counter SMALLINT DEFAULT 1,
  joined_at timestamptz NOT NULL,
  left_at timestamptz
);
