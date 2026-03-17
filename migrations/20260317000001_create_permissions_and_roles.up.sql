-- Create roles table
-- Roles are dynamic, runtime-configurable, and can inherit from other roles.
create table roles (
    id uuid primary key default uuidv7(),
    name text not null unique,
    description text,
    inherited_from_role_id uuid references roles(id) on delete set null,
    created_at timestamptz not null default now()
);

-- Create role_permissions table
-- Links roles to code-defined permissions (stored by their string code) with an
-- optional resource scope.
--
-- scope_kind: "shop", "register", "event", etc.  NULL means global (no restriction).
-- scope_id:   specific resource UUID.  NULL means all resources of that kind.
create table role_permissions (
    id uuid primary key default uuidv7(),
    role_id uuid not null references roles(id) on delete cascade,
    permission text not null,
    scope_kind text,
    scope_id uuid,
    created_at timestamptz not null default now(),

    unique nulls not distinct (role_id, permission, scope_kind, scope_id)
);

-- Create user_roles table
-- Users can hold multiple roles.
create table user_roles (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users(id) on delete cascade,
    role_id uuid not null references roles(id) on delete cascade,
    created_at timestamptz not null default now(),

    unique(user_id, role_id)
);

-- Insert default roles
insert into roles (name, description) values
    ('owner', 'Full system access'),
    ('admin', 'Administrative access');

-- Assign all permissions to the owner role (global scope — scope_kind IS NULL)
insert into role_permissions (role_id, permission)
select r.id, p.code
from roles r
cross join (values
    ('settings.configure'),
    ('invite.send'),
    ('invite.view'),
    ('user.remove'),
    ('user.read'),
    ('guest.remove'),
    ('guest.read')
) as p(code)
where r.name = 'owner';

-- Assign permissions to admin role (all except settings.configure)
insert into role_permissions (role_id, permission)
select r.id, p.code
from roles r
cross join (values
    ('invite.send'),
    ('invite.view'),
    ('user.remove'),
    ('user.read'),
    ('guest.remove'),
    ('guest.read')
) as p(code)
where r.name = 'admin';

-- Migrate existing user roles from the old role column to user_roles table
insert into user_roles (user_id, role_id)
select u.id, r.id
from users u
join roles r on r.name = u.role
where u.role in ('owner', 'admin');

-- Drop old role column from users (now handled via user_roles)
alter table users drop column role;
