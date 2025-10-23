#[macro_use]
extern crate rocket;

pub mod db;
pub mod error;
pub mod models;
pub mod request_logger;
pub mod routes;
pub mod search;
pub mod sync;
pub mod threading;

use crate::db::NexusDb;
use crate::search::{EmbeddingClient, EmbeddingConfig, SearchConfig};
use crate::request_logger::RequestLogger;
use crate::sync::dispatcher::SyncDispatcher;
use crate::sync::queue::JobQueue;
use env_logger::Env;
use rocket::fairing::AdHoc;
use rocket::http::Method;
use rocket::{Build, Rocket};
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_db_pools::Database;
use rocket_okapi::{
    openapi_get_routes,
    rapidoc::{GeneralConfig, HideShowConfig, RapiDocConfig, make_rapidoc},
    settings::UrlObject,
    swagger_ui::{SwaggerUIConfig, make_swagger_ui},
};
use std::sync::{Arc, Once};
use tokio::sync::Mutex;

static LOGGER: Once = Once::new();

fn init_logger() {
    LOGGER.call_once(|| {
        env_logger::Builder::from_env(
            Env::default().default_filter_or("info,rocket::server=warn,rocket::request=warn"),
        )
        .init();
    });
}

pub fn rocket() -> Rocket<Build> {
    init_logger();

    // Ensure cache directory exists
    let cache_path =
        std::env::var("THREADING_CACHE_BASE_PATH").unwrap_or_else(|_| "./cache".to_string());
    std::fs::create_dir_all(&cache_path).expect("Failed to create cache directory");
    log::info!("Cache directory initialized at: {}", cache_path);

    // Configure CORS
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![
                Method::Get,
                Method::Post,
                Method::Put,
                Method::Delete,
                Method::Patch,
            ]
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
        .attach(cors)
        // Run database migrations on startup
        .attach(AdHoc::try_on_ignite(
            "Run Migrations",
            |rocket| async move {
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
            },
        ))
        // Fairing to clone and manage the database pool for background tasks and job queue
        .attach(AdHoc::try_on_ignite(
            "Manage DB Pool and Job Queue",
            |rocket| async move {
                match NexusDb::fetch(&rocket) {
                    Some(db) => {
                        let pool = (**db).clone();

                        // Initialize job queue with database pool
                        let job_queue = Arc::new(Mutex::new(JobQueue::new(pool.clone())));

                        Ok(rocket.manage(pool).manage(job_queue))
                    }
                    None => Err(rocket),
                }
            },
        ))
        .attach(AdHoc::try_on_ignite(
            "Search Configuration",
            |rocket| async move {
                let mut search_config = SearchConfig::from_env();
                let embedding_config = EmbeddingConfig::from_env();

                let embedding_client = if search_config.enable_semantic {
                    match EmbeddingClient::new(embedding_config.clone()) {
                        Ok(client) => {
                            if let Err(err) = client.healthcheck().await {
                                log::warn!(
                                    "embedding service health check failed: {}. Continuing with best-effort semantic search.",
                                    err
                                );
                            }
                            Some(client)
                        }
                        Err(err) => {
                            log::error!(
                                "failed to initialize embedding client: {}. Semantic search disabled.",
                                err
                            );
                            search_config.enable_semantic = false;
                            None
                        }
                    }
                } else {
                    None
                };

                Ok(rocket
                    .manage(search_config)
                    .manage(embedding_config)
                    .manage(embedding_client))
            },
        ))
        // Spawn sync dispatcher in background
        .attach(AdHoc::on_liftoff("Spawn Sync Dispatcher", |rocket| {
            Box::pin(async move {
                if let Some(pool) = rocket.state::<rocket_db_pools::sqlx::PgPool>() {
                    let dispatcher_pool = pool.clone();
                    let embedding_client = rocket
                        .state::<Option<EmbeddingClient>>()
                        .and_then(|state| state.clone());
                    let search_config = rocket
                        .state::<SearchConfig>()
                        .cloned()
                        .unwrap_or_default();
                    let embedding_config = rocket
                        .state::<EmbeddingConfig>()
                        .cloned()
                        .unwrap_or_default();
                    tokio::spawn(async move {
                        log::info!("starting sync dispatcher");
                        let dispatcher =
                            SyncDispatcher::new(dispatcher_pool, embedding_client, embedding_config, search_config);
                        dispatcher.run().await
                    });
                } else {
                    log::error!("failed to spawn sync dispatcher: database pool not found");
                }
            })
        }))
        .mount(
            "/api/v1",
            openapi_get_routes![
                // Health routes
                routes::health::health_check,
                // Mailing list routes
                routes::mailing_lists::list_mailing_lists,
                routes::mailing_lists::get_mailing_list,
                routes::mailing_lists::get_mailing_list_with_repos,
                routes::mailing_lists::toggle_mailing_list,
                routes::mailing_lists::seed_mailing_lists,
                // Thread routes
                routes::threads::list_threads,
                routes::threads::search_threads,
                routes::threads::get_thread,
                // Email routes
                routes::emails::get_email,
                // Author routes
                routes::authors::search_authors,
                routes::authors::get_author,
                routes::authors::get_author_emails,
                routes::authors::get_author_threads_started,
                routes::authors::get_author_threads_participated,
                // Stats routes
                routes::stats::get_stats,
                // Admin routes
                routes::admin::start_sync,
                routes::admin::queue_sync,
                routes::admin::get_sync_status,
                routes::admin::cancel_sync,
                routes::admin::reset_db,
                routes::admin::get_database_status,
                routes::admin::get_database_config,
                routes::admin::refresh_search_index,
            ],
        )
        .mount(
            "/api/docs/swagger/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../../v1/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/api/docs/rapidoc/",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("Nexus API", "../../v1/openapi.json")],
                    ..Default::default()
                },
                hide_show: HideShowConfig {
                    allow_spec_url_load: false,
                    allow_spec_file_load: false,
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
}

