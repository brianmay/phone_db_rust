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
