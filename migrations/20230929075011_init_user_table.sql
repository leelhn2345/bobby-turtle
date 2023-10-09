CREATE TABLE users (
   user_id BIGINT NOT NULL PRIMARY KEY,
   user_type TEXT CHECK (user_type in ('admin', 'user')) NOT NULL,
   username TEXT,
   first_name TEXT,
   last_name TEXT
);