#[cfg_attr(not(test), allow(dead_code))]
pub mod test_support {
    use rocket::config::LogLevel;
    use rocket::figment::Figment;
    use rocket::local::asynchronous::Client as AsyncClient;
    use rocket::local::blocking::Client;
    use rocket::{Build, Rocket, Route};
    use rocket_db_pools::sqlx::{self, PgPool};

    pub use database::{TestDatabase, TestDatabaseError};

    /// Convenience helpers for seeding auth- and notification-related tables in tests.
    pub struct TestFixtures<'a> {
        pool: &'a PgPool,
    }

    impl<'a> TestFixtures<'a> {
        /// Create a fixture helper bound to the provided pool.
        pub fn new(pool: &'a PgPool) -> Self {
            Self { pool }
        }

        /// Insert a user row and optional local credentials, returning the new user id.
        pub async fn insert_user(
            &self,
            email: &str,
            display_name: Option<&str>,
            role: &str,
            password_hash: Option<&str>,
        ) -> Result<i32, sqlx::Error> {
            let user_id: i32 = sqlx::query_scalar(
                "INSERT INTO users (auth_provider, email, display_name, role) VALUES ($1, $2, $3, $4) RETURNING id",
            )
            .bind("local")
            .bind(email)
            .bind(display_name.map(|name| name.to_string()))
            .bind(role)
            .fetch_one(self.pool)
            .await?;

            if let Some(hash) = password_hash {
                sqlx::query(
                    "INSERT INTO local_user_credentials (user_id, password_hash) VALUES ($1, $2)",
                )
                .bind(user_id)
                .bind(hash)
                .execute(self.pool)
                .await?;
            }

            Ok(user_id)
        }

        /// Record a thread follow preference for a user.
        pub async fn follow_thread(
            &self,
            user_id: i32,
            mailing_list_id: i32,
            thread_id: i32,
            level: &str,
        ) -> Result<(), sqlx::Error> {
            sqlx::query(
                "INSERT INTO user_thread_follows (user_id, mailing_list_id, thread_id, level) VALUES ($1, $2, $3, $4)",
            )
            .bind(user_id)
            .bind(mailing_list_id)
            .bind(thread_id)
            .bind(level)
            .execute(self.pool)
            .await?;

            Ok(())
        }

        /// Insert a notification row for assertion in tests.
        pub async fn insert_notification(
            &self,
            user_id: i32,
            mailing_list_id: i32,
            thread_id: i32,
            email_id: Option<i32>,
        ) -> Result<i32, sqlx::Error> {
            sqlx::query_scalar(
                "INSERT INTO notifications (user_id, mailing_list_id, thread_id, email_id, type) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            )
            .bind(user_id)
            .bind(mailing_list_id)
            .bind(thread_id)
            .bind(email_id)
            .bind("new_reply")
            .fetch_one(self.pool)
            .await
        }
    }

    pub mod database {
        use log::LevelFilter;
        use rocket_db_pools::sqlx::postgres::{PgConnectOptions, PgPoolOptions};
        use rocket_db_pools::sqlx::{self, ConnectOptions, PgPool};
        use testcontainers::{GenericImage, ImageExt, core::WaitFor};
        use testcontainers_modules::testcontainers::{
            ContainerAsync, core::error::TestcontainersError, runners::AsyncRunner,
        };
        use thiserror::Error;
        use tokio::runtime::Handle;
        use uuid::Uuid;

        static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

        #[derive(Debug, Error)]
        pub enum TestDatabaseError {
            #[error("TEST_DATABASE_URL not set")]
            MissingUrl,
            #[error("database error: {0}")]
            Sqlx(#[from] sqlx::Error),
            #[error("migration error: {0}")]
            Migration(#[from] sqlx::migrate::MigrateError),
            #[error("container error: {0}")]
            Container(#[from] TestcontainersError),
        }

        /// Ephemeral database factory for integration tests.
        pub struct TestDatabase {
            pool: Option<PgPool>,
            admin_options: PgConnectOptions,
            database_name: String,
            container: Option<ContainerAsync<GenericImage>>,
        }

        impl TestDatabase {
            /// Provision a fresh database by launching a disposable Postgres container.
            pub async fn new_from_env() -> Result<Self, TestDatabaseError> {
                Self::new().await
            }

            /// Provision a fresh database given a base connection string.
            pub async fn new() -> Result<Self, TestDatabaseError> {
                let image = GenericImage::new("tensorchord/vchord-postgres", "pg18-v0.5.3")
                    .with_wait_for(WaitFor::message_on_stdout(
                        "database system is ready to accept connections",
                    ))
                    .with_wait_for(WaitFor::message_on_stderr(
                        "database system is ready to accept connections",
                    ));

                let request = image
                    .with_env_var("POSTGRES_DB", "postgres")
                    .with_env_var("POSTGRES_USER", "postgres")
                    .with_env_var("POSTGRES_PASSWORD", "postgres")
                    .with_cmd([
                        "-c".to_string(),
                        "shared_preload_libraries=vchord".to_string(),
                    ]);

                let container = request.start().await?;

                let host = container.get_host().await?.to_string();
                let port = container.get_host_port_ipv4(5432).await?;
                let admin_url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);

                let base_options: PgConnectOptions =
                    admin_url.parse().map_err(TestDatabaseError::Sqlx)?;
                let base_options = base_options.log_statements(LevelFilter::Off);

                let base_name = base_options
                    .get_database()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "postgres".to_string());

                let admin_options = base_options.clone().database("postgres");
                let admin_pool = PgPoolOptions::new()
                    .max_connections(1)
                    .connect_with(admin_options.clone())
                    .await
                    .map_err(TestDatabaseError::Sqlx)?;

                let new_db_name = format!("{}_{}", base_name, Uuid::new_v4().simple());
                let create_sql = format!("CREATE DATABASE \"{}\" TEMPLATE template0", new_db_name);
                sqlx::query(&create_sql)
                    .execute(&admin_pool)
                    .await
                    .map_err(TestDatabaseError::Sqlx)?;

                let pool = PgPoolOptions::new()
                    .max_connections(5)
                    .connect_with(base_options.clone().database(&new_db_name))
                    .await
                    .map_err(TestDatabaseError::Sqlx)?;

                MIGRATOR.run(&pool).await?;

                Ok(Self {
                    pool: Some(pool),
                    admin_options,
                    database_name: new_db_name,
                    container: Some(container),
                })
            }

            /// Cloneable connection pool for use in tests and Rocket state.
            pub fn pool(&self) -> &PgPool {
                self.pool.as_ref().expect("test database pool is available")
            }

            /// Convenience method returning a clone of the pooled connection handle.
            pub fn pool_clone(&self) -> PgPool {
                self.pool().clone()
            }

            /// Re-run migrations to ensure schema freshness (idempotent).
            pub async fn reset(&self) -> Result<(), TestDatabaseError> {
                MIGRATOR.run(self.pool()).await?;
                Ok(())
            }

            /// Close pool connections and drop the ephemeral database.
            pub async fn close(mut self) -> Result<(), TestDatabaseError> {
                if let Some(pool) = self.pool.take() {
                    pool.close().await;
                }

                drop_database_with_fallback(self.admin_options.clone(), &self.database_name)
                    .await
                    .map_err(TestDatabaseError::Sqlx)?;

                if let Some(container) = self.container.take() {
                    drop(container);
                }

                Ok(())
            }
        }

        async fn drop_database_with_fallback(
            admin_options: PgConnectOptions,
            database_name: &str,
        ) -> Result<(), sqlx::Error> {
            let admin_pool = PgPoolOptions::new()
                .max_connections(1)
                .connect_with(admin_options)
                .await?;

            let drop_force = format!("DROP DATABASE \"{}\" WITH (FORCE)", database_name);
            match sqlx::query(&drop_force).execute(&admin_pool).await {
                Ok(_) => Ok(()),
                Err(err) if force_drop_unsupported(&err) => {
                    let drop_sql = format!("DROP DATABASE \"{}\"", database_name);
                    sqlx::query(&drop_sql).execute(&admin_pool).await?;
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }

        fn force_drop_unsupported(err: &sqlx::Error) -> bool {
            matches!(
                err,
                sqlx::Error::Database(db_err)
                    if db_err
                        .code()
                        .map(|code| code == "42601" || code == "0A000")
                        .unwrap_or(false)
            )
        }

        impl Drop for TestDatabase {
            fn drop(&mut self) {
                if let Some(pool) = self.pool.take() {
                    let admin_options = self.admin_options.clone();
                    let db_name = self.database_name.clone();
                    if let Ok(handle) = Handle::try_current() {
                        handle.spawn(async move {
                            pool.close().await;
                            let _ =
                                drop_database_with_fallback(admin_options.clone(), &db_name).await;
                        });
                    } else {
                        std::thread::spawn(move || {
                            if let Ok(rt) = tokio::runtime::Runtime::new() {
                                rt.block_on(async move {
                                    pool.close().await;
                                    let _ = drop_database_with_fallback(
                                        admin_options.clone(),
                                        &db_name,
                                    )
                                    .await;
                                });
                            }
                        });
                    }
                }

                if let Some(container) = self.container.take() {
                    drop(container);
                }
            }
        }
    }

    /// Builder for constructing Rocket instances tailored for integration tests.
    #[derive(Default)]
    pub struct TestRocketBuilder {
        figment: Figment,
        mounts: Vec<(String, Vec<Route>)>,
        pg_pool: Option<PgPool>,
    }

    impl TestRocketBuilder {
        /// Start a builder with sensible defaults: random port, logging disabled.
        pub fn new() -> Self {
            let figment = rocket::Config::figment()
                .merge(("port", 0))
                .merge(("log_level", LogLevel::Off))
                .merge(("cli_colors", false));

            Self {
                figment,
                mounts: Vec::new(),
                pg_pool: None,
            }
        }

        /// Mount routes under `/api/v1`.
        pub fn mount_api_routes(mut self, routes: Vec<Route>) -> Self {
            self.mounts.push(("/api/v1".to_string(), routes));
            self
        }

        /// Manage a `PgPool` instance for tests that exercise database-backed routes.
        pub fn manage_pg_pool(mut self, pool: PgPool) -> Self {
            self.pg_pool = Some(pool);
            self
        }

        /// Finish building the Rocket instance.
        pub fn build(self) -> Rocket<Build> {
            let mut rocket = rocket::custom(self.figment);

            for (base, routes) in self.mounts {
                rocket = rocket.mount(base, routes);
            }

            if let Some(pool) = self.pg_pool {
                rocket = rocket.manage(pool);
            }

            rocket
        }

        /// Convenience helper to produce a blocking local client.
        pub fn blocking_client(self) -> Client {
            Client::tracked(self.build()).expect("valid Rocket instance")
        }

        /// Convenience helper to produce an asynchronous local client.
        pub async fn async_client(self) -> AsyncClient {
            AsyncClient::tracked(self.build())
                .await
                .expect("valid Rocket instance")
        }
    }
}
