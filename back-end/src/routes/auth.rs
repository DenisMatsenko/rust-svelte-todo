use crate::{
    auth::{AuthService, OptionalAuthSession, clear_auth_cookie, make_auth_cookie},
    db::DatabaseService,
    error::AppError,
    models::{SigninUser, Token, User},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::CookieJar;

/// Sign in
///
/// Authenticates an existing user and sets an `auth_token` HttpOnly cookie.
///
/// The cookie is used automatically for subsequent requests. The token is also returned
/// in the response body for clients that prefer the `Authorization: Bearer` header.
#[utoipa::path(
    post,
    path = "/auth/signin",
    request_body = SigninUser,
    responses(
        (status = 200, description = "Authenticated, returns JWT token", body = Token),
        (status = 401, description = "Invalid email or password", body = crate::error::ErrorResponse),
    ),
    tag = "Auth"
)]
#[tracing::instrument(skip_all)]
pub async fn signin(
    State(auth): State<AuthService>,
    State(db): State<DatabaseService>,
    jar: CookieJar,
    Json(payload): Json<SigninUser>,
) -> Result<(CookieJar, Json<Token>), AppError> {
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
    let token = auth.encode_jwt(&user.id)?;
    let jar = jar.add(make_auth_cookie(token.clone()));
    Ok((jar, Json(Token { token })))
}

/// Get current user
///
/// Returns the profile of the currently authenticated user.
///
/// Authentication is accepted via `auth_token` cookie (preferred) or
/// `Authorization: Bearer <token>` header.
#[utoipa::path(
    get,
    path = "/auth/me",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Current user profile", body = User),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    tag = "Auth"
)]
#[tracing::instrument(skip_all)]
pub async fn me(OptionalAuthSession(user): OptionalAuthSession) -> Result<Json<User>, AppError> {
    let user = user.ok_or(AppError::Unauthorized)?;
    tracing::debug!(user.id = %user.id, "fetched current user");
    Ok(Json(user))
}

/// Sign out
///
/// Clears the `auth_token` cookie.
#[utoipa::path(
    post,
    path = "/auth/signout",
    responses(
        (status = 204, description = "Signed out, cookie cleared"),
    ),
    tag = "Auth"
)]
#[tracing::instrument(skip_all)]
pub async fn signout(jar: CookieJar) -> (StatusCode, CookieJar) {
    let jar = jar.remove(clear_auth_cookie());
    tracing::info!("user signed out");
    (StatusCode::NO_CONTENT, jar)
}
