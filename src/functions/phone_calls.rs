use crate::models::contacts::Contact;
use crate::models::phone_calls as models;
use dioxus::prelude::*;
use dioxus_fullstack::{ServerFnError, server};

#[cfg(feature = "server")]
use super::common::get_user_id;
#[cfg(feature = "server")]
use super::common::{AppError, get_database_connection};

#[server]
pub async fn search_phone_calls(
    query: String,
) -> Result<Vec<(models::PhoneCall, Contact)>, ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;
    let mut conn = get_database_connection().await?;

    crate::server::database::service::phone_calls::search_phone_calls(&mut conn, query)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

#[server]
pub async fn get_phone_call_by_id(
    id: models::PhoneCallId,
) -> Result<Option<models::PhoneCall>, ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;

    let mut conn = get_database_connection().await?;

    crate::server::database::service::phone_calls::get_phone_call_by_id(&mut conn, id)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

#[server]
pub async fn create_phone_call(
    phone_call: models::NewPhoneCall,
) -> Result<models::PhoneCall, ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;

    let mut conn = get_database_connection().await?;

    crate::server::database::service::phone_calls::create_phone_call(&mut conn, phone_call)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

#[server]
pub async fn update_phone_call(
    old_phone_call: models::PhoneCall,
    change_phone_call: models::ChangePhoneCall,
) -> Result<models::PhoneCall, ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;

    let conn = get_database_connection().await?;

    crate::server::database::service::phone_calls::update_phone_call(
        conn,
        old_phone_call,
        change_phone_call,
    )
    .await
    .map_err(AppError::from)
    .map_err(ServerFnError::from)
}

#[server]
pub async fn delete_phone_call(old_phone_call: models::PhoneCall) -> Result<(), ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;

    let conn = get_database_connection().await?;

    crate::server::database::service::phone_calls::delete_phone_call(conn, old_phone_call)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}
