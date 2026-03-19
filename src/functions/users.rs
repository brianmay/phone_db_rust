use dioxus::prelude::*;
use dioxus_fullstack::{ServerFnError, server};

use crate::models::users as models;

#[cfg(feature = "server")]
use super::common::{AppError, assert_is_admin, get_database_connection};

#[server]
pub async fn get_users() -> Result<Vec<models::User>, ServerFnError> {
    assert_is_admin().await?;
    let mut conn = get_database_connection().await?;

    crate::server::database::service::users::get_users(&mut conn)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

#[server]
pub async fn get_user_by_id(id: models::UserId) -> Result<Option<models::User>, ServerFnError> {
    assert_is_admin().await?;
    let mut conn = get_database_connection().await?;

    crate::server::database::service::users::get_user_by_id(&mut conn, id)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

#[server]
pub async fn create_user(user: models::NewUser) -> Result<models::User, ServerFnError> {
    assert_is_admin().await?;
    let mut conn = get_database_connection().await?;

    let hashed_password = password_auth::generate_hash(&user.password);

    crate::server::database::service::users::create_user(&mut conn, user, &hashed_password)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

#[server]
pub async fn update_user(
    old_user: models::User,
    change_user: models::ChangeUser,
    password: Option<String>,
) -> Result<models::User, ServerFnError> {
    use crate::server::database::service::users;

    assert_is_admin().await?;
    let mut conn = get_database_connection().await?;

    let hashed_password = password.as_ref().map(password_auth::generate_hash);

    users::update_user(&mut conn, old_user, change_user, hashed_password)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

#[server]
pub async fn delete_user(id: models::User) -> Result<(), ServerFnError> {
    use crate::server::database::service::users;

    assert_is_admin().await?;
    let mut conn = get_database_connection().await?;

    users::delete_user(&mut conn, id)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}
