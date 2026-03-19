-- Your SQL goes here
CREATE TABLE "users"(
	"id" INT8 NOT NULL PRIMARY KEY,
	"username" VARCHAR(255),
	"password_hash" VARCHAR(255),
	"is_admin" BOOL,
	"is_trusted" BOOL,
	"is_phone" BOOL,
	"inserted_at" TIMESTAMPTZ NOT NULL,
	"updated_at" TIMESTAMPTZ NOT NULL
);
CREATE TABLE "defaults"(
	"id" INT8 NOT NULL PRIMARY KEY,
	"order" INT4,
	"regexp" VARCHAR(255),
	"name" VARCHAR(255),
	"action" VARCHAR(255),
	"inserted_at" TIMESTAMPTZ NOT NULL,
	"updated_at" TIMESTAMPTZ NOT NULL
);
CREATE TABLE "contacts"(
	"id" INT8 NOT NULL PRIMARY KEY,
	"phone_number" VARCHAR(255) NOT NULL,
	"name" VARCHAR(255),
	"action" VARCHAR(255) NOT NULL,
	"inserted_at" TIMESTAMPTZ NOT NULL,
	"updated_at" TIMESTAMPTZ NOT NULL,
	"comments" VARCHAR(255)
);
CREATE TABLE "phone_calls"(
	"id" INT8 NOT NULL PRIMARY KEY,
	"action" VARCHAR(255) NOT NULL,
	"contact_id" INT8 NOT NULL,
	"inserted_at" TIMESTAMPTZ NOT NULL,
	"updated_at" TIMESTAMPTZ NOT NULL,
	"destination_number" VARCHAR(255),
	FOREIGN KEY ("contact_id") REFERENCES "contacts"("id")
);
CREATE TABLE "schema_migrations"(
	"version" INT8 NOT NULL PRIMARY KEY,
	"inserted_at" TIMESTAMP
);