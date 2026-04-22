pub mod auth;
pub mod todos;
pub mod users;

use axum::{
    Router,
    extract::FromRef,
    http::{HeaderValue, Method, header},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::{
    OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::{auth::AuthService, db::DatabaseService};

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Todos"),
        (name = "Auth"),
        (name = "Users"),
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

/// Shared axum application state.
/// Both `Database` and `AuthService` are extractable individually via `FromRef`.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseService,
    pub auth: AuthService,
}

impl FromRef<AppState> for DatabaseService {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl FromRef<AppState> for AuthService {
    fn from_ref(state: &AppState) -> Self {
        state.auth.clone()
    }
}

pub fn build_router(db: DatabaseService, auth: AuthService) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(auth::signin))
        .routes(routes!(auth::me))
        .routes(routes!(auth::signout))
        .routes(routes!(todos::list_todos, todos::create_todo))
        .routes(routes!(
            todos::get_todo,
            todos::update_todo,
            todos::delete_todo
        ))
        .routes(routes!(users::list_users, users::create_user))
        .routes(routes!(
            users::get_user,
            users::update_user,
            users::delete_user
        ))
        .with_state(AppState { db, auth })
        .split_for_parts();

    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:3001"
                .parse::<HeaderValue>()
                .expect("valid origin"),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true);

    router
        .merge(SwaggerUi::new("/swagger").url("/openapi.json", api.clone()))
        .merge(Scalar::with_url("/scalar", api))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
