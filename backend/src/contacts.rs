use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use tap::Pipe;

use crate::errors;

use super::errors::{Response, Result};
use super::AppState;

use common::{Action, ContactDetails, ContactUpdateRequest};

#[derive(Debug, Deserialize, Serialize)]
pub struct Contact {
    pub id: i64,
    pub phone_number: String,
    pub name: Option<String>,
    pub action: Action,
    pub comments: Option<String>,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Contact {
    pub fn get_update_request(
        self,
        action: Action,
        name: Option<String>,
        comments: Option<String>,
    ) -> ContactUpdateRequest {
        ContactUpdateRequest {
            id: self.id,
            name,
            action,
            comments,
        }
    }
}
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_contacts))
        .route("/", post(post_contacts))
        .with_state(state)
}

#[axum::debug_handler]
async fn get_contacts(State(db): State<PgPool>) -> Result<Vec<ContactDetails>> {
    sqlx::query_as!(
        ContactDetails,
        r#"
        SELECT *, (SELECT COUNT(*) FROM phone_calls WHERE contact_id = contacts.id) as number_calls
        FROM contacts
        "#
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
