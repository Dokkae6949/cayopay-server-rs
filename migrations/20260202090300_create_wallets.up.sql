create table wallets (
    id uuid primary key,
    owner_actor_id uuid not null references actors(id) on delete cascade,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);
