use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

#[utoipa::path(
    get,
    path = "/hello",
    responses(
        (status = 200, description = "Hello, world!")
    )
)]
pub async fn get() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "message": "Hello, world!" })))
}
