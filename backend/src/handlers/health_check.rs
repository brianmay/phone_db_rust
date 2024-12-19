use axum::extract::State;
use sqlx::PgPool;

use crate::errors::{Response, Result};
use crate::ldap::get_ldap_contact;
use crate::{AppState, Ldap};

#[axum::debug_handler(state = AppState)]
pub async fn health_check_handler(
    State(db): State<PgPool>,
    State(ldap): State<Ldap>,
) -> Result<()> {
    sqlx::query!("SELECT 1 as result").fetch_one(&db).await?;
    let response = Response::new(());

    let contact = get_ldap_contact("000", &ldap).await?;
    println!("{contact:#?}");

    Ok(response)
}
