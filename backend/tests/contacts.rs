use axum_test::TestServer;
use backend::types::Contact;
use common::{Action, ContactDetails, Response};
use sqlx::PgPool;

#[sqlx::test]
async fn test_no_contacts(db: PgPool) {
    let app = backend::get_test_router(db).await;

    let server = TestServer::new(app).unwrap();

    // Get the request.
    let response = server.get("/api/contacts").await;

    // Assertions.
    response.assert_status_ok();
    let response = response.json::<Response<Vec<Contact>>>();

    let Response::Success { data: response } = response else {
        panic!("Expected a success response, got: {:?}", response);
    };

    assert_eq!(response.len(), 0);
}

#[sqlx::test]
async fn test_one_contact(db: PgPool) {
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
    let response = server.get("/api/contacts").await;

    // Assertions.
    response.assert_status_ok();
    let response = response.json::<Response<Vec<ContactDetails>>>();

    let Response::Success { data: response } = response else {
        panic!("Expected a success response, got: {:?}", response);
    };

    assert_eq!(response.len(), 1);
    assert_eq!(response[0].phone_number, "0412345678");
    assert_eq!(response[0].action, Action::Allow);
    assert_eq!(response[0].number_calls, Some(1));
}

#[sqlx::test]
async fn test_update_contact(db: PgPool) {
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

    let update_request = contact.get_update_request(
        Action::VoiceMail,
        Some("John Doe".to_string()),
        Some("Test".to_string()),
    );

    let response = server.post("/api/contacts").json(&update_request).await;

    // Assertions.
    response.assert_status_ok();
    let response = response.json::<Response<()>>();

    let Response::Success { data: () } = response else {
        panic!("Expected a success response, got: {:?}", response);
    };

    let contact = sqlx::query_as!(
        Contact,
        r#"
        SELECT *
        FROM contacts
        WHERE id = $1
        "#,
        update_request.id
    )
    .fetch_one(&db)
    .await;

    let contact = match contact {
        Ok(contact) => contact,
        Err(error) => panic!("Failed to fetch contact: {:?}", error),
    };

    assert_eq!(contact.action, Action::VoiceMail);
    assert_eq!(contact.name, Some("John Doe".to_string()));
    assert_eq!(contact.comments, Some("Test".to_string()));
}
