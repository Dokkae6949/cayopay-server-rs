drop trigger if exists actors_audit_timestamps on actors;
drop trigger if exists users_audit_timestamps on users;
drop trigger if exists guests_audit_timestamps on guests;

drop table if exists guests;
drop table if exists users;
drop table if exists actors;
