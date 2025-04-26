use axum::{Router, routing::get};
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod routes;
use crate::routes::blog;

#[derive(OpenApi)]
#[openapi(paths(blog::get_metadata))]
pub struct ApiDoc;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/blog/metadata/{slug}", get(blog::get_metadata))
        .merge(SwaggerUi::new("/").url("/api-doc/openapi.json", ApiDoc::openapi()));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
