use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use arc_swap::ArcSwap;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::FromRef;
use axum::response::Html;
use axum::Extension;
use axum::{routing::get, Router};
use dioxus::prelude::*;
use dioxus_liveview::{interpreter_glue, LiveViewError, LiveViewSocket};
use futures_util::{SinkExt, StreamExt};
use handlers::assets::Manifest;
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

use common::PhoneCall;
use common::User;
use errors::Error;

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
    static_path: Arc<PathBuf>,
    manifest: Arc<Manifest>,
}

#[derive(Clone)]
pub struct Ldap {
    pool: Arc<ldap_pool::LdapPool>,
    base_dn: Arc<String>,
}

async fn get_phone_calls(db: &PgPool) -> Result<Vec<PhoneCall>, Error> {
    sqlx::query_as!(
        PhoneCall,
        r#"
        SELECT phone_calls.*, contacts.name as contact_name, contacts.phone_number as contact_phone_number, contacts.action as contact_action, contacts.comments as contact_comments
        FROM phone_calls
        INNER JOIN contacts ON contacts.id = phone_calls.contact_id
        ORDER BY inserted_at, id DESC
        LIMIT 10
        "#,
    )
    .fetch_all(db)
    .await?
    .pipe(Ok)
}

fn app(state: AppState) -> Element {
    let mut num = use_signal(|| 0);

    let static_path = state.static_path.display();

    rsx! {
        div {
            "hello axum! {num} {static_path}"
            button { onclick: move |_| num += 1, "Increment" }
        }
    }
}

/// Convert an Axum WebSocket into a `LiveViewSocket`.
///
/// This is required to launch a LiveView app using the Axum web framework.
pub fn axum_socket(ws: WebSocket) -> impl LiveViewSocket {
    ws.map(transform_rx)
        .with(transform_tx)
        .sink_map_err(|_| LiveViewError::SendingFailed)
}

fn transform_rx(message: Result<Message, axum::Error>) -> Result<Vec<u8>, LiveViewError> {
    message
        .map_err(|_| LiveViewError::SendingFailed)?
        .into_text()
        .map(|s| s.into_bytes())
        .map_err(|_| LiveViewError::SendingFailed)
}

async fn transform_tx(message: Vec<u8>) -> Result<Message, axum::Error> {
    Ok(Message::Binary(message))
}

trait MyRouter {
    fn with_virtual_dom2(
        self,
        route: &str,
        f: fn(AppState) -> Element,
        state: AppState,
        // app: impl Fn() -> dioxus_core::prelude::VirtualDom + Send + Sync + 'static,
    ) -> Self;
}

impl MyRouter for Router<AppState> {
    fn with_virtual_dom2(
        self,
        route: &str,
        f: fn(AppState) -> Element,
        state: AppState,
        //app: impl Fn() -> dioxus_core::prelude::VirtualDom + Send + Sync + 'static,
    ) -> Self {
        let app = move || VirtualDom::new_with_props::<AppState, _>(f, state.clone());

        let view = dioxus_liveview::LiveViewPool::new();

        let ws_path = if route.ends_with("/") {
            format!("{}ws", route)
        } else {
            format!("{}/ws", route)
        };

        let title = "Phone DB";

        let index_page_with_glue = move |glue: &str| {
            Html(format!(
                r#"
        <!DOCTYPE html>
        <html>
            <head><title>{title}</title></head>
            <body><div id="main"></div></body>
            {glue}
        </html>
        "#,
            ))
        };

        let app = Arc::new(app);

        self.route(
            &ws_path,
            get(move |ws: WebSocketUpgrade| async move {
                let app = app.clone();
                ws.on_upgrade(move |socket| async move {
                    _ = view
                        .launch_virtualdom(axum_socket(socket), move || app())
                        .await;
                })
            }),
        )
        .route(
            route,
            get(move || async move { index_page_with_glue(&interpreter_glue(&ws_path)) }),
        )
    }
}

fn test(state: AppState) -> Element {
    let phone_calls = use_resource(move || {
        let db = state.db.clone();
        async move { get_phone_calls(&db).await }
    });

    match &*phone_calls.read_unchecked() {
        Some(Ok(list)) => {
            // if it is, render the stories
            rsx! {
                div {
                    // iterate over the stories with a for loop
                    for story in list {
                        // render every story with the StoryListing component
                        "{story.id.to_string()}"
                        "{story.contact_id.to_string()}"
                        br {}
                    }
                }
            }
        }
        Some(Err(err)) => {
            // if there was an error, render the error
            rsx! {"An error occurred while fetching stories {err}"}
        }
        None => {
            // if the future is not resolved yet, render a loading message
            rsx! {"Loading items"}
        }
    }
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
    let static_path = env::var("STATIC_PATH")
        .expect("STATIC_PATH must be set")
        .pipe(PathBuf::from);

    let manifest = Manifest::load(&static_path)
        .await
        .expect("failed to load manifest");

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
        static_path: Arc::new(static_path),
        manifest: Arc::new(manifest),
    };

    Router::new()
        //.route("/", get(index_handler))
        .with_virtual_dom2("/", app, state.clone())
        .with_virtual_dom2("/test", test, state.clone())
        .nest(
            "/api/phone_calls",
            handlers::phone_calls::router(state.clone()),
        )
        .nest("/api/contacts", handlers::contacts::router(state.clone()))
        .fallback(handlers::assets::fallback_handler)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            oidc::middleware::auth,
        ))
        .layer(session_layer)
        .nest("/api", handlers::incoming_calls::router(state.clone()))
        .route(
            "/api/healthcheck",
            get(handlers::health_check::health_check_handler),
        )
        .with_state(state.clone())
        .route(
            "/_dioxus",
            get(move |ws: WebSocketUpgrade| async move {
                ws.on_upgrade(move |mut socket| async move {
                    // When the WebSocket is upgraded, launch the LiveView with the app component
                    while socket.recv().await.is_some() {}
                })
            }),
        )
}

pub async fn get_test_router(pool: sqlx::PgPool) -> Router {
    let static_path = env::var("STATIC_PATH")
        .expect("STATIC_PATH must be set")
        .pipe(PathBuf::from);

    let manifest = Manifest::load(&static_path)
        .await
        .expect("failed to load manifest");

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
        static_path: Arc::new(static_path),
        manifest: Arc::new(manifest),
    };

    Router::new()
        // .route("/", get(index_handler))
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

// #[axum::debug_handler]
// pub async fn index_handler(Extension(user): Extension<Arc<User>>) -> String {
//     format!("Hello, {user:#?}!")
// }
