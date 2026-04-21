use crate::{
    error::AppError,
    models::{DBUser, User},
};
use sqlx::PgPool;

#[tracing::instrument(skip(pool), fields(user.id = %id))]
pub async fn get_by_id(pool: &PgPool, id: &str) -> Result<Option<User>, AppError> {
    let user =
        sqlx::query_as::<_, User>("SELECT id, slug, full_name, email FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    Ok(user)
}

#[tracing::instrument(skip(pool), fields(user.slug = %slug))]
pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Option<DBUser>, AppError> {
    let user = sqlx::query_as::<_, DBUser>("SELECT * FROM users WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

#[tracing::instrument(skip(pool), fields(user.email = %email))]
pub async fn get_by_email(pool: &PgPool, email: &str) -> Result<Option<DBUser>, AppError> {
    let user = sqlx::query_as::<_, DBUser>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

#[tracing::instrument(skip(pool, password_hash), fields(user.id = %id, user.email = %email))]
pub async fn create(
    pool: &PgPool,
    id: &str,
    slug: &str,
    full_name: &str,
    email: &str,
    password_hash: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO users (id, slug, full_name, email, password) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(slug)
    .bind(full_name)
    .bind(email)
    .bind(password_hash)
    .execute(pool)
    .await?;
    Ok(())
}
