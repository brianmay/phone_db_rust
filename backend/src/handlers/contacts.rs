use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use sqlx::postgres::PgPool;
use tap::Pipe;

use crate::errors;

use crate::errors::{Response, Result};
use crate::AppState;

use common::{ContactDetails, ContactUpdateRequest, Page, PageRequest};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_contacts))
        .route("/", post(post_contacts))
        .with_state(state)
}

#[axum::debug_handler]
async fn get_contacts(
    State(db): State<PgPool>,
    Query(page): Query<PageRequest>,
) -> Result<Page<ContactDetails>> {
    let offset: i64 = (page.per_page * page.page) as i64;
    let limit = page.per_page as i64;

    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM contacts
        "#
    )
    .fetch_one(&db)
    .await?
    .unwrap_or_default()
    .pipe(u32::try_from)?;

    let rows = sqlx::query_as!(
        ContactDetails,
        r#"
        SELECT *, (SELECT COUNT(*) FROM phone_calls WHERE contact_id = contacts.id) as number_calls
        FROM contacts
        OFFSET $1 LIMIT $2
        "#,
        offset,
        limit,
    )
    .fetch_all(&db)
    .await?;

    Page::new(
        rows,
        count.div_ceil(page.per_page),
        page.page,
        page.per_page,
    )
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
