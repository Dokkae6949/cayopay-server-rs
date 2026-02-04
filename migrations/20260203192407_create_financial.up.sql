create table wallets (
    id uuid primary key default uuidv7(),
    owner_actor_id uuid references actors(id) on delete set null,
    label text unique,
    allow_overdraft boolean not null default false,
    created_at timestamptz not null default now(),
    updated_at timestamptz
);

create table transactions (
    id uuid primary key default uuidv7(),
    source_wallet_id uuid not null references wallets(id) on delete cascade,
    destination_wallet_id uuid not null references wallets(id) on delete cascade,
    executor_actor_id uuid references actors(id) on delete set null,
    amount_cents int not null check (amount_cents > 0),
    description text,
    created_at timestamptz not null default now(),
    updated_at timestamptz
);

create trigger wallets_audit_timestamps
    before insert or update on wallets
    for each row
    execute function enforce_audit_timestamps();

create trigger transactions_audit_timestamps
    before insert or update on transactions
    for each row
    execute function enforce_audit_timestamps();
