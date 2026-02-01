create table sessions (
    id uuid primary key,
    user_id uuid not null references users(id) on delete cascade,
    token text not null unique,
    expires_at timestamptz not null,
    created_at timestamptz not null default now()
);
