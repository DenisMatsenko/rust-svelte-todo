use crate::{
    error::AppError,
    models::{CreateTodo, Todo, UpdateTodo},
};
use ulid::Ulid;

use super::Database;

impl Database {
    pub async fn list_todos(&self) -> Result<Vec<Todo>, AppError> {
        let todos = sqlx::query_as!(Todo, "SELECT * FROM todos")
            .fetch_all(&self.pool)
            .await?;
        Ok(todos)
    }

    pub async fn create_todo(&self, slug: &str, payload: CreateTodo) -> Result<Todo, AppError> {
        let id = Ulid::new().to_string();
        sqlx::query_as!(
            Todo,
            "INSERT INTO todos (id, slug, title, description) VALUES ($1, $2, $3, $4) RETURNING *",
            id,
            slug,
            payload.title,
            payload.description,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::from)
    }

    pub async fn update_todo(
        &self,
        id: &str,
        slug: &str,
        update_todo: UpdateTodo,
    ) -> Result<Option<Todo>, AppError> {
        let todo = sqlx::query_as!(
            Todo,
            "UPDATE todos SET slug = $1, title = $2, description = $3, completed = $4 WHERE id = $5 RETURNING *",
            slug,
            update_todo.title,
            update_todo.description,
            update_todo.completed,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(todo)
    }

    pub async fn get_todo_by_id(&self, id: &str) -> Result<Option<Todo>, AppError> {
        let todo = sqlx::query_as!(Todo, "SELECT * FROM todos WHERE id = $1", id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(todo)
    }

    pub async fn get_todo_by_slug(&self, slug: &str) -> Result<Option<Todo>, AppError> {
        let todo = sqlx::query_as!(Todo, "SELECT * FROM todos WHERE slug = $1", slug)
            .fetch_optional(&self.pool)
            .await?;
        Ok(todo)
    }

    pub async fn delete_todo(&self, id: &str) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM todos WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
