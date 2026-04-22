use crate::{
    auth::OptionalAuthSession,
    db::Database,
    error::AppError,
    models::{CreateTodo, Todo, UpdateTodo},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use slug::slugify;
use ulid::Ulid;

/// List all todos
///
/// Returns all todos in the database.
///
/// The request must include a valid Bearer token in the Authorization header for authentication
/// (use the `/auth/signup` or `/auth/signin` endpoint to obtain a token).
#[utoipa::path(
    get,
    path = "/todos",
    params(
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "List of todos", body = Vec<Todo>),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    tag = "todos"
)]
#[tracing::instrument(skip_all)]
pub async fn list_todos(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<Database>,
) -> Result<Json<Vec<Todo>>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let todos = db.list_todos().await?;
    tracing::debug!(count = todos.len(), "listed todos");
    Ok(Json(todos))
}

/// Create a new todo
///
/// Creates a new todo with a unique slug generated from the title.
///
/// If a slug collision occurs, it will attempt to generate a new slug up to 3 times
/// before returning a conflict error.
///
/// The request must include a valid Bearer token in the Authorization header for authentication
/// (use the `/auth/signup` or `/auth/signin` endpoint to obtain a token).
#[utoipa::path(
    post,
    path = "/todos",
    request_body = CreateTodo,
    params(
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Todo created", body = Todo),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 409, description = "Slug already exists", body = crate::error::ErrorResponse),
    ),
    tag = "todos"
)]
#[tracing::instrument(skip_all)]
pub async fn create_todo(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<Database>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let mut slug = slugify(&payload.title);
    for attempt in 0..3 {
        if db.get_todo_by_slug(&slug).await?.is_some() {
            if attempt == 2 {
                tracing::warn!(slug = %slug, "slug collision after 3 attempts");
                return Err(AppError::Conflict(
                    "failed to generate unique slug after 3 attempts".to_owned(),
                ));
            }
            let suffix = &Ulid::new().to_string()[20..];
            slug = format!("{slug}-{suffix}");
        } else {
            break;
        }
    }

    let todo = db.create_todo(&slug, payload).await?;
    tracing::info!(todo.id = %todo.id, todo.slug = %todo.slug, "todo created");
    Ok((StatusCode::CREATED, Json(todo)))
}

/// Get a todo by ID
///
/// Returns a single todo identified by its ID.
///
/// The request must include a valid Bearer token in the Authorization header for authentication
/// (use the `/auth/signup` or `/auth/signin` endpoint to obtain a token).
#[utoipa::path(
    get,
    path = "/todos/{id}",
    params(
        ("id" = String, Path, description = "The todo ID"),
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Todo found", body = Todo),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Todo not found", body = crate::error::ErrorResponse),
    ),
    tag = "todos"
)]
#[tracing::instrument(skip_all, fields(todo.id = %id))]
pub async fn get_todo(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<Database>,
    Path(id): Path<String>,
) -> Result<Json<Todo>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let todo = db.get_todo_by_id(&id).await?.ok_or_else(|| {
        tracing::warn!(todo.id = %id, "todo not found");
        AppError::NotFound
    })?;

    Ok(Json(todo))
}

/// Update a todo
///
/// Updates the title, description, and completed status of an existing todo.
/// The slug is regenerated from the new title. If a slug collision occurs, it will
/// attempt to generate a new slug up to 3 times before returning a conflict error.
///
/// The request must include a valid Bearer token in the Authorization header for authentication
/// (use the `/auth/signup` or `/auth/signin` endpoint to obtain a token).
#[utoipa::path(
    put,
    path = "/todos/{id}",
    request_body = UpdateTodo,
    params(
        ("id" = String, Path, description = "The todo ID"),
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Todo updated", body = Todo),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Todo not found", body = crate::error::ErrorResponse),
        (status = 409, description = "Slug already exists", body = crate::error::ErrorResponse),
    ),
    tag = "todos"
)]
#[tracing::instrument(skip_all, fields(todo.id = %id))]
pub async fn update_todo(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<Database>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let mut slug = slugify(&payload.title);
    for attempt in 0..3 {
        if db.get_todo_by_slug(&slug).await?.is_some() {
            if attempt == 2 {
                tracing::warn!(todo.id = %id, slug = %slug, "slug collision after 3 attempts");
                return Err(AppError::Conflict(
                    "failed to generate unique slug after 3 attempts".to_owned(),
                ));
            }
            let suffix = &Ulid::new().to_string()[20..];
            slug = format!("{slug}-{suffix}");
        } else {
            break;
        }
    }

    let todo = db
        .update_todo(&id, &slug, payload)
        .await?
        .ok_or_else(|| {
            tracing::warn!(todo.id = %id, "todo not found for update");
            AppError::NotFound
        })?;

    tracing::info!(todo.id = %todo.id, todo.slug = %todo.slug, "todo updated");
    Ok(Json(todo))
}

/// Delete todo
///
/// Deletes a todo identified by its ID. Returns 204 No Content on success.
///
/// The request must include a valid Bearer token in the Authorization header for authentication
/// (use the `/auth/signup` or `/auth/signin` endpoint to obtain a token).
#[utoipa::path(
    delete,
    path = "/todos/{id}",
    params(
        ("id" = String, Path, description = "The todo ID"),
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 204, description = "Todo deleted"),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Todo not found", body = crate::error::ErrorResponse),
    ),
    tag = "todos"
)]
#[tracing::instrument(skip_all, fields(todo.id = %id))]
pub async fn delete_todo(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<Database>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    if db.get_todo_by_id(&id).await?.is_none() {
        tracing::warn!(todo.id = %id, "todo not found for delete");
        return Err(AppError::NotFound);
    }

    db.delete_todo(&id).await?;
    tracing::info!(todo.id = %id, "todo deleted");
    Ok(StatusCode::NO_CONTENT)
}
