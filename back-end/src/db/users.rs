use crate::{
    error::AppError,
    models::{DBUser, User},
};

use super::DatabaseService;

impl DatabaseService {
    pub async fn get_user_by_id(&self, id: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, slug, full_name, email FROM users WHERE id = $1",
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn get_user_by_slug(&self, slug: &str) -> Result<Option<DBUser>, AppError> {
        let user = sqlx::query_as!(DBUser, "SELECT * FROM users WHERE slug = $1", slug)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<DBUser>, AppError> {
        let user = sqlx::query_as!(DBUser, "SELECT * FROM users WHERE email = $1", email)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    pub async fn create_user(
        &self,
        id: &str,
        slug: &str,
        full_name: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO users (id, slug, full_name, email, password) VALUES ($1, $2, $3, $4, $5)",
            id,
            slug,
            full_name,
            email,
            password_hash,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
