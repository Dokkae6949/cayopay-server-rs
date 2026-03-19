-- Direct permission grants (replaces role-based permission checking).
--
-- resource_type: 'global' | 'shop'
-- resource_id:   NULL means ANY resource of that type; non-NULL means a specific resource.
-- permission:    plain-string code, e.g. 'settings.configure', 'product.create'.
create table granted_permissions (
    user_id       uuid  not null references users(id) on delete cascade,
    resource_type text  not null,
    resource_id   uuid,
    permission    text  not null,
    created_at    timestamptz not null default now(),

    -- The nil UUID (all-zeros) is reserved as the internal sentinel for "any resource"
    -- in the unique index below and must never be used as a real resource_id.
    constraint no_nil_resource_id
        check (resource_id <> '00000000-0000-0000-0000-000000000000')
);

-- Enforce uniqueness, treating NULL resource_id as a distinct, matchable "ANY" sentinel.
-- COALESCE maps NULL → the nil UUID so that two "any" grants for the same
-- (user, resource_type, permission) tuple are correctly rejected as duplicates.
create unique index uq_granted_permissions
    on granted_permissions (user_id, resource_type, coalesce(resource_id, '00000000-0000-0000-0000-000000000000'::uuid), permission);
