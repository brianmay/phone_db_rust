use std::{collections::HashMap, sync::Arc};

use arc_swap::ArcSwap;
use axum::{
    extract::{Query, Request, State},
    http::uri::PathAndQuery,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use common::User;
use tap::Pipe;
use tower_sessions::Session;

use crate::{errors::Error, oidc::Client};

#[axum::debug_middleware]
pub async fn auth(
    State(oidc_client): State<Arc<ArcSwap<Option<Client>>>>,
    Query(params): Query<HashMap<String, String>>,
    session: Session,
    mut request: Request,
    next: Next,
) -> Result<Response, Error> {
    if request.uri().path() == "/openid_connect_redirect_uri" {
        let code = params.get("code").cloned().unwrap_or_default();

        let state = params
            .get("state")
            .cloned()
            .unwrap_or_else(|| "/".to_string());

        let oidc_client = oidc_client.load();
        let Some(oidc_client) = oidc_client.as_ref() else {
            return Err(Error::OIDCNotInitialized);
        };

        let user = oidc_client.login(&code).await?;

        session.insert("user", Arc::new(user)).await?;
        return Redirect::to(&state).into_response().pipe(Ok);
    }

    let user = session.get::<Arc<User>>("user").await.unwrap_or_default();

    match user {
        Some(user) => {
            request.extensions_mut().insert(user.clone());
            next.run(request).await.pipe(Ok)
        }
        None => {
            let origin_url = request
                .uri()
                .path_and_query()
                .map_or("/", PathAndQuery::as_str);
            let oidc_client = oidc_client.load();
            let Some(oidc_client) = oidc_client.as_ref() else {
                return Err(Error::OIDCNotInitialized);
            };
            let auth_url = oidc_client.get_auth_url(origin_url);
            Ok(Redirect::to(&auth_url).into_response())
        }
    }
}
