use api_server::routes::health::{HealthResponse, health_check};
use api_server::test_support::TestRocketBuilder;
use rocket::http::Status;
use rocket::routes;

#[test]
fn health_endpoint_returns_ok() {
    let client = TestRocketBuilder::new()
        .mount_api_routes(routes![health_check])
        .blocking_client();

    let response = client.get("/api/v1/health").dispatch();
    assert_eq!(response.status(), Status::Ok);

    let payload: HealthResponse = response.into_json().expect("valid JSON payload");
    assert_eq!(payload.status, "ok");
}
