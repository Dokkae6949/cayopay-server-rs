drop table if exists users;

create table users (
    id uuid primary key,
    actor_id uuid not null unique references actors(id) on delete cascade,
    email text not null unique,
    password_hash text not null,
    first_name text not null,
    last_name text not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);
