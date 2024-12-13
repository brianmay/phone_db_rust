use axum_extra::headers::{authorization::Credentials, Authorization};
use axum_test::TestServer;
use common::{IncomingPhoneCallRequest, IncomingPhoneCallResponse, Response};
use sqlx::PgPool;

#[sqlx::test]
async fn test(db: PgPool) {
    let app = backend::get_test_router(db.clone());

    let server = TestServer::new(app).unwrap();

    let authorization_header = Authorization::basic("test", "test").0.encode();
    let authorization_header = authorization_header.to_str().unwrap();

    let request = IncomingPhoneCallRequest {
        phone_number: "1234567890".to_string(),
        destination_number: "0987654321".to_string(),
    };

    // Get the request.
    let response = server
        .post("/api/incoming_call/")
        .authorization(authorization_header)
        .json(&request)
        .await;

    // Assertions.
    response.assert_status_ok();
    let response = response.json::<Response<IncomingPhoneCallResponse>>();

    let Response::Success { data: response } = response else {
        panic!("Expected a success response, got: {:?}", response);
    };

    assert_eq!(response.name, None);
    assert_eq!(response.action, common::Action::Allow);
}
