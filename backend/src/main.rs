use axum::extract::{FromRef, State};
use axum::{response::IntoResponse, routing::get, Json, Router};
use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::net::TcpListener;

#[derive(FromRef, Clone)]
pub struct AppState {
    db: PgPool,
}

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must set");
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
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

    let app = Router::new()
        .route("/api/healthcheck", get(health_check_handler))
        .with_state(AppState { db: pool });

    println!("Server started successfully at 0.0.0.0:8080");

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

pub async fn health_check_handler(State(db): State<PgPool>) -> impl IntoResponse {
    const MESSAGE: &str = "API Services";

    let result = sqlx::query!("SELECT 1 as result").fetch_one(&db).await;

    let json_response = match result {
        Ok(_v) => {
            serde_json::json!({
                "status": "ok",
                "message": MESSAGE
            })
        }
        Err(err) => {
            serde_json::json!({
                "status": "err",
                "message": err.to_string(),
            })
        }
    };

    Json(json_response)
}
