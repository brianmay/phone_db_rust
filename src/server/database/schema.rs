// @generated automatically by Diesel CLI.

diesel::table! {
    contacts (id) {
        id -> Int8,
        #[max_length = 255]
        phone_number -> Varchar,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        #[max_length = 255]
        action -> Varchar,
        inserted_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 255]
        comments -> Nullable<Varchar>,
    }
}

diesel::table! {
    defaults (id) {
        id -> Int8,
        order -> Nullable<Int4>,
        #[max_length = 255]
        regexp -> Nullable<Varchar>,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        #[max_length = 255]
        action -> Varchar,
        inserted_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    groups (id) {
        id -> Int8,
        name -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    old_users (id) {
        id -> Int8,
        #[max_length = 255]
        username -> Nullable<Varchar>,
        #[max_length = 255]
        password_hash -> Nullable<Varchar>,
        is_admin -> Nullable<Bool>,
        is_trusted -> Nullable<Bool>,
        is_phone -> Nullable<Bool>,
        inserted_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    phone_calls (id) {
        id -> Int8,
        #[max_length = 255]
        action -> Varchar,
        contact_id -> Int8,
        inserted_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 255]
        destination_number -> Nullable<Varchar>,
        #[max_length = 255]
        source_number -> Varchar,
    }
}

diesel::table! {
    schema_migrations (version) {
        version -> Int8,
        inserted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    session (id) {
        id -> Text,
        data -> Jsonb,
        expiry_date -> Timestamptz,
    }
}

diesel::table! {
    user_groups (user_id, group_id) {
        user_id -> Int8,
        group_id -> Int8,
    }
}

diesel::table! {
    users (id) {
        id -> Int8,
        username -> Text,
        password -> Text,
        full_name -> Text,
        oidc_id -> Nullable<Text>,
        email -> Text,
        is_admin -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(phone_calls -> contacts (contact_id));
diesel::joinable!(user_groups -> groups (group_id));
diesel::joinable!(user_groups -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    contacts,
    defaults,
    groups,
    old_users,
    phone_calls,
    schema_migrations,
    session,
    user_groups,
    users,
);
