use axum::Extension;
use diesel_async::pooled_connection::PoolError;
use dioxus::prelude::*;
use dioxus_fullstack::FullstackContext;
use dioxus_fullstack::ServerFnError;
use std::ops::Deref;
use tap::Pipe;
use thiserror::Error;

use crate::models::users::UserId;
use crate::server::auth::Session;
use crate::server::database::connection::DatabaseConnection;
use crate::server::database::connection::DatabasePool;
use crate::server::ldap::connect::LdapConnection;
use crate::server::ldap::query::Error as LdapError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database pool error: {0}")]
    DbPool(#[from] mobc::Error<PoolError>),

    #[error("Database error: {0}")]
    Db(#[from] diesel::result::Error),

    #[error("LDAP error: {0}")]
    Ldap(#[from] LdapError),

    #[error("Contacts error: {0}")]
    Contacts(#[from] crate::server::database::service::contacts::Error),

    #[error("Users error: {0}")]
    Users(#[from] crate::server::database::service::users::Error),
}

impl From<AppError> for ServerFnError {
    fn from(err: AppError) -> Self {
        ServerFnError::new(err.to_string())
    }
}

pub async fn get_database_connection() -> Result<DatabaseConnection, ServerFnError> {
    let Extension(pool): Extension<DatabasePool> = FullstackContext::extract().await?;
    pool.get().await.map_err(AppError::from)?.pipe(Ok)
}

pub async fn get_ldap_connection() -> Result<(LdapConnection, String), ServerFnError> {
    let Extension(pool): Extension<crate::server::ldap::connect::LdapPool> =
        FullstackContext::extract().await?;

    let connection = pool
        .get()
        .await
        .map_err(LdapError::from)
        .map_err(AppError::from)?
        .deref()
        .clone();

    Ok((connection, pool.base_dn().to_string()))
}

pub async fn get_user_id() -> Result<UserId, ServerFnError> {
    let session: Session = FullstackContext::extract().await?;
    session
        .user
        .as_ref()
        .map(|x| UserId::new(x.id))
        .ok_or(ServerFnError::new("Not Logged In".to_string()))
}

pub async fn assert_is_admin() -> Result<(), ServerFnError> {
    let session: Session = FullstackContext::extract().await?;
    let user = session
        .user
        .as_ref()
        .ok_or(ServerFnError::new("Not Logged In".to_string()))?;
    user.is_admin
        .then_some(())
        .ok_or(ServerFnError::new("Not Admin".to_string()))?;
    Ok(())
}
