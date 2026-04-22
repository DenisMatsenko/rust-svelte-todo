use crate::{
    auth::{encode_jwt, OptionalAuthSession},
    db::Database,
    error::AppError,
    models::{CreateUser, Token, User},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
};
use slug::slugify;
use ulid::Ulid;

/// Sign up
///
/// Creates a new user account and returns a JWT token.
///
/// A unique slug is generated from the user's full name. If a slug collision occurs,
/// it will attempt to generate a new slug up to 3 times before returning a conflict error.
#[utoipa::path(
    post,
    path = "/auth/signup",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User created, returns JWT token", body = Token),
        (status = 409, description = "Email already in use", body = crate::error::ErrorResponse),
    ),
    tag = "auth"
)]
#[tracing::instrument(skip_all)]
pub async fn signup(
    State(db): State<Database>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<Token>), AppError> {
    if db.get_user_by_email(&payload.email).await?.is_some() {
        tracing::warn!(email = %payload.email, "signup with already-used email");
        return Err(AppError::Conflict("email already in use".to_owned()));
    }

    let mut slug = slugify(&payload.full_name);
    for attempt in 0..3 {
        if db.get_user_by_slug(&slug).await?.is_some() {
            if attempt == 2 {
                tracing::warn!(slug = %slug, "slug collision after 3 attempts during signup");
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

    let id = Ulid::new().to_string();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)?
        .to_string();

    db.create_user(&id, &slug, &payload.full_name, &payload.email, &password_hash)
        .await?;

    tracing::info!(user.id = %id, user.email = %payload.email, "user signed up");
    Ok((StatusCode::CREATED, Json(Token { token: encode_jwt(&id)? })))
}

/// Sign in
///
/// Authenticates an existing user with their email and password, and returns a JWT token.
///
/// The token can be used in the `Authorization: Bearer <token>` header to access protected endpoints.
#[utoipa::path(
    post,
    path = "/auth/signin",
    request_body = CreateUser,
    responses(
        (status = 200, description = "Authenticated, returns JWT token", body = Token),
        (status = 401, description = "Invalid email or password", body = crate::error::ErrorResponse),
    ),
    tag = "auth"
)]
#[tracing::instrument(skip_all)]
pub async fn signin(
    State(db): State<Database>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<Token>, AppError> {
    let user = db.get_user_by_email(&payload.email).await?.ok_or_else(|| {
        tracing::warn!(email = %payload.email, "signin attempt for unknown email");
        AppError::Unauthorized
    })?;

    let parsed_hash = PasswordHash::new(&user.password)?;
    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        tracing::warn!(user.id = %user.id, "invalid password on signin");
        return Err(AppError::Unauthorized);
    }

    tracing::info!(user.id = %user.id, "user signed in");
    Ok(Json(Token {
        token: encode_jwt(&user.id)?,
    }))
}

/// Get current user
///
/// Returns the profile of the currently authenticated user.
///
/// The request must include a valid Bearer token in the Authorization header for authentication
/// (use the `/auth/signup` or `/auth/signin` endpoint to obtain a token).
#[utoipa::path(
    get,
    path = "/users/me",
    params(
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Current user profile", body = User),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    tag = "users"
)]
#[tracing::instrument(skip_all)]
pub async fn me(OptionalAuthSession(user): OptionalAuthSession) -> Result<Json<User>, AppError> {
    let user = user.ok_or(AppError::Unauthorized)?;
    tracing::debug!(user.id = %user.id, "fetched current user");
    Ok(Json(user))
}
