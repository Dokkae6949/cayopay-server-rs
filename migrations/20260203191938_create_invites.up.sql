create table invites (
    id uuid primary key default uuidv7(),
    invitor_user_id uuid not null references users(id) on delete cascade,
    email text not null,
    token text not null unique,
    role text not null,
    status text not null default 'pending' check (status in ('pending', 'accepted', 'declined', 'revoked')),
    expires_at timestamptz not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz
);

create trigger invites_audit_timestamps
    before insert or update on invites
    for each row
    execute function enforce_audit_timestamps();
