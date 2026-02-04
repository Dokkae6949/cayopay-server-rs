create table actors (
    id uuid primary key default uuidv7(),
    created_at timestamptz not null default now(),
    updated_at timestamptz
);

create table users (
    id uuid primary key default uuidv7(),
    actor_id uuid not null unique references actors(id) on delete cascade,
    email text not null unique,
    password_hash text not null,
    first_name text not null,
    last_name text not null,
    role text not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz
);

create table guests (
    id uuid primary key default uuidv7(),
    actor_id uuid not null unique references actors(id) on delete cascade,
    email text,
    verified boolean not null default false,
    created_at timestamptz not null default now(),
    updated_at timestamptz
);

create trigger actors_audit_timestamps
    before insert or update on actors
    for each row
    execute function enforce_audit_timestamps();

create trigger users_audit_timestamps
    before insert or update on users
    for each row
    execute function enforce_audit_timestamps();

create trigger guests_audit_timestamps
    before insert or update on guests
    for each row
    execute function enforce_audit_timestamps();
