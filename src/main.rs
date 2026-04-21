use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use slug::slugify;
use sqlx::PgPool;
use thiserror::Error;
use ulid::Ulid;
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, Deserialize, ToSchema, Clone, sqlx::FromRow)]
struct Todo {
    id: String,
    slug: String,
    title: String,
    description: String,
    completed: bool,
}

#[derive(Deserialize, ToSchema)]
struct CreateTodo {
    title: String,
    description: String,
}

#[derive(Deserialize, ToSchema)]
struct UpdateTodo {
    title: String,
    description: String,
    completed: bool,
}

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

impl ErrorResponse {
    fn new(msg: impl Into<String>) -> Json<Self> {
        Json(Self { error: msg.into() })
    }
}

#[derive(Debug, Error)]
enum AppError {
    #[error("not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(err: argon2::password_hash::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound => {
                (StatusCode::NOT_FOUND, ErrorResponse::new("not found")).into_response()
            }
            Self::Unauthorized => {
                (StatusCode::UNAUTHORIZED, ErrorResponse::new("unauthorized")).into_response()
            }
            Self::Conflict(msg) => (StatusCode::CONFLICT, ErrorResponse::new(msg)).into_response(),
            Self::Internal(msg) => {
                eprintln!("internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::new("internal server error"),
                )
                    .into_response()
            }
        }
    }
}

#[derive(OpenApi)]
#[openapi(tags((name = "todos", description = "Todo management")))]
struct ApiDoc;

type AppState = PgPool;

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(signup))
        .routes(routes!(signin))
        .routes(routes!(me))
        .routes(routes!(list_todos, create_todo))
        .routes(routes!(get_todo, update_todo))
        .with_state(pool)
        .split_for_parts();

    let app = router
        .merge(SwaggerUi::new("/swagger").url("/openapi.json", api.clone()))
        .merge(Scalar::with_url("/scalar", api.clone()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    println!("Swagger UI at http://localhost:3000/swagger/");
    println!("Scalar UI at http://localhost:3000/scalar/");
    println!("OpenAPI JSON at http://localhost:3000/openapi.json");
    axum::serve(listener, app).await.unwrap();
}

#[utoipa::path(
    get,
    path = "/todos",
    responses((status = 200, description = "List of todos", body = Vec<Todo>)),
    tag = "todos"
)]
async fn list_todos(
    headers: HeaderMap,
    State(pool): State<AppState>,
) -> Result<Json<Vec<Todo>>, AppError> {
    try_authenticate(&pool, &headers).await.ok_or(AppError::Unauthorized)?;
    let todos = sqlx::query_as::<_, Todo>("SELECT * FROM todos")
        .fetch_all(&pool)
        .await?;
    Ok(Json(todos))
}

#[utoipa::path(
    post,
    path = "/todos",
    request_body = CreateTodo,
    responses(
        (status = 201, description = "Todo created", body = Todo),
        (status = 409, description = "Slug already exists", body = ErrorResponse),
    ),
    tag = "todos"
)]
async fn create_todo(
    headers: HeaderMap,
    State(pool): State<AppState>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let user = try_authenticate(&pool, &headers).await.ok_or(AppError::Unauthorized)?;
    println!("Creating todo for user: {}", user.id);

    let id = Ulid::new().to_string();
    let mut slug = slugify(&payload.title);

    for attempt in 0..3 {
        if get_todo_by_slug(&pool, &slug).await?.is_some() {
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

    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (id, slug, title, description) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(&id)
    .bind(&slug)
    .bind(&payload.title)
    .bind(&payload.description)
    .fetch_one(&pool)
    .await?;
    Ok((StatusCode::CREATED, Json(todo)))
}

#[utoipa::path(
    get,
    path = "/todos/{id}",
    responses(
        (status = 200, description = "Todo found", body = Todo),
        (status = 404, description = "Todo not found", body = ErrorResponse),
    ),
    tag = "todos"
)]
async fn get_todo(
    headers: HeaderMap,
    State(pool): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Todo>, AppError> {
    try_authenticate(&pool, &headers).await.ok_or(AppError::Unauthorized)?;
    let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(todo))
}

