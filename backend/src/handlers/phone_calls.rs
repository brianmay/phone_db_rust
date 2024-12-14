use axum::extract::{Query, State};
use axum::routing::get;
use axum::Router;
use sqlx::postgres::PgPool;
use tap::Pipe;

use crate::errors::{Response, Result};
use crate::AppState;

use common::{Page, PageRequest, PhoneCall};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_phone_calls))
        .with_state(state)
}

#[axum::debug_handler]
async fn get_phone_calls(
    State(db): State<PgPool>,
    Query(page): Query<PageRequest>,
) -> Result<Page<PhoneCall>> {
    let offset = (page.per_page * page.page) as i64;
    let limit = page.per_page as i64;

    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM phone_calls
        "#
    )
    .fetch_one(&db)
    .await?
    .unwrap_or_default()
    .pipe(u32::try_from)?;

    let rows = sqlx::query_as!(
        PhoneCall,
        r#"
        SELECT phone_calls.*, contacts.name as contact_name, contacts.action as contact_action, contacts.comments as contact_comments
        FROM phone_calls
        INNER JOIN contacts ON contacts.id = phone_calls.contact_id
        OFFSET $1 LIMIT $2
        "#,
        offset,
        limit
    ).fetch_all(&db).await?;

    Page::new(
        rows,
        count.div_ceil(page.per_page),
        page.page,
        page.per_page,
    )
    .pipe(Response::new)
    .pipe(Ok)
}
