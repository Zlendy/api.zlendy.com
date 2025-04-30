use core::option::Option::Some;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    Json, debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;

pub type SharedBlogState = Arc<Mutex<BlogState>>;

#[derive(Default, Debug, Clone)]
pub struct BlogState {
    value: Option<BlogRoutes>,
    last_modified: DateTime<Utc>,
}

pub type BlogRoutes = HashMap<String, BlogValue>;

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
    let diff = Utc::now() - last_modified;
    return diff.num_minutes() > minutes;
}

async fn update_routes(
    current_routes: Option<BlogRoutes>,
    host: String,
) -> Result<BlogRoutes, reqwest::Error> {
    let response = reqwest::get(format!("{host}/blog.json"))
        .await?
        .json::<HashMap<String, Option<String>>>()
        .await?;

    // Reuse BlogRoutes if it exists
    let mut routes = match current_routes {
        Some(routes) => routes,
        None => BlogRoutes::new(),
    };

    for (slug, fediverse) in response {
        let metadata = match routes.get(&slug) {
            Some(value) => value.metadata.clone(),
            None => BlogMetadata::default(),
        };

        routes.insert(
            slug,
            BlogValue {
                metadata, // This field is updated later
                last_modified: Utc::now(),
                fediverse,
            },
        );
    }

    return Ok(routes);
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
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let AppState { args, blog } = state;
    let mut blog_state = blog.lock().expect("mutex was poisoned").clone();

    let routes = match blog_state.value.clone() {
        Some(routes) if expired_cache(blog_state.last_modified, 5) => {
            // Routes cache has expired
            let routes = update_routes(Some(routes), args.zlendy_url.clone()).await;
            blog_state.last_modified = Utc::now();

            routes
        }
        Some(routes) => Ok(routes), // Routes cache is still valid
        None => {
            // Routes cache does not exist
            let routes = update_routes(None, args.zlendy_url.clone()).await;
            blog_state.last_modified = Utc::now();

            routes
        }
    };

    let Ok(routes) = routes else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    blog_state.value = Some(routes.clone());

    {
        // Mutex guard is unlocked outside this scope
        let mut blog_mutex = blog.lock().expect("mutex was poisoned");
        blog_mutex.value = blog_state.value.clone();
        blog_mutex.last_modified = blog_state.last_modified;
    }

    let Some(value) = routes.get(&slug) else {
        return Err(StatusCode::NOT_FOUND);
    };

    // TODO: Load data from Umami Analytics and store it in BlogMetadata

    println!("{}, {:#?}, {:#?}", slug, args, blog_state);

    Ok(Json(value.metadata.clone()))
}
