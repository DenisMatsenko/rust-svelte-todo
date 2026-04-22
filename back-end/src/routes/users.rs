use crate::{
    auth::OptionalAuthSession,
    db::DatabaseService,
    error::AppError,
    models::{CreateUser, UpdateUser, User},
};
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use slug::slugify;
use ulid::Ulid;

/// List all users
///
/// Returns all users in the database.
///
/// The request must include a valid Bearer token in the Authorization header for authentication
/// (use the `/auth/signup` or `/auth/signin` endpoint to obtain a token).
#[utoipa::path(
    get,
    path = "/users",
    params(
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    tag = "Users"
)]
#[tracing::instrument(skip_all)]
pub async fn list_users(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
) -> Result<Json<Vec<User>>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let users = db.list_users().await?;
    tracing::debug!(count = users.len(), "listed users");
    Ok(Json(users))
}

/// Create a new user
///
/// Creates a new user with a unique slug generated from the full name.
///
/// If a slug collision occurs, it will attempt to generate a new slug up to 3 times
/// before returning a conflict error.
///
/// The request must include a valid Bearer token in the Authorization header for authentication
/// (use the `/auth/signup` or `/auth/signin` endpoint to obtain a token).
#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUser,
    params(
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "User created", body = User),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 409, description = "Slug already exists", body = crate::error::ErrorResponse),
    ),
    tag = "Users"
)]
#[tracing::instrument(skip_all)]
pub async fn create_user(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let id = Ulid::new().to_string();

    let mut slug = slugify(&payload.full_name);
    for attempt in 0..3 {
        if db.get_user_by_slug(&slug).await?.is_some() {
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

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)?
        .to_string();

    let user = db
        .create_user(
            &id,
            &slug,
            &payload.full_name,
            &payload.email,
            &password_hash,
            payload.role,
        )
        .await?;
    tracing::info!(user.id = %user.id, user.slug = %user.slug, "user created");
    Ok((StatusCode::CREATED, Json(user)))
}

/// Get a user by ID
///
/// Returns a single user identified by its ID.
///
/// The request must include a valid Bearer token in the Authorization header for authentication
/// (use the `/auth/signup` or `/auth/signin` endpoint to obtain a token).
#[utoipa::path(
    get,
    path = "/users/{id}",
    params(
        ("id" = String, Path, description = "The user ID"),
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "User not found", body = crate::error::ErrorResponse),
    ),
    tag = "Users"
)]
#[tracing::instrument(skip_all, fields(user.id = %id))]
pub async fn get_user(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    Path(id): Path<String>,
) -> Result<Json<User>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let user = db.get_user_by_id(&id).await?.ok_or_else(|| {
        tracing::warn!(user.id = %id, "user not found");
        AppError::NotFound
    })?;

    Ok(Json(user))
}

/// Update a user
///
/// Updates the full name, email, and role of an existing user.
/// The slug is regenerated from the new full name. If a slug collision occurs, it will
/// attempt to generate a new slug up to 3 times before returning a conflict error.
///
/// The request must include a valid Bearer token in the Authorization header for authentication
/// (use the `/auth/signup` or `/auth/signin` endpoint to obtain a token).
#[utoipa::path(
    put,
    path = "/users/{id}",
    request_body = UpdateUser,
    params(
        ("id" = String, Path, description = "The user ID"),
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "User updated", body = User),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "User not found", body = crate::error::ErrorResponse),
        (status = 409, description = "Slug already exists", body = crate::error::ErrorResponse),
    ),
    tag = "Users"
)]
#[tracing::instrument(skip_all, fields(user.id = %id))]
pub async fn update_user(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUser>,
) -> Result<Json<User>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let mut slug = slugify(&payload.full_name);
    for attempt in 0..3 {
        if db
            .get_user_by_slug_excluding_id(&slug, &id)
            .await?
            .is_some()
        {
            if attempt == 2 {
                tracing::warn!(user.id = %id, slug = %slug, "slug collision after 3 attempts");
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

    let user = db
        .update_user(&id, &slug, &payload.full_name, payload.role)
        .await?
        .ok_or_else(|| {
            tracing::warn!(user.id = %id, "user not found for update");
            AppError::NotFound
        })?;

    tracing::info!(user.id = %user.id, user.slug = %user.slug, "user updated");
    Ok(Json(user))
}

/// Delete user
///
/// Deletes a user identified by its ID. Returns 204 No Content on success.
///
/// The request must include a valid Bearer token in the Authorization header for authentication
/// (use the `/auth/signup` or `/auth/signin` endpoint to obtain a token).
#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(
        ("id" = String, Path, description = "The user ID"),
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 204, description = "User deleted"),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "User not found", body = crate::error::ErrorResponse),
    ),
    tag = "Users"
)]
#[tracing::instrument(skip_all, fields(user.id = %id))]
pub async fn delete_user(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    if db.get_user_by_id(&id).await?.is_none() {
        tracing::warn!(user.id = %id, "user not found for delete");
        return Err(AppError::NotFound);
    }

    db.delete_user(&id).await?;
    tracing::info!(user.id = %id, "user deleted");
    Ok(StatusCode::NO_CONTENT)
}
