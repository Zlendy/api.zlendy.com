use core::option::Option::Some;
use std::{collections::HashMap, sync::Arc};

use axum::{
    Json, debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::Mutex;
use utoipa::ToSchema;

use crate::{
    AppState,
    umami::{self, LoginRequest},
};

pub type SharedBlogState = Arc<Mutex<BlogState>>;

#[derive(Default, Debug, Clone)]
pub struct BlogState {
    value: Option<BlogRoutes>,
    last_modified: DateTime<Utc>,
    umami_token: Option<String>,
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
    diff.num_minutes() > minutes
}

async fn update_routes(
    current_routes: Option<BlogRoutes>,
    host: String,
) -> Result<BlogRoutes, reqwest::Error> {
    println!("fn: blog::update_routes");

    let response = reqwest::get(format!("{host}/blog.json"))
        .await?
        .json::<HashMap<String, Option<String>>>()
        .await?;

    // Reuse BlogRoutes if it exists
    let mut routes = current_routes.unwrap_or_default();

    for (slug, fediverse) in response {
        let metadata = match routes.get(&slug) {
            Some(value) => value.metadata.clone(),
            None => BlogMetadata::default(),
        };

        routes.insert(
            slug,
            BlogValue {
                metadata, // This field is updated later
                last_modified: DateTime::default(),
                fediverse,
            },
        );
    }

    Ok(routes)
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
    let mut blog_state = blog.lock().await;

    println!("\n\nfn: blog::get_metadata ({})", slug);

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

    let Ok(mut routes) = routes else {
        println!("error: routes cache couldn't be updated");
        println!("{:#?}", routes);
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    blog_state.value = Some(routes.clone());

    let Some(mut value) = routes.get(&slug).cloned() else {
        println!("error: slug \"{}\" was not found in routes cache", slug);
        return Err(StatusCode::NOT_FOUND);
    };

    if !expired_cache(value.last_modified, 5) {
        println!("info: found valid entry in routes cache");
        return Ok(Json(value.metadata.clone()));
    }

    println!("info: updating entry in routes cache");

    let umami_token =
        match umami::verify(args.umami_url.clone(), blog_state.umami_token.clone()).await {
            Ok(token) => token,
            Err(_) => {
                let umami_login = umami::login(
                    args.umami_url.clone(),
                    LoginRequest {
                        username: args.umami_username.clone(),
                        password: args.umami_password.clone(),
                    },
                )
                .await;

                let Ok(umami_login) = umami_login else {
                    println!("error: invalid umami login credentials");
                    return Err(StatusCode::SERVICE_UNAVAILABLE);
                };

                umami_login.token
            }
        };

    blog_state.umami_token = Some(umami_token.clone());

    let umami_pageviews = umami::pageviews_path(
        args.umami_url.clone(),
        umami_token.clone(),
        args.umami_website_id.clone(),
        format!("/blog/{}", slug.clone()),
    )
    .await;

    let Ok(umami_pageviews) = umami_pageviews else {
        println!("error: couldn't parse pageviews");
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    value.metadata.views = umami_pageviews;
    value.last_modified = Utc::now();

    routes.insert(slug.clone(), value.clone());

    blog_state.value = Some(routes.clone());

    println!("info: fetched updated data successfully");
    // println!("{:#?}", blog_state);
    Ok(Json(value.metadata.clone()))
}
