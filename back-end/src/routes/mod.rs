pub mod auth;
pub mod billing;
pub mod finance_plan;
pub mod tenant_notes;
pub mod tenants;
pub mod users;

use axum::{
    Router,
    extract::FromRef,
    http::{HeaderValue, Method, header},
};
use tower_http::{cors::{AllowOrigin, CorsLayer}, trace::TraceLayer};
use utoipa::{
    OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::{auth::AuthService, db::DatabaseService, mongo::MongoService};

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Auth"),
        (name = "Users"),
        (name = "Tenants", description = "Multi-tenant registry and per-tenant fan counts (Atlas / MongoDB)"),
        (name = "Finance Plan", description = "Admin finance plan entries used to build the income graph"),
        (name = "Billing", description = "Admin billing entries defining plan prices per fan count threshold"),
        (name = "Tenant Notes", description = "Admin notes attached to individual tenants"),
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
/// `DatabaseService`, `AuthService`, and `MongoService` are each individually
/// extractable via `FromRef`.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseService,
    pub auth: AuthService,
    pub mongo: MongoService,
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

impl FromRef<AppState> for MongoService {
    fn from_ref(state: &AppState) -> Self {
        state.mongo.clone()
    }
}

pub fn build_router(db: DatabaseService, auth: AuthService, mongo: MongoService, cors_origins: Vec<String>) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(auth::signin))
        .routes(routes!(auth::me))
        .routes(routes!(auth::signout))
        .routes(routes!(users::list_users, users::create_user))
        .routes(routes!(
            users::get_user,
            users::update_user,
            users::delete_user
        ))
        .routes(routes!(tenants::list_tenants))
        .routes(routes!(tenants::get_tenant))
        .routes(routes!(tenants::get_tenant_fans_count))
        .routes(routes!(tenants::get_tenant_stats))
        .routes(routes!(
            finance_plan::list_finance_plan_entries,
            finance_plan::create_finance_plan_entry
        ))
        .routes(routes!(
            finance_plan::get_finance_plan_entry,
            finance_plan::update_finance_plan_entry,
            finance_plan::delete_finance_plan_entry
        ))
        .routes(routes!(
            billing::list_billing_entries,
            billing::create_billing_entry
        ))
        .routes(routes!(
            billing::get_billing_entry,
            billing::update_billing_entry,
            billing::delete_billing_entry
        ))
        .routes(routes!(
            tenant_notes::get_tenant_note,
            tenant_notes::upsert_tenant_note,
            tenant_notes::delete_tenant_note
        ))
        .with_state(AppState { db, auth, mongo })
        .split_for_parts();

    let origins: Vec<HeaderValue> = cors_origins
        .iter()
        .map(|o| o.parse::<HeaderValue>().expect("valid CORS origin"))
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
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
