use crate::{
    error::AppError,
    models::{CreateTodo, Todo, UpdateTodo},
};
use sqlx::PgPool;
use ulid::Ulid;

#[tracing::instrument(skip(pool))]
pub async fn list(pool: &PgPool) -> Result<Vec<Todo>, AppError> {
    let todos = sqlx::query_as::<_, Todo>("SELECT * FROM todos")
        .fetch_all(pool)
        .await?;
    Ok(todos)
}

#[tracing::instrument(skip(pool, payload), fields(todo.slug = %slug))]
pub async fn create(pool: &PgPool, slug: &str, payload: CreateTodo) -> Result<Todo, AppError> {
    let id = Ulid::new().to_string();
    sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (id, slug, title, description) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(&id)
    .bind(slug)
    .bind(&payload.title)
    .bind(&payload.description)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

#[tracing::instrument(skip(pool, update_todo), fields(todo.id = %id, todo.slug = %slug))]
pub async fn update(
    pool: &PgPool,
    id: &str,
    slug: &str,
    update_todo: UpdateTodo,
) -> Result<Option<Todo>, AppError> {
    let todo = sqlx::query_as::<_, Todo>(
        "UPDATE todos SET slug = $1, title = $2, description = $3, completed = $4 WHERE id = $5 RETURNING *",
    )
    .bind(slug)
    .bind(&update_todo.title)
    .bind(&update_todo.description)
    .bind(update_todo.completed)
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(todo)
}

#[tracing::instrument(skip(pool), fields(todo.id = %id))]
pub async fn get_by_id(pool: &PgPool, id: &str) -> Result<Option<Todo>, AppError> {
    let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(todo)
}

#[tracing::instrument(skip(pool), fields(todo.slug = %slug))]
pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Todo>, AppError> {
    let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await?;
    Ok(todo)
}

#[tracing::instrument(skip(pool), fields(todo.id = %id))]
pub async fn delete(pool: &PgPool, id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
