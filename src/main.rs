use poem::{EndpointExt, Route, listener::TcpListener};
use poem_openapi::OpenApiService;
use routes::api;
use tracing::info;

use crate::setup::SetupResult;

mod auth;
mod config;
mod connectors;
mod routes;
mod setup;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt().init();
    let SetupResult { db, object_storage } = setup::setup_all().await.expect("setup failed");

    let api_service =
        OpenApiService::new(api(), "Story Time", "1.0").server("http://localhost:5000/");

    let spec_endpoint = api_service.spec_endpoint();
    let spec_yaml_endpoint = api_service.spec_endpoint_yaml();

    let swagger = api_service.swagger_ui();
    let scalar = api_service.scalar();

    let app = Route::new()
        .nest("/", api_service)
        .nest("/docs/swagger", swagger)
        .nest("/docs/", scalar)
        .nest("/docs/api.json", spec_endpoint)
        .nest("/docs/api.yaml", spec_yaml_endpoint)
        .data(db)
        .data(object_storage);

    info!("listening at: http://0.0.0.0:5000");
    poem::Server::new(TcpListener::bind("0.0.0.0:5000"))
        .run(app)
        .await
}
