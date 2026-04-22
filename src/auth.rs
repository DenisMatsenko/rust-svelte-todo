use std::convert::Infallible;

use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::request::Parts;

use crate::{db::Database, error::AppError, models::{Claims, User}};

/// Extractor for routes that work for both anonymous and authenticated users.
/// Always succeeds — use `session.0` to get `Option<User>`.
#[derive(Debug, Clone)]
pub struct OptionalAuthSession(pub Option<User>);

impl FromRequestParts<Database> for OptionalAuthSession {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Database,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(try_authenticate(state, &parts.headers).await))
    }
}

#[tracing::instrument(skip_all)]
pub async fn try_authenticate(db: &Database, headers: &HeaderMap) -> Option<User> {
    let Some(token) = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        tracing::debug!("missing or malformed Authorization header");
        return None;
    };

    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());
    let mut validation = jsonwebtoken::Validation::default();
    validation.required_spec_claims = std::collections::HashSet::new();

    let claims = match jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => data.claims,
        Err(err) => {
            tracing::warn!(error = %err, "jwt decode failed");
            return None;
        }
    };

    match db.get_user_by_id(&claims.id).await {
        Ok(Some(user)) => Some(user),
        Ok(None) => {
            tracing::warn!(user_id = %claims.id, "jwt references unknown user");
            None
        }
        Err(err) => {
            tracing::error!(error = %err, "db error during authentication");
            None
        }
    }
}

pub fn encode_jwt(id: &str) -> Result<String, AppError> {
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .as_secs()
        .saturating_add(60 * 60 * 24 * 7); // 7 days

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &Claims { id: id.to_owned(), exp },
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;
    Ok(token)
}
