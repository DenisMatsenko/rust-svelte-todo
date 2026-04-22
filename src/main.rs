mod auth;
mod config;
mod db;
mod error;
mod models;
mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new(
                    "rust_svelte_todo=debug,tower_http=debug,axum::rejection=trace",
                )
            }),
        )
        .init();

    let config = config::load_config()
        .await
        .expect("Failed to load configuration from environment");

    let pool = sqlx::PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let db = db::DatabaseService::new(pool);
    let auth = auth::AuthService::new(db.clone(), config.jwt_secret);
    let app: axum::Router = routes::build_router(db, auth);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!();
    println!(r"  $$$$$$$\            $$\            $$\                         ");
    println!(r"  $$  __$$\           $$ |           $$ |                        ");
    println!(r"  $$ |  $$ | $$$$$$\  $$ | $$$$$$\ $$$$$$\    $$$$$$\   $$$$$$\ ");
    println!(r"  $$$$$$$  |$$  __$$\ $$ | \____$$\\_$$  _|  $$  __$$\ $$  __$$\");
    println!(r"  $$  __$$< $$$$$$$$ |$$ | $$$$$$$ | $$ |    $$ /  $$ |$$ /  $$ |");
    println!(r"  $$ |  $$ |$$   ____|$$ |$$  __$$ | $$ |$$\ $$ |  $$ |$$ |  $$ |");
    println!(r"  $$ |  $$ |\$$$$$$$\ $$ |\$$$$$$$ | \$$$$  |\$$$$$$  |\$$$$$$  |");
    println!(r"  \__|  \__| \_______|\__| \_______|  \____/  \______/  \______/ ");
    println!();
    println!("  ┌──────────────────────────────────────────────────────────┐");
    println!("  │                      API  DOCS                           │");
    println!("  ├───────────────┬──────────────────────────────────────────┤");
    println!("  │  Server       │  http://localhost:3000                   │");
    println!("  │  Swagger UI   │  http://localhost:3000/swagger/          │");
    println!("  │  Scalar UI    │  http://localhost:3000/scalar/           │");
    println!("  │  OpenAPI JSON │  http://localhost:3000/openapi.json      │");
    println!("  └───────────────┴──────────────────────────────────────────┘");
    println!();

    axum::serve(listener, app).await.unwrap();
}
