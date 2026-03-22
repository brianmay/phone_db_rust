use crate::models::contacts as models;
use dioxus::prelude::*;
use dioxus_fullstack::{ServerFnError, server};

#[cfg(feature = "server")]
use super::common::{AppError, get_database_connection, get_user_id};

#[server]
pub async fn search_contacts(query: String) -> Result<Vec<models::Contact>, ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;
    let mut conn = get_database_connection().await?;

    crate::server::database::service::contacts::search_contacts(&mut conn, query)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

/// Returns up to `page_size + 1` contacts matching `query`, sorted
/// `(name ASC NULLS LAST, id ASC)`, starting after the cursor row.
///
/// The cursor is split into two params because `Option<Option<String>>` does
/// not round-trip cleanly as a URL query param:
///   - `before_id`        — id of the last visible row (`None` = first page)
///   - `before_name`      — name of the last visible row, if it had one
///   - `before_name_null` — true when the last visible row's name was NULL
///
/// Caller uses the extra row to detect whether a next page exists.
#[server]
pub async fn search_contacts_paginated(
    query: String,
    before_id: Option<models::ContactId>,
    before_name: Option<String>,
    before_name_null: bool,
    page_size: i64,
) -> Result<Vec<models::Contact>, ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;
    let mut conn = get_database_connection().await?;

    // Reconstruct Option<Option<String>> from the split params.
    let cursor_name: Option<Option<String>> = if before_id.is_some() {
        if before_name_null {
            Some(None)
        } else {
            Some(before_name)
        }
    } else {
        None
    };

    crate::server::database::service::contacts::search_contacts_paginated(
        &mut conn,
        query,
        cursor_name,
        before_id,
        page_size + 1,
    )
    .await
    .map_err(AppError::from)
    .map_err(ServerFnError::from)
}

#[server]
pub async fn get_contact_by_id(
    id: models::ContactId,
) -> Result<Option<models::Contact>, ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;

    let mut conn = get_database_connection().await?;

    crate::server::database::service::contacts::get_contact_by_id(&mut conn, id)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

#[server]
pub async fn create_contact(contact: models::NewContact) -> Result<models::Contact, ServerFnError> {
    use crate::{functions::common::get_ldap_connection, server::database::service::contacts};

    let _logged_in_user_id = get_user_id().await?;

    let mut conn = get_database_connection().await?;
    let (mut ldap, base_dn) = get_ldap_connection().await?;

    contacts::create_contact(&mut conn, &base_dn, &mut ldap, contact)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

#[server]
pub async fn update_contact(
    old_contact: models::Contact,
    change_contact: models::ChangeContact,
) -> Result<models::Contact, ServerFnError> {
    use crate::{functions::common::get_ldap_connection, server::database::service::contacts};

    let _logged_in_user_id = get_user_id().await?;

    let mut conn = get_database_connection().await?;
    let (mut ldap, base_dn) = get_ldap_connection().await?;

    contacts::update_contact(&mut conn, &base_dn, &mut ldap, old_contact, change_contact)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

#[server]
pub async fn delete_contact(old_contact: models::Contact) -> Result<(), ServerFnError> {
    use crate::functions::common::get_ldap_connection;

    let _logged_in_user_id = get_user_id().await?;
    let mut conn = get_database_connection().await?;
    let (mut ldap, base_dn) = get_ldap_connection().await?;

    crate::server::database::service::contacts::delete_contact(
        &mut conn,
        &base_dn,
        &mut ldap,
        old_contact,
    )
    .await
    .map_err(AppError::from)
    .map_err(ServerFnError::from)
}
