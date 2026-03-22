CREATE TABLE old_users (
    id bigint NOT NULL PRIMARY KEY,
    username character varying(255),
    password_hash character varying(255),
    is_admin boolean DEFAULT false,
    is_trusted boolean DEFAULT false,
    is_phone boolean DEFAULT false,
    inserted_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL
);

CREATE TABLE schema_migrations (
    version bigint NOT NULL PRIMARY KEY,
    inserted_at timestamp(0) without time zone
);

CREATE UNIQUE INDEX users_username_index ON old_users (username);
