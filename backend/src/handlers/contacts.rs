use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_qs::axum::QsQuery;
use sqlx::postgres::PgPool;
use tap::Pipe;

use crate::{database, errors};

use crate::errors::{Response, Result};
use crate::AppState;

use common::{ContactDetails, ContactKey, ContactUpdateRequest, Page, PageRequest};

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
) -> Result<Page<ContactDetails, ContactKey>> {
    database::contacts::get_contacts(&db, &request)
        .await?
        .pipe(Response::new)
        .pipe(Ok)
}

#[axum::debug_handler]
async fn post_contacts(
    State(db): State<PgPool>,
    Json(request): Json<ContactUpdateRequest>,
) -> Result<ContactDetails> {
    let id = request.id;
    database::contacts::update_contact(&db, request)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => errors::Error::ObjectNotFound("Contact".to_string(), id),
            _ => errors::Error::Sqlx(err),
        })?
        .pipe(Response::new)
        .pipe(Ok)
}
