-- Your SQL goes here
CREATE TABLE "users"(
	"id" BIGSERIAL PRIMARY KEY,
	"username" VARCHAR(255),
	"password_hash" VARCHAR(255),
	"is_admin" BOOL DEFAULT false,
	"is_trusted" BOOL DEFAULT false,
	"is_phone" BOOL DEFAULT false,
	"inserted_at" TIMESTAMPTZ NOT NULL,
	"updated_at" TIMESTAMPTZ NOT NULL
);
CREATE TABLE "defaults"(
	"id" BIGSERIAL PRIMARY KEY,
	"order" INT4,
	"regexp" VARCHAR(255),
	"name" VARCHAR(255),
	"action" VARCHAR(255),
	"inserted_at" TIMESTAMPTZ NOT NULL,
	"updated_at" TIMESTAMPTZ NOT NULL
);
CREATE TABLE "contacts"(
	"id" BIGSERIAL PRIMARY KEY,
	"phone_number" VARCHAR(255) NOT NULL,
	"name" VARCHAR(255),
	"action" VARCHAR(255) NOT NULL,
	"inserted_at" TIMESTAMPTZ NOT NULL,
	"updated_at" TIMESTAMPTZ NOT NULL,
	"comments" VARCHAR(255)
);
CREATE TABLE "phone_calls"(
	"id" BIGSERIAL PRIMARY KEY,
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
CREATE UNIQUE INDEX users_username_index ON users USING btree (username);
CREATE UNIQUE INDEX contacts_phone_number_index ON contacts USING btree (phone_number);
