use axum::extract::State;
use common::Action;
use sqlx::PgPool;

use crate::errors::{Response, Result};
use crate::ldap::get_contact;
use crate::types::Contact;
use crate::{AppState, Ldap};

#[axum::debug_handler(state = AppState)]
pub async fn health_check_handler(
    State(db): State<PgPool>,
    State(ldap): State<Ldap>,
) -> Result<()> {
    sqlx::query!("SELECT 1 as result").fetch_one(&db).await?;
    let response = Response::new(());
    let now = chrono::Utc::now();

    let contact = Contact {
        id: 23,
        name: Some("Me".to_string()),
        comments: Some("I don't know this person".to_string()),
        phone_number: "1".to_string(),
        action: Action::Allow,
        inserted_at: now,
        updated_at: now,
    };

    let contact = get_contact(&contact, &ldap).await?;
    println!("{contact:#?}");

    Ok(response)
}
