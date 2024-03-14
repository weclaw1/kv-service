use anyhow::Context;
use axum_server::tls_openssl::OpenSSLConfig;
use either::Either;
use kv_service_frontend::create_grpc_client;
use std::{net::SocketAddr, path::PathBuf, str::FromStr};
use tonic::transport::{Certificate, ClientTlsConfig, Identity};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    let tls = dotenvy::var("TLS").context("TLS must be set")?.parse()?;

    let (http_server_tls_config, grpc_client_tls_config) = if tls {
        let http_server_tls_config = create_http_server_tls_config()?;
        let grpc_client_tls_config = create_grpc_client_tls_config()?;
        (Some(http_server_tls_config), Some(grpc_client_tls_config))
    } else {
        (None, None)
    };

    let grpc_server_address =
        dotenvy::var("GRPC_SERVER_ADDRESS").context("GRPC_SERVER_ADDRESS must be set")?;

    let client = create_grpc_client(&grpc_server_address, grpc_client_tls_config).await?;
    let http_server_address =
        dotenvy::var("HTTP_SERVER_ADDRESS").context("HTTP_SERVER_ADDRESS must be set")?;
    let http_server_address = SocketAddr::from_str(&http_server_address)?;

    let (server, router) = kv_service_frontend::create_http_server(
        http_server_address,
        http_server_tls_config,
        client,
    )?;

    tracing::info!("Listening on {}", http_server_address);
    match server {
        Either::Left(https_server) => https_server.serve(router.into_make_service()).await?,
        Either::Right(http_server) => http_server.serve(router.into_make_service()).await?,
    }
    Ok(())
}

pub fn create_grpc_client_tls_config() -> anyhow::Result<ClientTlsConfig> {
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(&PathBuf::new())
        .join("tls");
    let root_cert = std::fs::read_to_string(data_dir.join("root.crt"))?;
    let client_cert = std::fs::read_to_string(data_dir.join("client.crt"))?;
    let client_key = std::fs::read_to_string(data_dir.join("client.key"))?;
    let client_identity = Identity::from_pem(client_cert, client_key);
    let ca = Certificate::from_pem(root_cert);
    let ca_domain_name = dotenvy::var("CA_DOMAIN_NAME").context("CA_DOMAIN_NAME must be set")?;
    Ok(ClientTlsConfig::new()
        .ca_certificate(ca)
        .identity(client_identity)
        .domain_name(ca_domain_name))
}

pub fn create_http_server_tls_config() -> anyhow::Result<OpenSSLConfig> {
    Ok(OpenSSLConfig::from_pem_file(
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
    )?)
}
