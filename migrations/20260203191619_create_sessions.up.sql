create table sessions (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users(id) on delete cascade,
    token text not null unique,
    user_agent text,
    ip_address text,
    expires_at timestamptz not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz,

    constraint expires_after_created
        check (expires_at >= created_at)
);

create trigger sessions_audit_timestamps
    before insert or update on sessions
    for each row
    execute function enforce_audit_timestamps();
