-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS "users";
DROP TABLE IF EXISTS "defaults";
DROP TABLE IF EXISTS "phone_calls";
DROP TABLE IF EXISTS "contacts";
DROP TABLE IF EXISTS "schema_migrations";
DROP INDEX IF EXISTS users_username_index;
DROP INDEX IF EXISTS contacts_phone_number_index;