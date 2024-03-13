use axum::{
    routing::{delete, get, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::key_value_service::key_value_service_client::KeyValueServiceClient;

mod controllers;
mod error;
mod services;
mod utils;

pub mod key_value_service {
    tonic::include_proto!("keyvalueservice");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "kv_service_frontend=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let client = KeyValueServiceClient::connect("http://0.0.0.0:8081").await?;

    let app = Router::new()
        .route(
            "/api/:key",
            get(controllers::key_value_controller::get_value),
        )
        .route(
            "/api/:key",
            put(controllers::key_value_controller::put_value),
        )
        .route(
            "/api/:key",
            delete(controllers::key_value_controller::delete_value),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(client);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
