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

use crate::errors::Error;
use crate::errors::{Response, Result};
use crate::types::Contact;
use crate::AppState;
use crate::Authentication;
use crate::Ldap;
use crate::{database, ldap};

use common::{Action, IncomingPhoneCallRequest, PhoneCallDetails};

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

    let contact = sqlx::query_as!(
        Contact,
        r#"
        SELECT *
        FROM contacts
        WHERE phone_number = $1
        "#,
        request.phone_number
    )
    .fetch_optional(&db)
    .await?;

    let contact = match contact {
        Some(contact) => contact,
        None => {
            let defaults = database::defaults::get_defaults(&db).await?;

            let default = defaults.search_phone_number(&request.phone_number);

            let name = default.map(|d| d.name.clone());
            let action = default.map(|d| d.action).unwrap_or(Action::Allow);

            sqlx::query_as!(
                Contact,
                r#"
                INSERT INTO contacts (phone_number, name, action, inserted_at, updated_at)
                VALUES ($1,$2,$3,$4,$4)
                RETURNING *
                "#,
                request.phone_number,
                name,
                action.as_str(),
                now
            )
            .fetch_one(&db)
            .await?
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

    ldap::update_contact(&contact, &ldap).await?;

    phone_call.pipe(Response::new).pipe(Ok)
}
