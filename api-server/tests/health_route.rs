use api_server::models::ApiResponse;
use api_server::routes::health::{HealthResponse, live_health};
use api_server::test_support::TestRocketBuilder;
use rocket::http::Status;
use rocket::routes;

#[test]
fn health_endpoint_returns_ok() {
    let client = TestRocketBuilder::new()
        .mount_api_routes(routes![live_health])
        .blocking_client();

    let response = client.get("/api/v1/health/live").dispatch();
    assert_eq!(response.status(), Status::Ok);

    let payload: ApiResponse<HealthResponse> = response.into_json().expect("valid JSON payload");
    assert_eq!(payload.data.status, "ok");
}
