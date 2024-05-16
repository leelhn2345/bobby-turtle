create table job_experience (
  id serial primary key,
  company_name text not null,
  company_url text not null,
  job_title text not null,
  time_span text not null,
  description text[]
);

create table projects (
  id serial primary key,
  project_name text not null,
  project_url text not null,
  description text[]
);

create table job_skills (
  id bool primary key default true,
  languages text not null,
  tools text not null,
  frameworks text not null,
  others text not null,
  constraint check_one_row_id check (id = true)
);

create table about_me (
  id bool primary key default true,
  about_me text not null,
  constraint check_one_row_id check (id = true)
);
