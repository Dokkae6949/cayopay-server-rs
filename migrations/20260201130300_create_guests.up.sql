drop table if exists guests;

create table guests (
    id uuid primary key,
    actor_id uuid not null unique references actors(id) on delete cascade
);
