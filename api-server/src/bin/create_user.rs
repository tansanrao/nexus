use std::io::{self, Write};

use clap::{ArgGroup, Parser};
use sqlx::postgres::PgPoolOptions;

use api_server::auth::passwords::PasswordService;

#[derive(Parser, Debug)]
#[command(
    name = "create_user",
    about = "Create a local Nexus user account",
    group(ArgGroup::new("identity").required(true).args(["email"]))
)]
struct Args {
    /// Email address for the account (case insensitive).
    #[arg(long)]
    email: String,

    /// Plaintext password to hash and store for this user.
    #[arg(long)]
    password: String,

    /// Optional display name to associate with the account.
    #[arg(long)]
    display_name: Option<String>,

    /// Role to assign (`user` or `admin`).
    #[arg(long, default_value = "user")]
    role: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    let args = Args::parse();
    let email = args.email.trim().to_lowercase();

    if !email.contains('@') {
        writeln!(io::stderr(), "error: email must contain '@'")?;
        std::process::exit(1);
    }

    let role = match args.role.trim().to_lowercase().as_str() {
        "admin" => "admin",
        "user" => "user",
        other => {
            writeln!(
                io::stderr(),
                "error: unsupported role '{other}'. Use 'user' or 'admin'."
            )?;
            std::process::exit(1);
        }
    };

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let mut tx = pool.begin().await?;

    let existing =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE lower(email) = lower($1)")
            .bind(&email)
            .fetch_one(&mut *tx)
            .await?;

    if existing > 0 {
        writeln!(
            io::stderr(),
            "error: a user with email '{email}' already exists."
        )?;
        std::process::exit(1);
    }

    let password_service = PasswordService::new().map_err(|err| {
        io::Error::new(io::ErrorKind::Other, format!("argon2 init failed: {err}"))
    })?;
    let password_hash = password_service
        .hash_password(&args.password)
        .map_err(|err| {
            io::Error::new(io::ErrorKind::Other, format!("password hash failed: {err}"))
        })?;

    let user_id: i32 = sqlx::query_scalar(
        "INSERT INTO users (auth_provider, email, display_name, role) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind("local")
    .bind(&email)
    .bind(args.display_name.as_ref())
    .bind(role)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO local_user_credentials (user_id, password_hash) VALUES ($1, $2)")
        .bind(user_id)
        .bind(password_hash)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    println!("Created {role} user '{email}' with id {user_id}");
    Ok(())
}
