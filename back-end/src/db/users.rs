use crate::{
    error::AppError,
    models::{DBUser, User, UserRole},
};

use super::DatabaseService;

impl DatabaseService {
    pub async fn list_users(&self) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as!(
            User,
            r#"SELECT id, slug, full_name, email, role AS "role: UserRole" FROM users"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, slug, full_name, email, role AS "role: UserRole" FROM users WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn get_user_by_slug(&self, slug: &str) -> Result<Option<DBUser>, AppError> {
        let user = sqlx::query_as!(
            DBUser,
            r#"SELECT id, slug, full_name, email, password, role AS "role: UserRole" FROM users WHERE slug = $1"#,
            slug,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<DBUser>, AppError> {
        let user = sqlx::query_as!(
            DBUser,
            r#"SELECT id, slug, full_name, email, password, role AS "role: UserRole" FROM users WHERE email = $1"#,
            email,
        )
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
        role: UserRole,
    ) -> Result<User, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"INSERT INTO users (id, slug, full_name, email, password, role) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, slug, full_name, email, role AS "role: UserRole""#,
            id,
            slug,
            full_name,
            email,
            password_hash,
            role as UserRole,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn update_user(
        &self,
        id: &str,
        slug: &str,
        full_name: &str,
        role: UserRole,
    ) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"UPDATE users SET slug = $2, full_name = $3, role = $4 WHERE id = $1 RETURNING id, slug, full_name, email, role AS "role: UserRole""#,
            id,
            slug,
            full_name,
            role as UserRole,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn get_user_by_slug_excluding_id(
        &self,
        slug: &str,
        id: &str,
    ) -> Result<Option<DBUser>, AppError> {
        let user = sqlx::query_as!(
            DBUser,
            r#"SELECT id, slug, full_name, email, password, role AS "role: UserRole" FROM users WHERE slug = $1 AND id != $2"#,
            slug,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn delete_user(&self, id: &str) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM users WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
