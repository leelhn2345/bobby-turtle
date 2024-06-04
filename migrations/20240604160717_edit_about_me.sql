begin;

alter table about_me
drop column about_me;

alter table about_me
add column about_me text[] not null;

commit;
