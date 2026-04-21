mod auth;
mod db;
mod error;
mod models;
mod routes;

use sqlx::PgPool;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(
                    "rust_svelte_todo=debug,tower_http=debug,axum::rejection=trace",
                )),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let app: axum::Router = routes::build_router(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on http://localhost:3000");
    tracing::info!("swagger UI at http://localhost:3000/swagger/");
    tracing::info!("scalar UI at http://localhost:3000/scalar/");
    axum::serve(listener, app).await.unwrap();
}
