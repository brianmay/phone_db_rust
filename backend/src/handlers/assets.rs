use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{
    body::Body,
    extract::{Request, State},
    http::Method,
    response::{Html, IntoResponse, Response},
    Extension,
};
use serde::Deserialize;
use thiserror::Error;
use tokio::fs;
use tracing::error;

use crate::{errors::Error as MyError, AppState};
use common::User;

#[derive(Debug, Error)]
pub enum ManifestLoadError {
    #[error("failed to load manifest.json")]
    LoadError(#[from] std::io::Error),

    #[error("failed to parse manifest.json")]
    ParseError(#[from] serde_json::Error),
}

#[derive(Deserialize)]
pub struct Manifest(HashMap<String, String>, PathBuf);

impl Manifest {
    pub async fn load(static_path: &Path) -> Result<Self, ManifestLoadError> {
        let manifest_path = static_path.join("manifest.json");
        let manifest_str = fs::read_to_string(manifest_path).await?;
        let manifest: HashMap<String, String> = serde_json::from_str(&manifest_str)?;
        Ok(Self(manifest, static_path.to_owned()))
    }

    // async fn load_or_default(static_path: &Path) -> Self {
    //     Self::load(static_path).await.unwrap_or_else(|err| {
    //         tracing::error!("failed to load manifest: {}", err);
    //         Self(HashMap::new(), static_path.to_owned())
    //     })
    // }

    fn get_internal<'a>(&'a self, key: &'a str) -> &'a str {
        self.0.get(key).map_or_else(
            || {
                error!("Cannot find {key} in manifest.json");
                key
            },
            |s| s.as_str(),
        )
    }

    fn get_path(&self, key: &str) -> PathBuf {
        self.1.join(self.get_internal(key))
    }

    pub fn get_url(&self, key: &str) -> String {
        format!("/assets/{}", self.get_internal(key))
    }
}

#[axum::debug_handler(state = AppState)]
pub async fn fallback_handler(
    user: Option<Extension<Arc<User>>>,
    State(manifest): State<Arc<Manifest>>,
    req: Request<Body>,
) -> Result<Response, MyError> {
    if req.method() != Method::GET {
        return Err(MyError::MethodNotAllowed);
    }

    if user.is_none() {
        return Err(MyError::NotFound);
    }

    serve_index_html(&manifest).await
}

async fn serve_index_html(manifest: &Manifest) -> Result<Response, MyError> {
    let index_path = manifest.get_path("index.html");
    {
        fs::read_to_string(index_path)
            .await
            .map(|index_content| Html(index_content).into_response())
            .map_err(|err| MyError::AssetLoad("index.html".to_string(), err))
    }
}
