use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Data, Request, Response};
use std::time::Instant;

/// Fairing to log one line per HTTP request with timing
pub struct RequestLogger;

#[rocket::async_trait]
impl Fairing for RequestLogger {
    fn info(&self) -> Info {
        Info {
            name: "Request Logger",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        // Store request start time in local cache
        request.local_cache(|| Instant::now());
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        // Calculate request duration
        let start_time = request.local_cache(|| Instant::now());
        let duration = start_time.elapsed();

        // Get request details
        let method = request.method();
        let uri = request.uri();
        let status = response.status();

        // Log single line with essential info
        log::info!(
            "{} {} -> {} ({:.2}ms)",
            method,
            uri,
            status.code,
            duration.as_secs_f64() * 1000.0
        );
    }
}
