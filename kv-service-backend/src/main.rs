use std::{collections::HashMap, path::PathBuf};

use key_value_service::key_value_service_server::KeyValueServiceServer;
use services::key_value_service::KeyValueService;
use tonic::transport::{Certificate, Server, ServerTlsConfig};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod key_value_service {
    tonic::include_proto!("keyvalueservice");
}

mod services;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kv_service_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(&PathBuf::new())
        .join("tls");
    let cert = std::fs::read_to_string(data_dir.join("server.crt"))?;
    let key = std::fs::read_to_string(data_dir.join("server.key"))?;

    let server_identity = tonic::transport::Identity::from_pem(cert, key);

    let client_ca_cert = std::fs::read_to_string(data_dir.join("root.crt"))?;
    let client_ca_cert = Certificate::from_pem(client_ca_cert);

    let addr = "127.0.0.1:8081".parse()?;
    let storage = HashMap::new();
    let key_value_service = KeyValueService::new(storage);

    tracing::info!("Listening on {}", addr);
    Server::builder()
        .tls_config(
            ServerTlsConfig::new()
                .identity(server_identity)
                .client_ca_root(client_ca_cert),
        )?
        .trace_fn(|_| tracing::info_span!("kv_service_backend_server"))
        .add_service(KeyValueServiceServer::new(key_value_service))
        .serve(addr)
        .await?;

    Ok(())
}
