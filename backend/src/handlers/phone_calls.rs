use axum::extract::State;
use axum::routing::get;
use axum::Router;
use serde_qs::axum::QsQuery;
use sqlx::postgres::PgPool;
use tap::Pipe;

use crate::errors::{Response, Result};
use crate::{database, AppState};

use common::{Page, PageRequest, PhoneCallDetails, PhoneCallKey};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_phone_calls))
        .with_state(state)
}

#[axum::debug_handler(state = AppState)]
pub async fn get_phone_calls(
    State(db): State<PgPool>,
    QsQuery(request): QsQuery<PageRequest<PhoneCallKey>>,
) -> Result<Page<PhoneCallDetails, PhoneCallKey>> {
    database::phone_calls::get_phone_calls(&db, &request)
        .await?
        .pipe(Response::new)
        .pipe(Ok)
}
