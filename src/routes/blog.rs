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
    fediverse::{self, NoteResponse},
    umami::{self, LoginRequest},
};

pub type SharedBlogState = Arc<Mutex<BlogState>>;

#[derive(Default, Debug, Clone)]
pub struct BlogState {
    value: BlogRoutes,
    values_last_modified: DateTime<Utc>,
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

async fn update_routes(mut routes: BlogRoutes, host: String) -> Result<BlogRoutes, reqwest::Error> {
    println!("fn: blog::update_routes");

    let response = reqwest::get(format!("{host}/blog.json"))
        .await?
        .json::<HashMap<String, Option<String>>>()
        .await?;

    for (slug, fediverse) in response {
        let metadata = match routes.get(&slug) {
            Some(value) => value.metadata.clone(),
            None => BlogMetadata::default(),
        };

        routes.insert(
            slug,
            BlogValue {
                // These fields are updated later
                metadata,
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

    if expired_cache(blog_state.last_modified, 5) {
        let updated_routes = update_routes(blog_state.value.clone(), args.zlendy_url.clone()).await;
        let Ok(updated_routes) = updated_routes else {
            println!("error: routes cache couldn't be updated");
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        };

        blog_state.value = updated_routes;
        blog_state.last_modified = Utc::now();
    }

    let Some(mut value) = blog_state.value.get(&slug).cloned() else {
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

    let fediverse_note = match value.fediverse {
        Some(ref fediverse) => {
            let response = fediverse::note(args.fediverse_url.clone(), fediverse.to_string()).await;

            let Ok(response) = response else {
                println!("error: couldn't parse note");
                return Err(StatusCode::SERVICE_UNAVAILABLE);
            };

            response
        }
        None => NoteResponse::default(),
    };

    value.metadata = BlogMetadata {
        views: umami_pageviews,
        comments: fediverse_note.replies_count,
        reactions: fediverse_note.reaction_count,
    };
    value.last_modified = Utc::now();

    blog_state.value.insert(slug.clone(), value.clone());

    println!("info: fetched updated data successfully");
    // println!("{:#?}", blog_state);
    Ok(Json(value.metadata.clone()))
}

#[utoipa::path(
    get,
    path = "/blog/metadata",
    responses(
        (status = 200, description = "Metadata from all blog posts", body = HashMap<String, BlogMetadata>),
    )
)]
#[debug_handler]
pub async fn get_metadata_all(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let AppState { args, blog } = state;
    let mut blog_state = blog.lock().await;

    println!("\n\nfn: blog::get_metadata_all");

    if expired_cache(blog_state.last_modified, 5) {
        let updated_routes = update_routes(blog_state.value.clone(), args.zlendy_url.clone()).await;
        let Ok(updated_routes) = updated_routes else {
            println!("error: routes cache couldn't be updated");
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        };

        blog_state.value = updated_routes;
        blog_state.last_modified = Utc::now();
    }

    if !expired_cache(blog_state.values_last_modified, 5) {
        println!("info: routes cache is still valid");

        let hashmap: HashMap<String, BlogMetadata> = blog_state
            .value
            .iter()
            .map(|(key, value)| (key.clone(), value.metadata.clone()))
            .collect();
        return Ok(Json(hashmap));
    }

    println!("info: updating entries in routes cache");

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

    let umami_pageviews_map = umami::pageviews_prefix(
        args.umami_url.clone(),
        umami_token.clone(),
        args.umami_website_id.clone(),
        "/blog".to_string(),
    )
    .await;

    let Ok(umami_pageviews_map) = umami_pageviews_map else {
        println!("{:#?}", umami_pageviews_map);
        println!("error: couldn't parse pageviews");
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let fediverse_notes_map =
        fediverse::notes_user(args.fediverse_url.clone(), args.fediverse_user_id.clone()).await;

    let Ok(fediverse_notes_map) = fediverse_notes_map else {
        println!("error: couldn't parse notes");
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let mut routes = HashMap::<String, BlogMetadata>::new();

    for (slug, value) in &mut blog_state.value {
        let views = umami_pageviews_map
            .get(&format!("/blog/{slug}"))
            .cloned()
            .unwrap_or_default();

        let mut comments: u64 = 0;
        let mut reactions: u64 = 0;

        if let Some(fediverse) = &value.fediverse {
            let note = fediverse_notes_map
                .get(fediverse)
                .cloned()
                .unwrap_or_default();
            comments = note.replies_count;
            reactions = note.reaction_count;
        }

        let metadata = BlogMetadata {
            views,
            comments,
            reactions,
        };

        routes.insert(slug.clone(), metadata.clone());
        value.last_modified = Utc::now();
        value.metadata = metadata;
    }

    blog_state.values_last_modified = Utc::now();

    println!("info: fetched updated data successfully");
    // println!("{:#?}", blog_state);
    Ok(Json(routes))
}
