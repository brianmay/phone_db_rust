//! Error handling
use axum::http::StatusCode;
use axum::{
    response::{IntoResponse, Response as AxumResponse},
    Json,
};
use serde::Serialize;
use std::result::Result as RustResult;
use tap::Pipe;
use thiserror::Error;
use tracing::error;

use common::Response as MyResponse;

/// An error response
#[derive(Error, Debug)]
pub enum Error {
    #[error("SQLX Error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("{0} Not found: {1}")]
    NotFound(String, i64),

    #[error("OIDC Error: {0}")]
    Oidc(#[from] super::oidc::Error),

    #[error("OIDC not initialized")]
    OIDCNotInitialized,

    #[error("Not authorized")]
    NotAuthorized,

    #[error("Session error: {0}")]
    Session(#[from] tower_sessions_core::session::Error),

    #[error("LDAP error: {0}")]
    Ldap(#[from] simple_ldap::Error),

    #[error("LDAP3 error: {0}")]
    Ldap3(#[from] simple_ldap::ldap3::LdapError),

    #[error("Too many LDAP results")]
    LdapTooManyResults,

    #[error("Integer error: {0}")]
    Int(#[from] std::num::TryFromIntError),
}

impl IntoResponse for Error {
    fn into_response(self) -> AxumResponse {
        match &self {
            error @ Self::Sqlx(..) => {
                error!("{error}");
                let response = MyResponse::<()>::Error {
                    message: "internal error".to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
            }
            error @ Self::NotFound(class, id) => {
                error!("{error}");
                let response = MyResponse::<()>::Error {
                    message: format!("{class} with id {id} not found"),
                };
                (StatusCode::NOT_FOUND, Json(response)).into_response()
            }
            error @ Self::Oidc(..) => {
                error!("{error}");
                let response = MyResponse::<()>::Error {
                    message: "internal error".to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
            }
            error @ Self::OIDCNotInitialized => {
                error!("{error}");
                let response = MyResponse::<()>::Error {
                    message: "internal error".to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
            }
            error @ Self::NotAuthorized => {
                error!("{error}");
                let response = MyResponse::<()>::Error {
                    message: "Not authorized".to_string(),
                };
                (StatusCode::UNAUTHORIZED, Json(response)).into_response()
            }
            error @ Self::Session(..) => {
                error!("{error}");
                let response = MyResponse::<()>::Error {
                    message: "internal error".to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
            }
            error @ Self::Ldap(..) => {
                error!("{error}");
                let response = MyResponse::<()>::Error {
                    message: "internal error".to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
            }
            error @ Self::Ldap3(..) => {
                error!("{error}");
                let response: MyResponse<()> = MyResponse::<()>::Error {
                    message: "internal error".to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
            }
            error @ Self::LdapTooManyResults => {
                error!("{error}");
                let response = MyResponse::<()>::Error {
                    message: "internal error".to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
            }
            error @ Self::Int(..) => {
                error!("{error}");
                let response = MyResponse::<()>::Error {
                    message: "internal error".to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
            }
        }
    }
}

pub struct Response<T>(T);

impl<T> Response<T> {
    pub fn new(data: T) -> Self {
        Self(data)
    }
}

impl<T: Serialize> IntoResponse for Response<T> {
    fn into_response(self) -> AxumResponse {
        MyResponse::Success { data: self.0 }
            .pipe(Json)
            .into_response()
    }
}

pub type Result<T> = RustResult<Response<T>, Error>;
