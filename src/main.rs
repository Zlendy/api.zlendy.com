use std::{borrow::Cow, error::Error, sync::Arc, time::Duration};

use args::Args;
use axum::{
    Router, error_handling::HandleErrorLayer, http::StatusCode, response::IntoResponse,
    routing::get,
};
use routes::blog::BlogState;
use tokio::{net::TcpListener, sync::RwLock};
use tower::{BoxError, ServiceBuilder};
use tower_http::compression::CompressionLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod args;
pub mod routes;
use crate::routes::blog;

#[derive(Default, Clone)]
pub struct SharedAppState {
    blog: Arc<RwLock<BlogState>>,
    args: Args,
}

#[derive(OpenApi)]
#[openapi(info(title = "api.zlendy.com"), paths(blog::get_metadata))]
pub struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::load()?;
    let address = format!("{}:{}", args.host, args.port);

    let mut state = SharedAppState::default();
    state.args = args;

    let app = Router::new()
        .route("/blog/metadata/{slug}", get(blog::get_metadata))
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
        .with_state(state);

    let listener = TcpListener::bind(address).await.unwrap();
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
