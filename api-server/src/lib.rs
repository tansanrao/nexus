#[macro_use]
extern crate rocket;

pub mod db;
pub mod error;
pub mod models;
pub mod request_logger;
pub mod routes;
pub mod sync;
pub mod threading;

use crate::db::NexusDb;
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
        // Spawn sync dispatcher in background
        .attach(AdHoc::on_liftoff("Spawn Sync Dispatcher", |rocket| {
            Box::pin(async move {
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
    use rocket_db_pools::sqlx::PgPool;

    pub use database::{TestDatabase, TestDatabaseError};

    pub mod database {
        use log::LevelFilter;
        use rocket_db_pools::sqlx::postgres::{PgConnectOptions, PgPoolOptions};
        use rocket_db_pools::sqlx::{self, ConnectOptions, PgPool};
        use testcontainers_modules::{
            postgres::Postgres,
            testcontainers::{
                ContainerAsync, core::error::TestcontainersError, runners::AsyncRunner,
            },
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
            container: Option<ContainerAsync<Postgres>>,
        }

        impl TestDatabase {
            /// Provision a fresh database by launching a disposable Postgres container.
            pub async fn new_from_env() -> Result<Self, TestDatabaseError> {
                Self::new().await
            }

            /// Provision a fresh database given a base connection string.
            pub async fn new() -> Result<Self, TestDatabaseError> {
                let container = Postgres::default().with_host_auth().start().await?;

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
