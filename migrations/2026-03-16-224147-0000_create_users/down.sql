drop table "session";
drop table user_groups;
drop table users;
drop table groups;
alter table old_users
    rename to users;