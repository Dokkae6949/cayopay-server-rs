-- Create permissions table
-- Each permission defines an action on a subject, e.g. action="send" subject="invite"
create table permissions (
    id uuid primary key default uuidv7(),
    action text not null,
    subject text not null,
    description text,
    created_at timestamptz not null default now(),

    unique(action, subject)
);

-- Create roles table
-- Roles are dynamic, runtime-configurable, and can inherit from other roles
create table roles (
    id uuid primary key default uuidv7(),
    name text not null unique,
    description text,
    inherited_from_role_id uuid references roles(id) on delete set null,
    created_at timestamptz not null default now()
);

-- Create role_permissions table
-- Links roles to permissions with an optional scope
-- scope_kind: "global", "shop", "register", "event", etc.
-- scope_id: specific resource UUID (null means all resources of that kind)
create table role_permissions (
    id uuid primary key default uuidv7(),
    role_id uuid not null references roles(id) on delete cascade,
    permission_id uuid not null references permissions(id) on delete cascade,
    scope_kind text not null default 'global',
    scope_id uuid,
    created_at timestamptz not null default now(),

    unique nulls not distinct (role_id, permission_id, scope_kind, scope_id)
);

-- Create user_roles table
-- Users can have multiple roles
create table user_roles (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users(id) on delete cascade,
    role_id uuid not null references roles(id) on delete cascade,
    created_at timestamptz not null default now(),

    unique(user_id, role_id)
);

-- Insert default permissions
insert into permissions (action, subject, description) values
    ('configure', 'settings',   'Configure system settings'),
    ('send',      'invite',     'Send invites to new users'),
    ('view',      'invite',     'View all invites'),
    ('remove',    'user',       'Remove users from the system'),
    ('read',      'user',       'Read user details'),
    ('remove',    'guest',      'Remove guests from the system'),
    ('read',      'guest',      'Read guest details');

-- Insert default roles
insert into roles (name, description) values
    ('owner', 'Full system access'),
    ('admin', 'Administrative access');

-- Assign all permissions to owner role (global scope)
insert into role_permissions (role_id, permission_id)
select r.id, p.id
from roles r, permissions p
where r.name = 'owner';

-- Assign permissions to admin role (all except configure:settings)
insert into role_permissions (role_id, permission_id)
select r.id, p.id
from roles r, permissions p
where r.name = 'admin'
  and not (p.action = 'configure' and p.subject = 'settings');

-- Migrate existing user roles from the old role column to user_roles table
insert into user_roles (user_id, role_id)
select u.id, r.id
from users u
join roles r on r.name = u.role
where u.role in ('owner', 'admin');

-- Drop old role column from users (now handled via user_roles)
alter table users drop column role;
