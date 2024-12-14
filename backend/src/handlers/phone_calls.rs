use axum::extract::State;
use axum::routing::get;
use axum::Router;
use sqlx::postgres::PgPool;
use tap::Pipe;

use crate::errors::{Response, Result};
use crate::AppState;

use common::PhoneCall;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_phone_calls))
        .with_state(state)
}

#[axum::debug_handler]
async fn get_phone_calls(State(db): State<PgPool>) -> Result<Vec<PhoneCall>> {
    sqlx::query_as!(
        PhoneCall,
        r#"
        SELECT phone_calls.*, contacts.name as contact_name, contacts.action as contact_action, contacts.comments as contact_comments
        FROM phone_calls
        INNER JOIN contacts ON contacts.id = phone_calls.contact_id
        "#
    )
    .fetch_all(&db)
    .await?
    .pipe(Response::new)
    .pipe(Ok)
}
