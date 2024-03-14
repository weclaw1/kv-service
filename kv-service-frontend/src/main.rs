use axum_server::tls_openssl::OpenSSLConfig;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};
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

    let config = OpenSSLConfig::from_pem_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap_or(&PathBuf::new())
            .join("tls")
            .join("client.crt"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap_or(&PathBuf::new())
            .join("tls")
            .join("client.key"),
    )?;

    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(&PathBuf::new())
        .join("tls");
    let root_cert = std::fs::read_to_string(data_dir.join("root.crt"))?;
    let client_cert = std::fs::read_to_string(data_dir.join("client.crt"))?;
    let client_key = std::fs::read_to_string(data_dir.join("client.key"))?;
    let client_identity = Identity::from_pem(client_cert, client_key);
    let ca = Certificate::from_pem(root_cert);
    let tls = ClientTlsConfig::new()
        .ca_certificate(ca)
        .identity(client_identity)
        .domain_name("example.com");

    let channel = Channel::from_static("https://127.0.0.1:8081")
        .tls_config(tls)?
        .connect()
        .await
        .expect("Couldn't connect to kv-service-backend, make sure it's running.");
    let client = KeyValueServiceClient::new(channel);

    let state = controllers::AppState {
        key_value_service: Arc::new(GrpcKeyValueService::new(KeyValueServiceGrpcClient(client))),
    };

    let app = controllers::create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Listening on {}", addr);
    axum_server::bind_openssl(addr, config)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
