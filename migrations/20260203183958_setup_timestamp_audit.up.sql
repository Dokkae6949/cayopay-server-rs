create or replace function enforce_audit_timestamps()
returns trigger as $$
begin
    if tg_op = 'INSERT' then
        new.created_at = current_timestamp;
        new.updated_at = null;

        return new;
    end if;

    new.updated_at = current_timestamp;

    if new.created_at is distinct from old.created_at then
        new.created_at = old.created_at;
    end if;

    return new;
end;
$$ language plpgsql;
