use axum::Json;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
struct Todo {
    id: u64,
    title: String,
    completed: bool,
}

#[derive(Deserialize, ToSchema)]
struct CreateTodo {
    title: String,
    completed: bool,
}

#[derive(OpenApi)]
#[openapi(
    tags((name = "todos", description = "Todo management"))
)]
struct ApiDoc;

type AppState = Arc<Mutex<Vec<Todo>>>;

#[tokio::main]
async fn main() {
    let state: AppState = Arc::new(Mutex::new(Vec::new()));

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(list_todos, create_todo))
        .routes(routes!(get_todo, update_todo))
        .with_state(state)
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
    responses(
        (status = 200, description = "List of todos", body = Vec<Todo>)
    ),
    tag = "todos"
)]
async fn list_todos(State(state): State<AppState>) -> (StatusCode, Json<Vec<Todo>>) {
    let todos = state.lock().unwrap();
    (StatusCode::OK, Json(todos.clone()))
}

#[utoipa::path(
    post,
    path = "/todos",
    request_body = CreateTodo,
    responses(
        (status = 201, description = "Todo created", body = Todo)
    ),
    tag = "todos"
)]

async fn create_todo(
    State(state): State<AppState>,
    Json(payload): Json<CreateTodo>,
) -> (StatusCode, Json<Todo>) {
    let mut todos = state.lock().unwrap();
    let id = todos.last().map_or(1, |t| t.id + 1);
    let todo = Todo {
        id,
        title: payload.title,
        completed: payload.completed,
    };
    todos.push(todo.clone());
    (StatusCode::CREATED, Json(todo))
}

#[utoipa::path(
    get,
    path = "/todos/{id}",
    responses(
        (status = 200, description = "Todo found", body = Todo),
        (status = 404, description = "Todo not found")
    ),
    tag = "todos"
)]
async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> (StatusCode, Json<Option<Todo>>) {
    let todos = state.lock().unwrap();
    match todos.iter().find(|t| t.id == id) {
        Some(todo) => (StatusCode::OK, Json(Some(todo.clone()))),
        None => (StatusCode::NOT_FOUND, Json(None)),
    }
}

#[utoipa::path(
    put,
    path = "/todos/{id}",
    request_body = CreateTodo,
    responses(
        (status = 200, description = "Todo updated", body = Todo),
        (status = 404, description = "Todo not found")
    ),
    tag = "todos"
)]
async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(payload): Json<CreateTodo>,
) -> (StatusCode, Json<Option<Todo>>) {
    let mut todos = state.lock().unwrap();
    match todos.iter_mut().find(|t| t.id == id) {
        Some(todo) => {
            todo.title = payload.title;
            todo.completed = payload.completed;
            (StatusCode::OK, Json(Some(todo.clone())))
        }
        None => (StatusCode::NOT_FOUND, Json(None)),
    }
}
