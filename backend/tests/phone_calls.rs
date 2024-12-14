use axum_test::TestServer;
use backend::types::Contact;
use common::{Action, Page, PhoneCall, Response};
use sqlx::PgPool;

#[sqlx::test]
async fn test_no_phone_calls(db: PgPool) {
    let app = backend::get_test_router(db.clone()).await;

    let server = TestServer::new(app).unwrap();

    // Get the request.
    let response = server
        .get("/api/phone_calls")
        .add_query_param("page", 0)
        .add_query_param("per_page", 10)
        .await;

    // Assertions.
    response.assert_status_ok();
    let response = response.json::<Response<Page<PhoneCall>>>();

    let Response::Success { data: response } = response else {
        panic!("Expected a success response, got: {:?}", response);
    };

    assert_eq!(response.page, 0);
    assert_eq!(response.total, 0);
    assert_eq!(response.per_page, 10);
    assert_eq!(response.data.len(), 0);
}

#[sqlx::test]
async fn test_one_phone_call(db: PgPool) {
    let app = backend::get_test_router(db.clone()).await;
    let server = TestServer::new(app).unwrap();
    let time = chrono::Utc::now();

    let contact = sqlx::query_as!(
        Contact,
        r#"
        INSERT INTO contacts (phone_number, action, inserted_at, updated_at)
        VALUES ($1,$2,$3,$3)
        RETURNING *
        "#,
        "0412345678",
        Action::Allow.as_str(),
        time
    )
    .fetch_one(&db)
    .await
    .unwrap();

    sqlx::query!(
        r#"
        INSERT INTO phone_calls (action, contact_id, destination_number, inserted_at, updated_at)
        VALUES ($1, $2, $3, $4, $4)
        RETURNING id
        "#,
        contact.action.as_str(),
        contact.id,
        "0312345678",
        time
    )
    .fetch_one(&db)
    .await
    .unwrap();

    // Get the request.
    let response = server
        .get("/api/phone_calls")
        .add_query_param("page", 0)
        .add_query_param("per_page", 10)
        .await;

    // Assertions.
    response.assert_status_ok();
    let response = response.json::<Response<Page<PhoneCall>>>();

    let Response::Success { data: response } = response else {
        panic!("Expected a success response, got: {:?}", response);
    };

    assert_eq!(response.page, 0);
    assert_eq!(response.total, 1);
    assert_eq!(response.per_page, 10);
    assert_eq!(response.data.len(), 1);
    assert_eq!(
        response.data[0].destination_number,
        Some("0312345678".to_string())
    );
    assert_eq!(response.data[0].action, Action::Allow);
}
