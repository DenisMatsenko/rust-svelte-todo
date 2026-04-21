use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone, sqlx::FromRow)]
pub struct Todo {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub completed: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTodo {
    pub title: String,
    pub description: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateTodo {
    pub title: String,
    pub description: String,
    pub completed: bool,
}

#[derive(Serialize, ToSchema)]
pub struct Token {
    pub token: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateUser {
    pub full_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub exp: u64,
}

#[derive(Serialize, ToSchema, Clone, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub slug: String,
    pub full_name: String,
    pub email: String,
}

#[derive(sqlx::FromRow)]
pub struct DBUser {
    pub id: String,
    pub slug: String,
    pub full_name: String,
    pub email: String,
    pub password: String,
}
