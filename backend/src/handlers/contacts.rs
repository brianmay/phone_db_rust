use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_qs::axum::QsQuery;
use sqlx::postgres::PgPool;
use sqlx_conditional_queries::conditional_query_as;
use tap::Pipe;

use crate::errors;

use crate::errors::{Response, Result};
use crate::AppState;

use common::{ContactDetails, ContactKey, ContactUpdateRequest, PageRequest};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_contacts))
        .route("/", post(post_contacts))
        .with_state(state)
}

#[axum::debug_handler]
async fn get_contacts(
    State(db): State<PgPool>,
    QsQuery(request): QsQuery<PageRequest<ContactKey>>,
) -> Result<Vec<ContactDetails>> {
    conditional_query_as!(
        ContactDetails,
        r#"
        SELECT *, (SELECT COUNT(*) FROM phone_calls WHERE contact_id = contacts.id) as number_calls
        FROM contacts
        {#where_clause}
        ORDER BY phone_number, id
        LIMIT 10
        "#,
        #where_clause = match request.after_key {
            Some(ContactKey{phone_number, id}) => "WHERE (phone_number, id) > ({phone_number},{id})",
            None => "",
        }
    )
    .fetch_all(&db)
    .await?
    .pipe(Response::new)
    .pipe(Ok)
}

#[axum::debug_handler]
async fn post_contacts(
    State(db): State<PgPool>,
    Json(request): Json<ContactUpdateRequest>,
) -> Result<()> {
    let time = chrono::Utc::now();
    let ContactUpdateRequest {
        id,
        name,
        action,
        comments,
    } = request;

    let result = sqlx::query!(
        r#"
        UPDATE contacts SET name = $2, action = $3, comments = $4, updated_at = $5
        WHERE id = $1
        "#,
        id,
        name,
        action.as_str(),
        comments,
        time
    )
    .execute(&db)
    .await?;

    if result.rows_affected() == 0 {
        Err(errors::Error::NotFound("Contact".to_string(), request.id))?;
    }

    Ok(Response::new(()))
}
