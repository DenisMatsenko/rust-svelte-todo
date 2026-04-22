use std::convert::Infallible;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::HeaderMap;
use axum::http::request::Parts;

use crate::{
    db::DatabaseService,
    error::AppError,
    models::{Claims, User},
};

/// Owns the JWT secret and a Database handle.
/// All authentication logic lives here — token encoding, decoding, user lookup.
#[derive(Clone)]
pub struct AuthService {
    db: DatabaseService,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(db: DatabaseService, jwt_secret: String) -> Self {
        Self { db, jwt_secret }
    }

    pub fn encode_jwt(&self, id: &str) -> Result<String, AppError> {
        let exp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AppError::Internal(e.to_string()))?
            .as_secs()
            .saturating_add(60 * 60 * 24 * 7); // 7 days

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &Claims {
                id: id.to_owned(),
                exp,
            },
            &jsonwebtoken::EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )?;
        Ok(token)
    }

    fn decode_jwt(&self, token: &str) -> Option<Claims> {
        let mut validation = jsonwebtoken::Validation::default();
        validation.required_spec_claims = std::collections::HashSet::new();

        jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .ok()
    }

    #[tracing::instrument(skip_all)]
    pub async fn try_authenticate(&self, headers: &HeaderMap) -> Option<User> {
        let token = headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .and_then(|v| v.split(',').next())
            .map(str::trim)
            .filter(|s| !s.is_empty())?;

        let claims = self.decode_jwt(token).or_else(|| {
            tracing::warn!("jwt decode failed");
            None
        })?;

        match self.db.get_user_by_id(&claims.id).await {
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
}

/// Extractor for routes that work for both anonymous and authenticated users.
/// Always succeeds — use `session.0` to get `Option<User>`.
#[derive(Debug, Clone)]
pub struct OptionalAuthSession(pub Option<User>);

impl<S> FromRequestParts<S> for OptionalAuthSession
where
    AuthService: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = AuthService::from_ref(state);
        Ok(Self(auth.try_authenticate(&parts.headers).await))
    }
}
