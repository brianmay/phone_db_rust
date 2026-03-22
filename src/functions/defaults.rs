use crate::models::defaults as models;
use dioxus::prelude::*;
use dioxus_fullstack::{ServerFnError, server};

#[cfg(feature = "server")]
use super::common::{AppError, get_database_connection, get_user_id};

#[server]
pub async fn search_defaults(query: String) -> Result<Vec<models::Default>, ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;
    let mut conn = get_database_connection().await?;

    crate::server::database::service::defaults::search_defaults(&mut conn, query)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

#[server]
pub async fn get_default_by_id(
    id: models::DefaultId,
) -> Result<Option<models::Default>, ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;

    let mut conn = get_database_connection().await?;

    crate::server::database::service::defaults::get_default_by_id(&mut conn, id)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

#[server]
pub async fn create_default(default: models::NewDefault) -> Result<models::Default, ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;

    let mut conn = get_database_connection().await?;

    crate::server::database::service::defaults::create_default(&mut conn, default)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

#[server]
pub async fn update_default(
    old_default: models::Default,
    change_default: models::ChangeDefault,
) -> Result<models::Default, ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;

    let mut conn = get_database_connection().await?;

    crate::server::database::service::defaults::update_default(
        &mut conn,
        old_default,
        change_default,
    )
    .await
    .map_err(AppError::from)
    .map_err(ServerFnError::from)
}

#[server]
pub async fn delete_default(old_default: models::Default) -> Result<(), ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;

    let mut conn = get_database_connection().await?;

    crate::server::database::service::defaults::delete_default(&mut conn, old_default)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}
