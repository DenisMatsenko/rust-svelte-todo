pub mod todos;
pub mod users;

use sqlx::PgPool;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn create(pool: PgPool) -> Self {
        Self { pool }
    }
}