#[utoipa::path(
    put,
    path = "/todos/{id}",
    request_body = UpdateTodo,
    responses(
        (status = 200, description = "Todo updated", body = Todo),
        (status = 404, description = "Todo not found", body = ErrorResponse),
        (status = 409, description = "Slug already exists", body = ErrorResponse),
    ),
    tag = "todos"
)]
async fn update_todo(
    headers: HeaderMap,
    State(pool): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, AppError> {
    try_authenticate(&pool, &headers).await.ok_or(AppError::Unauthorized)?;
    let mut slug = slugify(&payload.title);

    for attempt in 0..3 {
        if get_todo_by_slug(&pool, &slug).await?.is_some() {
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

    let todo = sqlx::query_as::<_, Todo>(
        "UPDATE todos SET slug = $1, title = $2, description = $3, completed = $4 WHERE id = $5 RETURNING *",
    )
    .bind(&slug)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(payload.completed)
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(todo))
}

async fn get_todo_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Todo>, AppError> {
    let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await?;
    Ok(todo)
}

// ── Auth ─────────────────────────────────────────────────────────────────────

#[derive(Serialize, ToSchema)]
struct Token {
    token: String,
}

#[derive(Deserialize, ToSchema)]
struct CreateUser {
    full_name: String,
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    id: String,
}

#[derive(Serialize, ToSchema, Clone, sqlx::FromRow)]
struct User {
    id: String,
    slug: String,
    full_name: String,
    email: String,
}

#[derive(sqlx::FromRow)]
struct DBUser {
    id: String,
    slug: String,
    full_name: String,
    email: String,
    password: String,
}

async fn try_authenticate(pool: &PgPool, headers: &HeaderMap) -> Option<User> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))?;

    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());

    let mut validation = jsonwebtoken::Validation::default();
    validation.validate_exp = false;
    validation.required_spec_claims = std::collections::HashSet::new();

    let claims = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )
    .ok()?
    .claims;

    sqlx::query_as::<_, User>("SELECT id, slug, full_name, email FROM users WHERE id = $1")
        .bind(&claims.id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

fn encode_jwt(id: &str) -> Result<String, AppError> {
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &Claims { id: id.to_owned() },
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;
    Ok(token)
}

#[utoipa::path(
    post,
    path = "/auth/signup",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User created", body = Token),
        (status = 409, description = "Email already exists", body = ErrorResponse),
    ),
    tag = "users"
)]
async fn signup(
    State(pool): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<Token>), AppError> {
    if get_user_by_email(&pool, &payload.email).await?.is_some() {
        return Err(AppError::Conflict("email already in use".to_owned()));
    }

    let mut slug = slugify(&payload.full_name);
    for attempt in 0..3 {
        if get_user_by_slug(&pool, &slug).await?.is_some() {
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

    let id = Ulid::new().to_string();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)?
        .to_string();

    sqlx::query(
        "INSERT INTO users (id, slug, full_name, email, password) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&id)
    .bind(&slug)
    .bind(&payload.full_name)
    .bind(&payload.email)
    .bind(password_hash)
    .execute(&pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(Token {
            token: encode_jwt(&id)?,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/signin",
    request_body = CreateUser,
    responses(
        (status = 200, description = "User signed in", body = Token),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
    ),
    tag = "users"
)]
async fn signin(
    State(pool): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<Token>, AppError> {
    let user = get_user_by_email(&pool, &payload.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let parsed_hash = PasswordHash::new(&user.password)?;
    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(AppError::Unauthorized);
    }

    Ok(Json(Token {
        token: encode_jwt(&user.id)?,
    }))
}

#[utoipa::path(
    get,
    path = "/users/me",
    responses(
        (status = 200, description = "Current user info", body = User),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "users"
)]
async fn me(
    headers: HeaderMap,
    State(pool): State<AppState>,
) -> Result<Json<User>, AppError> {
    let user = try_authenticate(&pool, &headers).await.ok_or(AppError::Unauthorized)?;
    Ok(Json(user))
}

async fn get_user_by_slug(pool: &PgPool, slug: &str) -> Result<Option<DBUser>, AppError> {
    let user = sqlx::query_as::<_, DBUser>("SELECT * FROM users WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<DBUser>, AppError> {
    let user = sqlx::query_as::<_, DBUser>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}
