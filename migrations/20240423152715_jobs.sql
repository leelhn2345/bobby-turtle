CREATE TABLE jobs_cron (
  id SERIAL PRIMARY KEY,
  target BIGINT NOT NULL REFERENCES chatrooms (id),
  job_id UUID UNIQUE,
  type TEXT NOT NULL, -- 'morning-greeting', `night-greeting`, `wish`, etc...
  cron_str TEXT NOT NULL,
  message TEXT NOT NULL,
);

CREATE TABLE jobs_one_off (
  id SERIAL PRIMARY KEY,
  target BIGINT NOT NULL REFERENCES chatrooms (id),
  job_id UUID UNIQUE,
  type TEXT NOT NULL, -- `reminder`, `task`, etc...
  due TEXT NOT NULL,
  message TEXT NOT NULL
)
