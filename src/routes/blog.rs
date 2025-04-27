use std::collections::HashMap;

use axum::{
    Json, debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::SharedAppState;

pub type BlogRoutes = HashMap<String, BlogValue>;

#[derive(Default, Debug, Clone)]
pub struct BlogState {
    value: Option<BlogRoutes>,
    last_modified: DateTime<Utc>,
}

#[derive(Default, Debug, Clone)]
pub struct BlogValue {
    metadata: BlogMetadata,
    last_modified: DateTime<Utc>,
    fediverse: Option<String>,
}

#[derive(ToSchema, Serialize, Default, Debug, Clone)]
pub struct BlogMetadata {
    views: u64,
    comments: u64,
    reactions: u64,
}

fn expired_cache(last_modified: DateTime<Utc>, minutes: i64) -> bool {
    let diff = last_modified - Utc::now();
    return diff.num_minutes() > minutes;
}

async fn update_routes() -> BlogRoutes {
    let mut routes = BlogRoutes::new();
    routes.insert("first-post".to_string(), BlogValue::default()); // TODO: Load from zlendy.com

    return routes;
}

#[utoipa::path(
    get,
    path = "/blog/metadata/{slug}",
    params(
        ("slug" = String, Path, description = "Slug from blog post"),
    ),
    responses(
        (status = 200, description = "Metadata from one blog post", body = BlogMetadata),
        (status = NOT_FOUND, description = "Blog post was not found")
    )
)]
#[debug_handler]
pub async fn get_metadata(
    Path(slug): Path<String>,
    State(state): State<SharedAppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let SharedAppState { args, blog } = state;
    let mut blog = blog.write().await;

    let routes = match blog.value.clone() {
        Some(_) if expired_cache(blog.last_modified, 5) => update_routes().await, // Routes cache has expired
        Some(routes) => routes,        // Routes cache is still valid
        None => update_routes().await, // Routes cache does not exist
    };
    blog.value = Some(routes.clone());

    let Some(value) = routes.get(&slug) else {
        return Err(StatusCode::NOT_FOUND);
    };

    // TODO: Load data from Umami Analytics and store it in BlogMetadata

    println!("{}, {:#?}, {:#?}", slug, args, blog);

    Ok(Json(value.metadata.clone()))
}
