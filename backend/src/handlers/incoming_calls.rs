use std::sync::Arc;

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use axum_extra::{
    headers::{authorization::Basic, Authorization},
    TypedHeader,
};
use sqlx::postgres::PgPool;
use tap::Pipe;
use tokio::sync::broadcast;

use crate::database;
use crate::database::contacts::get_contact_by_phone_number;
use crate::errors::Error;
use crate::errors::{Response, Result};
use crate::ldap::update_ldap_contact_from_contact;
use crate::AppState;
use crate::Authentication;
use crate::Ldap;

use common::{Action, ContactAddRequest, IncomingPhoneCallRequest, PhoneCallDetails};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/incoming_call/", post(post_handler))
        .with_state(state)
}

#[axum::debug_handler(state = AppState)]
async fn post_handler(
    State(authentication): State<Arc<Authentication>>,
    State(db): State<PgPool>,
    State(ldap): State<Ldap>,
    State(tx): State<broadcast::Sender<PhoneCallDetails>>,
    TypedHeader(Authorization(creds)): TypedHeader<Authorization<Basic>>,
    Json(request): Json<IncomingPhoneCallRequest>,
) -> Result<PhoneCallDetails> {
    let now = chrono::Utc::now();

    if creds.username() != authentication.username || creds.password() != authentication.password {
        return Err(Error::NotAuthorized);
    }

    let contact = get_contact_by_phone_number(&db, &request.phone_number).await?;

    let contact = match contact {
        Some(contact) => contact,
        None => {
            let defaults = database::defaults::get_defaults(&db).await?;

            let default = defaults.search_phone_number(&request.phone_number);

            let name = default.map(|d| d.name.clone());
            let action = default.map(|d| d.action).unwrap_or(Action::Allow);

            let request = ContactAddRequest {
                phone_number: request.phone_number.clone(),
                name,
                action,
                comments: None,
            };

            database::contacts::add_contact(&db, request).await?
        }
    };

    let phone_call_id = sqlx::query_scalar!(
        r#"
        INSERT INTO phone_calls (action, contact_id, phone_number ,destination_number, inserted_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $5)
        RETURNING id
        "#,
        contact.action.as_str(),
        contact.id,
        request.phone_number,
        request.destination_number,
        now
    )
    .fetch_one(&db)
    .await?;

    let phone_call = database::phone_calls::get_phone_call(&db, phone_call_id).await?;

    // This will only fail if no one is listening but we don't care.
    _ = tx.send(phone_call.clone());

    update_ldap_contact_from_contact(&contact.phone_number, &contact, &ldap).await?;

    phone_call.pipe(Response::new).pipe(Ok)
}
