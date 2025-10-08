#[macro_use]
extern crate rocket;

use env_logger::Env;

mod db;
mod error;
mod models;
mod routes;
mod seed_data;
mod sync;
mod threading;

use db::LinuxKbDb;
use rocket::fairing::AdHoc;
use rocket::http::Method;
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_db_pools::Database;
use std::sync::Arc;
use sync::queue::JobQueue;
use tokio::sync::Mutex;

#[launch]
fn rocket() -> _ {
    // Initialize logger (defaults to INFO level, set RUST_LOG env var to override)
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    log::info!("Starting Linux Kernel Knowledge Base API Server");

    // Configure CORS
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Put, Method::Delete, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true)
        .to_cors()
        .expect("Error creating CORS");

    rocket::build()
        .attach(LinuxKbDb::init())
        .attach(cors)
        // Fairing to clone and manage the database pool for background tasks and job queue
        .attach(AdHoc::try_on_ignite("Manage DB Pool and Job Queue", |rocket| async move {
            match LinuxKbDb::fetch(&rocket) {
                Some(db) => {
                    let pool = (**db).clone();

                    // Initialize job queue with database pool
                    let job_queue = Arc::new(Mutex::new(JobQueue::new(pool.clone())));

                    Ok(rocket.manage(pool).manage(job_queue))
                }
                None => Err(rocket),
            }
        }))
        .mount(
            "/api",
            routes![
                // Mailing list routes
                routes::mailing_lists::list_mailing_lists,
                routes::mailing_lists::get_mailing_list,
                routes::mailing_lists::get_mailing_list_with_repos,
                routes::mailing_lists::toggle_mailing_list,
                routes::mailing_lists::seed_mailing_lists,
                // Existing routes
                routes::threads::list_threads,
                routes::threads::search_threads,
                routes::threads::get_thread,
                routes::emails::get_email,
                routes::authors::search_authors,
                routes::authors::get_author,
                routes::authors::get_author_emails,
                routes::authors::get_author_threads_started,
                routes::authors::get_author_threads_participated,
                routes::stats::get_stats,
                // Admin routes
                routes::admin::queue_sync,
                routes::admin::get_sync_status,
                routes::admin::cancel_sync,
                routes::admin::reset_db,
                routes::admin::get_database_status,
            ],
        )
}
