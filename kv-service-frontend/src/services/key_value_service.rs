use serde_json::Value;
use tonic::{transport::Channel, Request};

use crate::{
    error::ServiceError,
    key_value_service::{
        key_value_service_client::KeyValueServiceClient, KeyRequest, KeyValueRequest,
    },
    utils::{prost_to_serde_json, serde_json_to_prost},
};

pub async fn get_value(
    mut client: KeyValueServiceClient<Channel>,
    key: &str,
) -> Result<Option<Value>, ServiceError> {
    let request = Request::new(KeyRequest {
        key: key.to_string(),
    });
    let response = client.get(request).await?;
    Ok(response.into_inner().value.map(prost_to_serde_json))
}

pub async fn put_value(
    mut client: KeyValueServiceClient<Channel>,
    key: &str,
    value: Value,
) -> Result<bool, ServiceError> {
    let request = Request::new(KeyValueRequest {
        key: key.to_string(),
        value: Some(serde_json_to_prost(value)),
    });
    let response = client.set(request).await?;
    Ok(response.into_inner().updated)
}

pub async fn delete_value(
    mut client: KeyValueServiceClient<Channel>,
    key: &str,
) -> Result<bool, ServiceError> {
    let request = Request::new(KeyRequest {
        key: key.to_string(),
    });
    let response = client.delete(request).await?;
    Ok(response.into_inner().deleted)
}
