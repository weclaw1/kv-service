use std::collections::HashMap;

use key_value_service::key_value_service_server::KeyValueServiceServer;
use services::key_value_service::KeyValueService;
use tonic::transport::Server;
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

    let addr = "127.0.0.1:8081".parse()?;
    let storage = HashMap::new();
    let key_value_service = KeyValueService::new(storage);

    Server::builder()
        .trace_fn(|_| tracing::info_span!("kv_service_backend_server"))
        .add_service(KeyValueServiceServer::new(key_value_service))
        .serve(addr)
        .await?;

    Ok(())
}
