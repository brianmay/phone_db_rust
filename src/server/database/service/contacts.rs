use diesel_async::AsyncConnection;
use thiserror::Error;

use crate::models::contacts as models;
use crate::server::database::connection as database;
use crate::server::database::models::contacts;
use crate::server::ldap::connect as ldap;
use crate::server::ldap::query::Error as LdapError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] database::Error),
    #[error("Diesel error: {0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("LDAP error: {0}")]
    Ldap(#[from] LdapError),
}

pub async fn search_contacts(
    conn: &mut database::DatabaseConnection,
    query: String,
) -> Result<Vec<models::Contact>, Error> {
    contacts::search_contacts(conn, &query)
        .await
        .map(|rows| rows.into_iter().map(|(c, n)| c.into_model(n)).collect())
        .map_err(database::Error::from)
        .map_err(Error::from)
}

pub async fn get_contact_by_id(
    conn: &mut database::DatabaseConnection,
    id: models::ContactId,
) -> Result<Option<models::Contact>, Error> {
    contacts::get_contact_by_id(conn, id.as_inner())
        .await
        .map(|x| x.map(|(c, n)| c.into_model(n)))
        .map_err(database::Error::from)
        .map_err(Error::from)
}

pub async fn get_contact_by_phone_number(
    conn: &mut database::DatabaseConnection,
    phone_number: &str,
) -> Result<Option<models::Contact>, Error> {
    contacts::get_contact_by_phone_number(conn, phone_number)
        .await
        .map(|x| x.map(|(c, n)| c.into_model(n)))
        .map_err(database::Error::from)
        .map_err(Error::from)
}

pub async fn create_contact(
    conn: &mut database::DatabaseConnection,
    base_dn: &str,
    ldap: &mut ldap::LdapConnection,
    contact: models::NewContact,
) -> Result<models::Contact, Error> {
    let new_contact = contacts::NewContact::from_front_end(&contact);
    let base_dn = base_dn.to_string();
    let ldap = ldap.clone();

    conn.transaction::<_, Error, _>(move |conn| {
        let base_dn = base_dn.clone();
        let mut ldap = ldap.clone();
        let new_contact = new_contact.clone();
        Box::pin(async move {
            let contact = contacts::create_contact(conn, new_contact)
                .await
                .map_err(Error::from)?;

            use crate::server::ldap::query::update_ldap_contact_from_contact;

            let model_contact = contact.into_model(0);

            update_ldap_contact_from_contact(
                &model_contact.phone_number,
                &model_contact,
                &base_dn,
                &mut ldap,
            )
            .await
            .map_err(Error::from)?;

            Ok(model_contact)
        })
    })
    .await
}

pub async fn update_contact(
    conn: &mut database::DatabaseConnection,
    base_dn: &str,
    ldap: &mut ldap::LdapConnection,
    old_contact: models::Contact,
    change_contact: models::ChangeContact,
) -> Result<models::Contact, Error> {
    let updates = contacts::ChangeContact::from_front_end(&change_contact);
    let old_contact_id = old_contact.id.as_inner();
    let old_phone_number = old_contact.phone_number.clone();
    let base_dn = base_dn.to_string();
    let ldap = ldap.clone();

    conn.transaction::<_, Error, _>(move |conn| {
        let base_dn = base_dn.clone();
        let mut ldap = ldap.clone();
        let updates = updates.clone();
        let old_phone_number = old_phone_number.clone();
        Box::pin(async move {
            let contact = contacts::update_contact(conn, old_contact_id, updates)
                .await
                .map_err(Error::from)?;

            let count = contacts::get_phone_call_count(conn, contact.id)
                .await
                .map_err(Error::from)?;

            use crate::server::ldap::query::update_ldap_contact_from_contact;

            let model_contact = contact.into_model(count);

            update_ldap_contact_from_contact(
                &old_phone_number,
                &model_contact,
                &base_dn,
                &mut ldap,
            )
            .await
            .map_err(Error::from)?;

            Ok(model_contact)
        })
    })
    .await
}

pub async fn delete_contact(
    conn: &mut database::DatabaseConnection,
    base_dn: &str,
    ldap: &mut ldap::LdapConnection,
    old_contact: models::Contact,
) -> Result<(), Error> {
    let old_contact_id = old_contact.id.as_inner();
    let old_phone_number = old_contact.phone_number.clone();
    let base_dn = base_dn.to_string();
    let ldap = ldap.clone();

    conn.transaction::<_, Error, _>(move |conn| {
        let base_dn = base_dn.clone();
        let mut ldap = ldap.clone();
        let old_phone_number = old_phone_number.clone();
        Box::pin(async move {
            crate::server::database::models::contacts::delete_contact(conn, old_contact_id)
                .await
                .map_err(Error::from)?;

            use crate::server::ldap::query::delete_ldap_contact_from_phone_number;

            delete_ldap_contact_from_phone_number(&old_phone_number, &base_dn, &mut ldap)
                .await
                .map_err(Error::from)?;

            Ok(())
        })
    })
    .await
}
