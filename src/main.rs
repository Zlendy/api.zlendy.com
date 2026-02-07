use std::{borrow::Cow, time::Duration};

use args::Args;
use axum::http::header::{ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue};
use axum::response::Response;
use axum::{
    Router,
    error_handling::HandleErrorLayer,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
};
use routes::blog::SharedBlogState;
use tokio::net::TcpListener;
use tower::{BoxError, ServiceBuilder};
use tower_http::compression::CompressionLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod args;
pub mod errors;
pub mod fediverse;
pub mod routes;
pub mod umami;
use crate::routes::blog;

#[derive(Default, Clone)]
pub struct AppState {
    blog: SharedBlogState,
    args: Args,
}

#[derive(OpenApi)]
#[openapi(
    info(title = "api.zlendy.com"),
    paths(blog::get_metadata, blog::get_metadata_all)
)]
pub struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), dotenvy::Error> {
    env_logger::init();

    let args = Args::load()?;
    log::trace!("{args:#?}");

    let address = format!("{}:{}", args.host, args.port);

    let state = AppState {
        args,
        ..Default::default()
    };

    let app = Router::new()
        .route("/blog/metadata/{slug}", get(blog::get_metadata))
        .route("/blog/metadata", get(blog::get_metadata_all))
        .merge(SwaggerUi::new("/").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .layer(CompressionLayer::new())
        .layer(
            ServiceBuilder::new()
                // Handle errors from middleware
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(10)),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            response_headers,
        ))
        .with_state(state);

    let listener = TcpListener::bind(&address).await.unwrap();
    log::info!("listening on {address}");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out"));
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Cow::from("service is overloaded, try again later"),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from(format!("Unhandled internal error: {error}")),
    )
}

async fn response_headers(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    if let Some(value) = state.args.access_control_allow_origin {
        match value.parse::<HeaderValue>() {
            Ok(value) => {
                response
                    .headers_mut()
                    .insert(ACCESS_CONTROL_ALLOW_ORIGIN, value);
            }
            Err(_) => {
                log::error!("value of header \"{ACCESS_CONTROL_ALLOW_ORIGIN}\" couldn't be parsed");
            }
        }
    }

    response
}
