use crate::models::contacts as contact_models;
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
) -> Result<Vec<(models::PhoneCall, contact_models::Contact)>, ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;
    let mut conn = get_database_connection().await?;

    crate::server::database::service::phone_calls::search_phone_calls(&mut conn, query)
        .await
        .map_err(AppError::from)
        .map_err(ServerFnError::from)
}

/// Returns up to `page_size + 1` results matching `query`, starting after the
/// cursor `(before_ts, before_id)` if provided.  The caller uses the extra
/// entry to detect whether a next page exists.
#[server]
pub async fn search_phone_calls_paginated(
    query: String,
    before_ts: Option<chrono::DateTime<chrono::Utc>>,
    before_id: Option<models::PhoneCallId>,
    page_size: i64,
) -> Result<Vec<(models::PhoneCall, contact_models::Contact)>, ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;
    let mut conn = get_database_connection().await?;

    let before = match (before_ts, before_id) {
        (Some(ts), Some(id)) => Some((ts, id)),
        _ => None,
    };

    crate::server::database::service::phone_calls::search_phone_calls_paginated(
        &mut conn,
        query,
        before,
        page_size + 1,
    )
    .await
    .map_err(AppError::from)
    .map_err(ServerFnError::from)
}

/// Returns up to `page_size + 1` calls for a contact, starting after the
/// cursor `(before_ts, before_id)` if provided.  The caller uses the extra
/// entry to detect whether a next page exists (fetch `page_size + 1`, render
/// only `page_size`, next-page cursor = last rendered row).
#[server]
pub async fn get_phone_calls_for_contact(
    contact_id: contact_models::ContactId,
    before_ts: Option<chrono::DateTime<chrono::Utc>>,
    before_id: Option<models::PhoneCallId>,
    page_size: i64,
) -> Result<Vec<models::PhoneCall>, ServerFnError> {
    let _logged_in_user_id = get_user_id().await?;
    let mut conn = get_database_connection().await?;

    let before = match (before_ts, before_id) {
        (Some(ts), Some(id)) => Some((ts, id)),
        _ => None,
    };

    crate::server::database::service::phone_calls::get_phone_calls_for_contact(
        &mut conn,
        contact_id,
        before,
        page_size + 1,
    )
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
