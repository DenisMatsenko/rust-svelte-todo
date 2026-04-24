pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub mongo_db_url: String,
    /// Comma-separated list of allowed CORS origins, e.g.
    /// `http://localhost:3001,http://172.16.33.40:3001`
    pub cors_origins: Vec<String>,
}

pub async fn load_config() -> Result<Config, std::env::VarError> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let mongo_db_url = std::env::var("MONGO_DB_URL").expect("MONGO_DB_URL must be set");
    let cors_origins = std::env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3001".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    Ok(Config {
        database_url,
        jwt_secret,
        mongo_db_url,
        cors_origins,
    })
}
