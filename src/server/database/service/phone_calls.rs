use diesel_async::AsyncConnection;
use thiserror::Error;

use crate::models::contacts as contact_models;
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
) -> Result<Vec<(models::PhoneCall, contact_models::Contact)>, Error> {
    phone_calls::search_phone_calls(conn, &query)
        .await
        .map(|x| {
            x.into_iter()
                .map(|(phone_call, contact, count)| (phone_call.into(), contact.into_model(count)))
                .collect()
        })
        .map_err(database::Error::from)
        .map_err(Error::from)
}

pub async fn get_phone_calls_for_contact(
    conn: &mut database::DatabaseConnection,
    contact_id: contact_models::ContactId,
    before: Option<(chrono::DateTime<chrono::Utc>, models::PhoneCallId)>,
    page_size: i64,
) -> Result<Vec<models::PhoneCall>, Error> {
    let before_raw = before.map(|(ts, id)| (ts, id.as_inner()));
    phone_calls::get_phone_calls_for_contact(conn, contact_id.as_inner(), before_raw, page_size)
        .await
        .map(|rows| rows.into_iter().map(|p| p.into()).collect())
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

    conn.transaction::<_, Error, _>(move |conn| {
        let new_phone_call = new_phone_call.clone();
        Box::pin(async move {
            let phone_call: models::PhoneCall =
                phone_calls::create_phone_call(conn, new_phone_call)
                    .await
                    .map(|x| x.into())
                    .map_err(Error::from)?;

            Ok(phone_call)
        })
    })
    .await
}
