use axum::extract::State;
use axum::routing::get;
use axum::Router;
use serde_qs::axum::QsQuery;
use sqlx::postgres::PgPool;
use sqlx_conditional_queries::conditional_query_as;
use tap::Pipe;

use crate::errors::{Response, Result};
use crate::AppState;

use common::{PageRequest, PhoneCall, PhoneCallKey};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_phone_calls))
        .with_state(state)
}

#[axum::debug_handler(state = AppState)]
async fn get_phone_calls(
    State(db): State<PgPool>,
    QsQuery(request): QsQuery<PageRequest<PhoneCallKey>>,
) -> Result<Vec<PhoneCall>> {
    conditional_query_as!(
        PhoneCall,
        r#"
        SELECT phone_calls.*, contacts.name as contact_name, contacts.phone_number as contact_phone_number, contacts.action as contact_action, contacts.comments as contact_comments
        FROM phone_calls
        INNER JOIN contacts ON contacts.id = phone_calls.contact_id
        {#where_clause}
        ORDER BY inserted_at, id DESC
        LIMIT 10
        "#,
        #where_clause = match request.after_key {
            Some(PhoneCallKey{inserted_at, id}) => "WHERE (phone_calls.inserted_at, phone_calls.id) > ({inserted_at},{id})",
            None => "",
        }
    )
    .fetch_all(&db)
    .await?
    .pipe(Response::new)
    .pipe(Ok)
}
