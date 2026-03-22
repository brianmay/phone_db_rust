use diesel_async::AsyncConnection;
use thiserror::Error;

use crate::models::contacts::Contact;
use crate::models::phone_calls as models;
use crate::server::database::connection as database;
use crate::server::database::models::phone_calls;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(#[from] database::Error),
    #[error("Diesel error: {0}")]
    DieselError(#[from] diesel::result::Error),
}

pub async fn search_phone_calls(
    conn: &mut database::DatabaseConnection,
    query: String,
) -> Result<Vec<(models::PhoneCall, Contact)>, Error> {
    phone_calls::search_phone_calls(conn, &query)
        .await
        .map(|x| {
            x.into_iter()
                .map(|(phone_call, contact)| (phone_call.into(), contact.into()))
                .collect::<Vec<(models::PhoneCall, Contact)>>()
        })
        .map_err(database::Error::from)
        .map_err(Error::from)
}

pub async fn get_phone_call_by_id(
    conn: &mut database::DatabaseConnection,
    id: models::PhoneCallId,
) -> Result<Option<models::PhoneCall>, Error> {
    phone_calls::get_phone_call_by_id(conn, id.as_inner())
        .await
        .map(|x| x.map(|y| y.into()))
        .map_err(database::Error::from)
        .map_err(Error::from)
}

pub async fn create_phone_call(
    conn: &mut database::DatabaseConnection,
    phone_call: models::NewPhoneCall,
) -> Result<models::PhoneCall, Error> {
    let new_phone_call = phone_calls::NewPhoneCall::from_front_end(&phone_call);

    conn.transaction::<_, Error, _>(|conn| {
        Box::pin(async move {
            let phone_call: models::PhoneCall =
                phone_calls::create_phone_call(conn, &new_phone_call)
                    .await
                    .map(|x| x.into())
                    .map_err(Error::from)?;

            Ok(phone_call)
        })
    })
    .await
}

pub async fn update_phone_call(
    mut conn: database::DatabaseConnection,
    old_phone_call: models::PhoneCall,
    change_phone_call: models::ChangePhoneCall,
) -> Result<models::PhoneCall, Error> {
    let updates = phone_calls::ChangePhoneCall::from_front_end(&change_phone_call);

    conn.transaction::<_, Error, _>(|conn| {
        Box::pin(async move {
            let phone_call: models::PhoneCall =
                phone_calls::update_phone_call(conn, old_phone_call.id.as_inner(), &updates)
                    .await
                    .map(|x| x.into())
                    .map_err(Error::from)?;

            Ok(phone_call)
        })
    })
    .await
}

pub async fn delete_phone_call(
    mut conn: database::DatabaseConnection,
    old_phone_call: models::PhoneCall,
) -> Result<(), Error> {
    conn.transaction::<_, Error, _>(|conn| {
        Box::pin(async move {
            crate::server::database::models::phone_calls::delete_phone_call(
                conn,
                old_phone_call.id.as_inner(),
            )
            .await
            .map_err(Error::from)?;

            Ok(())
        })
    })
    .await
}
