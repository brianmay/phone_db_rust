use std::env;
use std::sync::Arc;

use arc_swap::ArcSwap;
use axum::extract::{FromRef, State};
use axum::Extension;
use axum::{routing::get, Router};
use common::User;
use oidc::Client;
use sqlx::postgres::{PgPool, PgPoolOptions};
use time::Duration;
use tokio::net::TcpListener;
use tower_sessions::cookie::SameSite;
use tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store_chrono::PostgresStore;

pub mod contacts;
mod errors;
mod incoming_calls;
mod oidc;
mod phone_calls;

use errors::{Response, Result};

#[derive(FromRef, Clone)]
pub struct AppState {
    db: PgPool,
    oidc_client: Arc<ArcSwap<Option<Client>>>,
}

pub async fn main(http_listen: &str, database_url: &str) {
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
    {
        Ok(pool) => {
            println!("✅ Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("❌ Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let app = get_router(pool).await;

    println!("🚀 Server running at http://{}", http_listen);
    let listener = TcpListener::bind(&http_listen).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

pub async fn get_router(pool: sqlx::Pool<sqlx::Postgres>) -> Router {
    let session_store = PostgresStore::new(pool.clone());

    tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)))
        .with_same_site(SameSite::Lax);

    let redirect_uri = "http://localhost:4000/openid_connect_redirect_uri";
    let issuer = env::var("OIDC_DISCOVERY_URL").expect("OIDC_DISCOVERY_URL must be set");
    let client_id = env::var("OIDC_CLIENT_ID").expect("OIDC_CLIENT_ID must be set");
    let client_secret = env::var("OIDC_CLIENT_SECRET").expect("OIDC_CLIENT_SECRET must be set");
    let auth_scope = env::var("OIDC_AUTH_SCOPE").expect("OIDC_AUTH_SCOPE must be set");

    let oidc_config = oidc::Config {
        issuer,
        client_id,
        client_secret,
        redirect_uri: redirect_uri.to_string(),
        scopes: auth_scope,
    };

    let oidc_client = Arc::new(ArcSwap::new(Arc::new(None)));

    {
        let client = oidc_client.clone();
        tokio::spawn(async move {
            loop {
                tracing::info!("refreshing oidc client");
                let new_client = Client::new(&oidc_config).await;
                match new_client {
                    Ok(new_client) => {
                        client.store(Arc::new(Some(new_client)));
                        tokio::time::sleep(tokio::time::Duration::from_secs(10 * 60)).await;
                    }
                    Err(e) => {
                        tracing::error!("failed to refresh oidc client: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    }
                }
            }
        });
    }

    let state = AppState {
        db: pool,
        oidc_client,
    };

    Router::new()
        .route("/", get(index_handler))
        .nest("/api", incoming_calls::router(state.clone()))
        .nest("/api/phone_calls", phone_calls::router(state.clone()))
        .nest("/api/contacts", contacts::router(state.clone()))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            oidc::middleware::auth,
        ))
        .layer(session_layer)
        .route("/api/healthcheck", get(health_check_handler))
        .with_state(state)
}

#[axum::debug_handler]
pub async fn health_check_handler(State(db): State<PgPool>) -> Result<()> {
    sqlx::query!("SELECT 1 as result").fetch_one(&db).await?;
    let response = Response::new(());
    Ok(response)
}

#[axum::debug_handler]
pub async fn index_handler(Extension(user): Extension<Arc<User>>) -> String {
    format!("Hello, {user:#?}!")
}
