#[macro_use]
extern crate rocket;

use env_logger::Env;

mod db;
mod error;
mod models;
mod request_logger;
mod routes;
mod sync;
mod threading;

use db::{NexusDb, BulkWriteDb};
use request_logger::RequestLogger;
use rocket::fairing::AdHoc;
use rocket::http::Method;
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_db_pools::Database;
use std::sync::Arc;
use sync::queue::JobQueue;
use sync::dispatcher::SyncDispatcher;
use tokio::sync::Mutex;

#[launch]
fn rocket() -> _ {
    // Initialize logger with module-specific filtering
    // Suppress verbose Rocket logs, keep our app logs at INFO
    env_logger::Builder::from_env(
        Env::default().default_filter_or("info,rocket::server=warn,rocket::request=warn")
    ).init();

    log::info!("Starting Nexus API Server");

    // Ensure cache directory exists
    let cache_path = std::env::var("THREADING_CACHE_BASE_PATH")
        .unwrap_or_else(|_| "./cache".to_string());
    std::fs::create_dir_all(&cache_path)
        .expect("Failed to create cache directory");
    log::info!("Cache directory initialized at: {}", cache_path);

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
        .attach(RequestLogger)
        .attach(NexusDb::init())
        .attach(BulkWriteDb::init())
        .attach(cors)
        // Run database migrations on startup
        .attach(AdHoc::try_on_ignite("Run Migrations", |rocket| async move {
            match NexusDb::fetch(&rocket) {
                Some(db) => {
                    let pool = (**db).clone();
                    match sync::run_migrations(&pool).await {
                        Ok(_) => {
                            log::info!("database migrations successful");
                            Ok(rocket)
                        }
                        Err(e) => {
                            log::error!("database migrations failed: {}", e);
                            Err(rocket)
                        }
                    }
                }
                None => {
                    log::error!("database pool not available for migrations");
                    Err(rocket)
                }
            }
        }))
        // Fairing to clone and manage the database pool for background tasks and job queue
        .attach(AdHoc::try_on_ignite("Manage DB Pool and Job Queue", |rocket| async move {
            match NexusDb::fetch(&rocket) {
                Some(db) => {
                    let pool = (**db).clone();

                    // Initialize job queue with database pool
                    let job_queue = Arc::new(Mutex::new(JobQueue::new(pool.clone())));

                    Ok(rocket.manage(pool).manage(job_queue))
                }
                None => Err(rocket),
            }
        }))
        // Spawn sync dispatcher in background
        .attach(AdHoc::on_liftoff("Spawn Sync Dispatcher", |rocket| Box::pin(async move {
            if let Some(pool) = rocket.state::<rocket_db_pools::sqlx::PgPool>() {
                let dispatcher_pool = pool.clone();
                tokio::spawn(async move {
                    log::info!("starting sync dispatcher");
                    let dispatcher = SyncDispatcher::new(dispatcher_pool);
                    dispatcher.run().await
                });
            } else {
                log::error!("failed to spawn sync dispatcher: database pool not found");
            }
        })))
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
                routes::admin::start_sync,
                routes::admin::queue_sync,
                routes::admin::get_sync_status,
                routes::admin::cancel_sync,
                routes::admin::reset_db,
                routes::admin::get_database_status,
                routes::admin::get_database_config,
            ],
        )
}
