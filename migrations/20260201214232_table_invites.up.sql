create table invites (
    id uuid primary key,
    created_by uuid not null references users(id) on delete cascade,
    email text not null unique,
    token text not null unique,
    expires_at timestamptz not null,
    created_at timestamptz not null default now()
);
