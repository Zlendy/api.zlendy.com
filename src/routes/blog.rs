use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::args::Args;

#[derive(ToSchema, Serialize)]
#[schema(title = "BlogMetadata")]
struct Metadata {
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
pub async fn get_metadata(
    Path(slug): Path<String>,
    State(args): State<Args>,
) -> Result<impl IntoResponse, StatusCode> {
    println!("{}, {:#?}", slug, args);

    let metadata = Metadata {
        views: 1,
        comments: 2,
        reactions: 3,
    };

    Ok(Json(metadata))
}
