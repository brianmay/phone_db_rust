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
}

impl IntoResponse for Error {
    fn into_response(self) -> AxumResponse {
        match self {
            Self::Sqlx(error) => {
                error!("database error: {error}");
                let response = MyResponse::<()>::Error {
                    message: "internal error".to_string(),
                };
                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            }
            Self::NotFound(class, id) => {
                let response = MyResponse::<()>::Error {
                    message: format!("{class} with id {id} not found"),
                };
                (StatusCode::NOT_FOUND, Json(response)).into_response()
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
