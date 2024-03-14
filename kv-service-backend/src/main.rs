use std::path::PathBuf;

use anyhow::Context;
use kv_service_backend::create_grpc_server;
use tonic::transport::{Certificate, ServerTlsConfig};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kv_service_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = dotenvy::var("GRPC_SERVER_ADDRESS")
        .context("GRPC_SERVER_ADDRESS must be set")?
        .parse()?;

    let tls = dotenvy::var("TLS").context("TLS must be set")?.parse()?;

    let tls_config = if tls {
        Some(create_tls_config()?)
    } else {
        None
    };

    let server = create_grpc_server(tls_config)?;

    tracing::info!("Listening on {}", addr);
    server.serve(addr).await?;

    Ok(())
}

fn create_tls_config() -> anyhow::Result<ServerTlsConfig> {
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(&PathBuf::new())
        .join("tls");
    let cert = std::fs::read_to_string(data_dir.join("server.crt"))?;
    let key = std::fs::read_to_string(data_dir.join("server.key"))?;

    let server_identity = tonic::transport::Identity::from_pem(cert, key);

    let client_ca_cert = std::fs::read_to_string(data_dir.join("root.crt"))?;
    let client_ca_cert = Certificate::from_pem(client_ca_cert);

    Ok(ServerTlsConfig::new()
        .identity(server_identity)
        .client_ca_root(client_ca_cert))
}
