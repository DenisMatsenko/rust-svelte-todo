pub mod todos;
pub mod users;

use axum::Router;
use tower_http::trace::TraceLayer;
use utoipa::{
    OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::db::Database;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "auth", description = "Signup and signin"),
        (name = "todos", description = "Todo management"),
        (name = "users", description = "User management"),
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

pub fn build_router(db: Database) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(users::signup))
        .routes(routes!(users::signin))
        .routes(routes!(users::me))
        .routes(routes!(todos::list_todos, todos::create_todo))
        .routes(routes!(
            todos::get_todo,
            todos::update_todo,
            todos::delete_todo
        ))
        .with_state(db)
        .split_for_parts();

    router
        .merge(SwaggerUi::new("/swagger").url("/openapi.json", api.clone()))
        .merge(Scalar::with_url("/scalar", api))
        .layer(TraceLayer::new_for_http())
}
