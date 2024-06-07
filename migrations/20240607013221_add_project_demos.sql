begin;

drop table if exists projects;

create table projects (
  id serial primary key,
  name text not null,
  url text not null,
  description text[]
);

create type publicity as enum('public', 'member');

create table project_demos (
  id serial primary key,
  name text not null unique,
  url text not null,
  summary text not null,
  publicity publicity not null
);

commit;
