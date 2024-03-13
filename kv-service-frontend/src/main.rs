use std::{net::SocketAddr, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    key_value_service::key_value_service_client::KeyValueServiceClient,
    services::key_value_service::{GrpcKeyValueService, KeyValueServiceGrpcClient},
};

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

    let client = KeyValueServiceClient::connect("http://127.0.0.1:8081").await?;

    let state = controllers::AppState {
        key_value_service: Arc::new(GrpcKeyValueService::new(KeyValueServiceGrpcClient(client))),
    };

    let app = controllers::create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
