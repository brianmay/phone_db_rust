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

use crate::errors::Error;
use crate::Authentication;

use super::contacts::Contact;
use super::errors::{Response, Result};
use super::ldap;
use super::AppState;
use super::Ldap;

use common::{Action, IncomingPhoneCallRequest, IncomingPhoneCallResponse};

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
    TypedHeader(Authorization(creds)): TypedHeader<Authorization<Basic>>,
    Json(request): Json<IncomingPhoneCallRequest>,
) -> Result<IncomingPhoneCallResponse> {
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
            sqlx::query_as!(
                Contact,
                r#"
                INSERT INTO contacts (phone_number, action, inserted_at, updated_at)
                VALUES ($1,$2,$3,$3)
                RETURNING *
                "#,
                request.phone_number,
                Action::Allow.as_str(),
                now
            )
            .fetch_one(&db)
            .await?
        }
    };

    sqlx::query!(
        r#"
        INSERT INTO phone_calls (action, contact_id, destination_number, inserted_at, updated_at)
        VALUES ($1, $2, $3, $4, $4)
        RETURNING id
        "#,
        contact.action.as_str(),
        contact.id,
        request.destination_number,
        now
    )
    .fetch_one(&db)
    .await?;

    ldap::update_contact(&contact, &ldap).await?;

    IncomingPhoneCallResponse {
        name: contact.name,
        action: contact.action,
    }
    .pipe(Response::new)
    .pipe(Ok)
}
