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
pub struct CreateUser {
    pub full_name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUser {
    pub full_name: String,
    pub role: UserRole,
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
pub struct SignupUser {
    pub full_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub exp: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Editor,
    Viewer,
}

#[derive(Debug, Serialize, ToSchema, Clone, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub slug: String,
    pub full_name: String,
    pub email: String,
    pub role: UserRole,
}

#[derive(sqlx::FromRow)]
pub struct DBUser {
    pub id: String,
    pub slug: String,
    pub full_name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
}
