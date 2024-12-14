use std::env;
use std::sync::Arc;

use arc_swap::ArcSwap;
use axum::extract::FromRef;
use axum::Extension;
use axum::{routing::get, Router};
use common::User;
use oidc::Client;
use simple_ldap::pool as ldap_pool;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tap::Pipe;
use time::Duration;
use tokio::net::TcpListener;
use tower_sessions::cookie::SameSite;
use tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store_chrono::PostgresStore;

mod errors;
mod handlers;
mod ldap;
mod oidc;
pub mod types;

#[derive(Debug, Clone)]
pub struct Authentication {
    pub username: String,
    pub password: String,
}

#[derive(FromRef, Clone)]
pub struct AppState {
    db: PgPool,
    oidc_client: Arc<ArcSwap<Option<Client>>>,
    authentication: Arc<Authentication>,
    ldap: Ldap,
}

#[derive(Clone)]
pub struct Ldap {
    pool: Arc<ldap_pool::LdapPool>,
    base_dn: Arc<String>,
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

    let ldap = connect_ldap().await;

    let app = get_router(pool, ldap).await;

    println!("🚀 Server running at http://{}", http_listen);
    let listener = TcpListener::bind(&http_listen).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn connect_ldap() -> Ldap {
    let ldap_server = env::var("LDAP_SERVER").expect("LDAP_SERVER must be set");
    let ldap_port = env::var("LDAP_PORT").expect("LDAP_PORT must be set");
    let ldap_base_dn = env::var("LDAP_BASE_DN").expect("LDAP_BASE_DN must be set");
    let ldap_username = env::var("LDAP_USERNAME").expect("LDAP_USERNAME must be set");
    let ldap_password = env::var("LDAP_PASSWORD").expect("LDAP_PASSWORD must be set");
    let url = format!("ldap://{}:{}", ldap_server, ldap_port);

    let ldap_config = ldap_pool::LdapConfig {
        bind_dn: ldap_username,
        bind_pw: ldap_password,
        ldap_url: url,
        pool_size: 1,
        dn_attribute: Some("telephoneNumber".to_string()),
    };

    let pool = ldap_pool::build_connection_pool(&ldap_config).await;

    Ldap {
        pool: Arc::new(pool),
        base_dn: Arc::new(ldap_base_dn),
    }
}

pub async fn get_router(pool: sqlx::PgPool, ldap: Ldap) -> Router {
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

    let authentication = {
        let username = env::var("PHONE_USERNAME").expect("PHONE_USERNAME must be set");
        let password = env::var("PHONE_PASSWORD").expect("PHONE_PASSWORD must be set");
        Authentication { username, password }.pipe(Arc::new)
    };

    let state = AppState {
        db: pool,
        oidc_client,
        authentication,
        ldap,
    };

    Router::new()
        .route("/", get(index_handler))
        .nest(
            "/api/phone_calls",
            handlers::phone_calls::router(state.clone()),
        )
        .nest("/api/contacts", handlers::contacts::router(state.clone()))
        // .layer(axum::middleware::from_fn_with_state(
        //     state.clone(),
        //     oidc::middleware::auth,
        // ))
        .layer(session_layer)
        .nest("/api", handlers::incoming_calls::router(state.clone()))
        .route(
            "/api/healthcheck",
            get(handlers::health_check::health_check_handler),
        )
        .with_state(state)
}

pub async fn get_test_router(pool: sqlx::PgPool) -> Router {
    let ldap = connect_ldap().await;

    let user = User {
        sub: "test".to_string(),
        email: "test@example.org".to_string(),
        name: "Test User".to_string(),
        is_admin: false,
        groups: vec![],
    };

    let authentication = {
        Authentication {
            username: "test".to_string(),
            password: "test".to_string(),
        }
        .pipe(Arc::new)
    };

    let state = AppState {
        db: pool,
        oidc_client: Arc::new(ArcSwap::new(Arc::new(None))),
        authentication,
        ldap,
    };

    Router::new()
        .route("/", get(index_handler))
        .nest(
            "/api/phone_calls",
            handlers::phone_calls::router(state.clone()),
        )
        .nest("/api/contacts", handlers::contacts::router(state.clone()))
        .layer(Extension(Arc::new(user)))
        .nest("/api", handlers::incoming_calls::router(state.clone()))
        .route(
            "/api/healthcheck",
            get(handlers::health_check::health_check_handler),
        )
        .with_state(state)
}

#[axum::debug_handler]
pub async fn index_handler(Extension(user): Extension<Arc<User>>) -> String {
    format!("Hello, {user:#?}!")
}
