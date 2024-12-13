use axum::extract::{FromRef, State};
use axum::{routing::get, Router};
use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::net::TcpListener;

pub mod contacts;
mod errors;
mod incoming_calls;
mod phone_calls;

use errors::{Response, Result};

#[derive(FromRef, Clone)]
pub struct AppState {
    db: PgPool,
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

    let app = get_router(pool);

    println!("🚀 Server running at http://{}", http_listen);
    let listener = TcpListener::bind(&http_listen).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

pub fn get_router(pool: sqlx::Pool<sqlx::Postgres>) -> Router {
    let state = AppState { db: pool };

    Router::new()
        .nest("/api", incoming_calls::router(state.clone()))
        .nest("/api/phone_calls", phone_calls::router(state.clone()))
        .nest("/api/contacts", contacts::router(state.clone()))
        .route("/api/healthcheck", get(health_check_handler))
        .with_state(state)
}

#[axum::debug_handler]
pub async fn health_check_handler(State(db): State<PgPool>) -> Result<()> {
    sqlx::query!("SELECT 1 as result").fetch_one(&db).await?;
    let response = Response::new(());
    Ok(response)
}
