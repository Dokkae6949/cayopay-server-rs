-- Restore role column on users
alter table users add column role text not null default 'undefined';

-- Migrate user_roles data back to the old role column.
-- When a user has multiple roles, pick the highest-privilege one
-- (owner > admin > anything else) deterministically.
update users u
set role = r.name
from (
  select distinct on (ur.user_id)
    ur.user_id,
    r.name
  from user_roles ur
  join roles r on r.id = ur.role_id
  order by ur.user_id,
    case r.name when 'owner' then 0 when 'admin' then 1 else 2 end
) r
where r.user_id = u.id;

-- Drop new tables
drop table if exists user_roles;
drop table if exists role_permissions;
drop table if exists roles;
