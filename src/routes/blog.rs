use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize)]
#[schema(title = "BlogMetadata")]
struct Metadata {
    slug: String,
    views: u64,
    comments: u64,
    reactions: u64,
}

#[utoipa::path(
    get,
    path = "/blog/metadata/{slug}",
    params(
        ("slug" = String, Path, description = "Slug from blog post"),
    ),
    responses(
        (status = 200, description = "Metadata from one blog post", body = Metadata),
        (status = NOT_FOUND, description = "Blog post was not found")
    )
)]
pub async fn get_metadata(Path(slug): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let metadata = Metadata {
        slug,
        views: 1,
        comments: 2,
        reactions: 3,
    };

    Ok(Json(metadata))
}
