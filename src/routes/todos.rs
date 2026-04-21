use crate::{
    auth::try_authenticate,
    db,
    error::AppError,
    models::{CreateTodo, Todo, UpdateTodo},
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use slug::slugify;
use sqlx::PgPool;
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
pub async fn list_todos(
    headers: HeaderMap,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Todo>>, AppError> {
    try_authenticate(&pool, &headers)
        .await
        .ok_or(AppError::Unauthorized)?;

    let todos = db::todos::list(&pool).await?;

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
pub async fn create_todo(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    try_authenticate(&pool, &headers)
        .await
        .ok_or(AppError::Unauthorized)?;

    let mut slug = slugify(&payload.title);
    for attempt in 0..3 {
        if db::todos::get_by_slug(&pool, &slug).await?.is_some() {
            if attempt == 2 {
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

    let todo = db::todos::create(&pool, &slug, payload).await?;
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
pub async fn get_todo(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<Todo>, AppError> {
    try_authenticate(&pool, &headers)
        .await
        .ok_or(AppError::Unauthorized)?;

    let todo = db::todos::get_by_id(&pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

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
pub async fn update_todo(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, AppError> {
    try_authenticate(&pool, &headers)
        .await
        .ok_or(AppError::Unauthorized)?;
    let mut slug = slugify(&payload.title);

    for attempt in 0..3 {
        if db::todos::get_by_slug(&pool, &slug).await?.is_some() {
            if attempt == 2 {
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

    let todo = db::todos::update(&pool, &id, &slug, payload)
        .await?
        .ok_or(AppError::NotFound)?;

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
        (status = 409, description = "Slug already exists", body = crate::error::ErrorResponse),
    ),
    tag = "todos"
)]
pub async fn delete_todo(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<(), AppError> {
    try_authenticate(&pool, &headers)
        .await
        .ok_or(AppError::Unauthorized)?;

    db::todos::delete(&pool, &id).await?;

    Ok(())
}
