use axum::async_trait;
use serde_json::Value;
use tokio::sync::Mutex;
use tonic::{transport::Channel, Request};

use crate::{
    error::ServiceError,
    key_value_service::{
        key_value_service_client::KeyValueServiceClient, KeyRequest, KeyValueRequest,
    },
    utils::{prost_to_serde_json, serde_json_to_prost},
};

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait KeyValueService: Send + Sync {
    async fn get_value(&self, key: &str) -> Result<Option<Value>, ServiceError>;
    async fn put_value(&self, key: &str, value: Value) -> Result<bool, ServiceError>;
    async fn delete_value(&self, key: &str) -> Result<bool, ServiceError>;
}

pub struct GrpcKeyValueService {
    client: Mutex<KeyValueServiceClient<Channel>>,
}

impl GrpcKeyValueService {
    pub fn new(client: KeyValueServiceClient<Channel>) -> Self {
        Self {
            client: Mutex::new(client),
        }
    }
}

#[async_trait]
impl KeyValueService for GrpcKeyValueService {
    async fn get_value(&self, key: &str) -> Result<Option<Value>, ServiceError> {
        let request = Request::new(KeyRequest {
            key: key.to_string(),
        });
        let mut client = self.client.lock().await;
        let response = client.get(request).await?;
        Ok(response.into_inner().value.map(prost_to_serde_json))
    }

    async fn put_value(&self, key: &str, value: Value) -> Result<bool, ServiceError> {
        let request = Request::new(KeyValueRequest {
            key: key.to_string(),
            value: Some(serde_json_to_prost(value)),
        });
        let mut client = self.client.lock().await;
        let response = client.set(request).await?;
        Ok(response.into_inner().updated)
    }

    async fn delete_value(&self, key: &str) -> Result<bool, ServiceError> {
        let request = Request::new(KeyRequest {
            key: key.to_string(),
        });
        let mut client = self.client.lock().await;
        let response = client.delete(request).await?;
        Ok(response.into_inner().deleted)
    }
}
