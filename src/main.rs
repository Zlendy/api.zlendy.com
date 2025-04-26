use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get};
use serde_json::json;
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(paths(hello))]
pub struct ApiDoc;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/hello", get(hello))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[utoipa::path(
    get,
    path = "/hello",
    responses(
        (status = 200, description = "Hello, world!")
    )
)]
async fn hello() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "message": "Hello, world!" })))
}
