use std::env;
use std::sync::Arc;

use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Basic},
};
use diesel_async::AsyncConnection;
use thiserror::Error;
use tokio::sync::broadcast;

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::contacts::{Contact, NewContact};
use crate::models::defaults::DefaultList;
use crate::models::phone_calls::{NewPhoneCall, PhoneCall};
use crate::server::database::connection as database;
use crate::server::database::service::contacts;
use crate::server::database::service::defaults;
use crate::server::database::service::phone_calls;
use crate::server::ldap::connect as ldap;
use crate::server::ldap::query::Error as LdapError;

#[derive(Debug, Clone)]
pub struct Authentication {
    pub username: String,
    pub password: String,
}

impl Authentication {
    pub fn get_from_env() -> Authentication {
        let username = env::var("PHONE_USERNAME").expect("PHONE_USERNAME must be set");
        let password = env::var("PHONE_PASSWORD").expect("PHONE_PASSWORD must be set");
        Authentication { username, password }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IncomingPhoneCallRequest {
    pub phone_number: String,
    pub destination_number: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PhoneCallDetails {
    pub id: i64,
    pub action: String,
    pub contact_id: i64,
    pub contact_name: Option<String>,
    pub contact_phone_number: String,
    pub contact_action: String,
    pub contact_comments: Option<String>,
    pub phone_number: String,
    pub destination_number: Option<String>,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub number_calls: Option<i64>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Not authorized")]
    NotAuthorized,
    #[error("Database error: {0}")]
    Database(#[from] database::Error),
    #[error("LDAP error: {0}")]
    Ldap(#[from] LdapError),
    #[error("Contacts error: {0}")]
    Contacts(#[from] contacts::Error),
    #[error("Defaults error: {0}")]
    Defaults(#[from] defaults::Error),
    #[error("Phone calls error: {0}")]
    PhoneCalls(#[from] phone_calls::Error),
    #[error("Diesel error: {0}")]
    Diesel(#[from] diesel::result::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::NotAuthorized => {
                (axum::http::StatusCode::UNAUTHORIZED, self.to_string()).into_response()
            } // _ => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),
            Error::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error",
                )
                    .into_response()
            }
            Error::Ldap(e) => {
                tracing::error!("LDAP error: {:?}", e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error",
                )
                    .into_response()
            }
            Error::Contacts(e) => {
                tracing::error!("Wrappers error: {:?}", e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error",
                )
                    .into_response()
            }
            Error::Defaults(e) => {
                tracing::error!("Defaults error: {:?}", e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error",
                )
                    .into_response()
            }
            Error::PhoneCalls(e) => {
                tracing::error!("Phone calls error: {:?}", e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error",
                )
                    .into_response()
            }
            Error::Diesel(e) => {
                tracing::error!("Diesel error: {:?}", e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error",
                )
                    .into_response()
            }
        }
    }
}

pub async fn post_handler(
    Extension(authentication): Extension<Arc<Authentication>>,
    Extension(db): Extension<database::DatabasePool>,
    Extension(ldap): Extension<ldap::LdapPool>,
    Extension(tx): Extension<broadcast::Sender<(PhoneCall, Contact)>>,
    TypedHeader(Authorization(creds)): TypedHeader<Authorization<Basic>>,
    Json(request): Json<IncomingPhoneCallRequest>,
) -> Result<Json<PhoneCallDetails>, Error> {
    if creds.username() != authentication.username || creds.password() != authentication.password {
        return Err(Error::NotAuthorized);
    }

    let mut conn = db.get().await.map_err(database::Error::from)?;
    let base_dn = ldap.base_dn().to_string();
    let mut ldap_conn = ldap.get().await.map_err(LdapError::from)?;

    let contact = contacts::get_contact_by_phone_number(&mut conn, &request.phone_number).await?;
    let request_clone = request.clone();

    let (phone_call, contact) = conn
        .transaction::<_, Error, _>(|conn| {
            Box::pin(async move {
                let contact = match contact {
                    Some(contact) => contact,
                    None => {
                        let defaults = defaults::get_all_defaults(conn)
                            .await
                            .map(DefaultList::new)?;

                        let default = defaults.search_phone_number(&request.phone_number);

                        let name = default.and_then(|d| d.name.clone());
                        let action = default
                            .map(|d| d.action.clone())
                            .unwrap_or_else(|| "allow".to_string());

                        let request = NewContact {
                            phone_number: request.phone_number.clone(),
                            name,
                            action,
                            comments: None,
                        };

                        contacts::create_contact(conn, &base_dn, &mut ldap_conn, request).await?
                    }
                };

                let new_phone_call = NewPhoneCall {
                    action: contact.action.clone(),
                    contact_id: contact.id,
                    destination_number: Some(request.destination_number.clone()),
                };

                let phone_call = phone_calls::create_phone_call(conn, new_phone_call).await?;

                Ok((phone_call, contact))
            })
        })
        .await?;

    let details = PhoneCallDetails {
        id: phone_call.id.as_inner(),
        action: phone_call.action.clone(),
        contact_id: contact.id.as_inner(),
        contact_name: contact.name.clone(),
        contact_phone_number: contact.phone_number.clone(),
        contact_action: contact.action.clone(),
        contact_comments: contact.comments.clone(),
        phone_number: request_clone.phone_number,
        destination_number: Some(request_clone.destination_number),
        inserted_at: phone_call.inserted_at,
        updated_at: phone_call.updated_at,
        number_calls: Some(contact.phone_call_count),
    };

    _ = tx.send((phone_call, contact));

    Ok(Json(details))
}
