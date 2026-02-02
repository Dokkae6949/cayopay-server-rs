alter table users add column if not exists role text not null default 'undefined';
alter table invites add column if not exists role text not null default 'undefined';
