create table transactions (
    id uuid primary key,
    sender_wallet_id uuid not null references wallets(id) on delete cascade,
    receiver_wallet_id uuid not null references wallets(id) on delete cascade,
    executor_actor_id uuid not null references actors(id) on delete cascade,

    -- Amount in Cents to avoid floating point issues
    amount integer not null,

    description text,

    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
)
