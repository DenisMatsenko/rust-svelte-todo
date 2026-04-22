pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
}

pub async fn load_config() -> Result<Config, std::env::VarError> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    Ok(Config {
        database_url,
        jwt_secret,
    })
}
