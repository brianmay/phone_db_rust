use axum_test::TestServer;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn test(pool: PgPool) {
    let app = backend::get_router(pool);

    let server = TestServer::new(app).unwrap();

    // Get the request.
    let response = server.get("/api/healthcheck").await;

    // Assertions.
    response.assert_status_ok();
    response.assert_json(&json!({
        "status": "Success",
        "data": None::<()>
    }));
}
