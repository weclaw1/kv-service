use std::{net::SocketAddr, sync::Arc};

use crate::key_value_service::key_value_service_client::KeyValueServiceClient;
use anyhow::Context;
use axum::Router;
use axum_server::{
    accept::DefaultAcceptor,
    tls_openssl::{OpenSSLAcceptor, OpenSSLConfig},
    Server,
};
use controllers::create_router;
use either::Either::{self, Left, Right};
use services::key_value_service::{GrpcKeyValueService, KeyValueServiceGrpcClient};
use tonic::transport::{Channel, ClientTlsConfig};

pub mod key_value_service {
    tonic::include_proto!("keyvalueservice");
}

mod controllers;
mod error;
mod services;
mod utils;

type EitherHttpsOrHttpServer = Either<Server<OpenSSLAcceptor>, Server<DefaultAcceptor>>;

pub async fn create_grpc_client(
    grpc_server_address: &str,
    client_tls_config: Option<ClientTlsConfig>,
) -> anyhow::Result<KeyValueServiceClient<Channel>> {
    let endpoint = if let Some(client_tls_config) = client_tls_config {
        Channel::from_shared(format!("https://{}", grpc_server_address))?
            .tls_config(client_tls_config)?
    } else {
        Channel::from_shared(format!("http://{}", grpc_server_address))?
    };

    let channel = endpoint
        .connect()
        .await
        .context("Couldn't connect to kv-service-backend, make sure it's running.")?;
    Ok(KeyValueServiceClient::new(channel))
}

pub fn create_http_server(
    addr: SocketAddr,
    tls_config: Option<OpenSSLConfig>,
    grpc_client: KeyValueServiceClient<Channel>,
) -> anyhow::Result<(EitherHttpsOrHttpServer, Router)> {
    let state = controllers::AppState {
        key_value_service: Arc::new(GrpcKeyValueService::new(KeyValueServiceGrpcClient(
            grpc_client,
        ))),
    };
    let router = create_router(state);
    let server = if let Some(tls_config) = tls_config {
        Left(axum_server::bind_openssl(addr, tls_config))
    } else {
        Right(axum_server::bind(addr))
    };
    Ok((server, router))
}
