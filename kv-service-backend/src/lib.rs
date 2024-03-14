use std::collections::HashMap;

use key_value_service::key_value_service_server::KeyValueServiceServer;
use services::key_value_service::KeyValueService;
use tonic::transport::{server::Router, Server, ServerTlsConfig};

pub mod key_value_service {
    tonic::include_proto!("keyvalueservice");
}

mod services;
mod utils;

pub fn create_grpc_server(tls_config: Option<ServerTlsConfig>) -> anyhow::Result<Router> {
    let storage = HashMap::new();
    let key_value_service = KeyValueService::new(storage);

    let mut server = Server::builder();

    if let Some(tls_config) = tls_config {
        server = server.tls_config(tls_config)?;
    }

    Ok(server
        .trace_fn(|_| tracing::info_span!("kv_service_backend_server"))
        .add_service(KeyValueServiceServer::new(key_value_service)))
}
