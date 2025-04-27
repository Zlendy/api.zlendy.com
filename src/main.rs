use std::error::Error;

use args::Args;
use axum::{Router, routing::get};
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod args;
pub mod routes;
use crate::routes::blog;

#[derive(OpenApi)]
#[openapi(info(title = "api.zlendy.com"), paths(blog::get_metadata))]
pub struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::load()?;
    let address = format!("{}:{}", args.host, args.port);

    let app = Router::new()
        .route("/blog/metadata/{slug}", get(blog::get_metadata))
        .merge(SwaggerUi::new("/").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .with_state(args);

    let listener = TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
