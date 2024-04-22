BEGIN;

CREATE TABLE chatrooms (
  id BIGINT NOT NULL,
  PRIMARY KEY (id),
  title TEXT,
  is_group BOOLEAN NOT NULL,
  joined_counter SMALLINT DEFAULT 1,
  joined_at timestamptz NOT NULL,
  left_at timestamptz
);

INSERT INTO
  chatrooms (id, title, is_group, joined_at)
VALUES
  (
    -4126100441,
    'my_botss',
    TRUE,
    '2024-04-22 08:11:57.713283+00'
  ),
  (
    220272763,
    NULL,
    FALSE,
    '2024-04-22 08:11:57.713283+00'
  );

COMMIT;
