create table shops (
    id uuid primary key default uuidv7(),
    owner_user_id uuid references users(id) on delete set null,
    name text not null unique,
    created_at timestamptz not null default now(),
    updated_at timestamptz
);

create table shop_offerings (
    id uuid primary key default uuidv7(),
    shop_id uuid not null references shops(id) on delete cascade,
    name text not null unique,
    description text,
    price_cents int not null check (price_cents > 0),
    created_at timestamptz not null default now(),
    updated_at timestamptz
);

create table shop_members (
    id uuid primary key default uuidv7(),
    shop_id uuid not null references shops(id) on delete cascade,
    user_id uuid not null references users(id) on delete cascade,
    created_at timestamptz not null default now(),
    updated_at timestamptz,

    unique(shop_id, user_id)
);

create trigger shops_audit_timestamps
    before insert or update on shops
    for each row
    execute function enforce_audit_timestamps();

create trigger shop_offerings_audit_timestamps
    before insert or update on shop_offerings
    for each row
    execute function enforce_audit_timestamps();

create trigger shop_members_audit_timestamps
    before insert or update on shop_members
    for each row
    execute function enforce_audit_timestamps();
