use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use tonic::transport::Channel;

use crate::{
    error::ServiceError, key_value_service::key_value_service_client::KeyValueServiceClient,
    services::key_value_service,
};

pub async fn get_value(
    State(client): State<KeyValueServiceClient<Channel>>,
    Path(key): Path<String>,
) -> Result<(StatusCode, Json<Option<Value>>), ServiceError> {
    tracing::debug!("Getting value for key: {}", key);
    let value = key_value_service::get_value(client, &key).await?;
    let response = if let Some(value) = value {
        tracing::debug!("Got value: {:?} for key: {}", value, key);
        (StatusCode::OK, Json(Some(value)))
    } else {
        tracing::debug!("Value for key not found: {}", key);
        (StatusCode::NOT_FOUND, Json(None))
    };
    Ok(response)
}

pub async fn put_value(
    State(client): State<KeyValueServiceClient<Channel>>,
    Path(key): Path<String>,
    body: Json<Value>,
) -> Result<StatusCode, ServiceError> {
    tracing::debug!("Putting value {} for key {}", body.0, key);
    let updated = key_value_service::put_value(client, &key, body.0).await?;
    let response = if updated {
        tracing::debug!("Updated value for key: {}", key);
        StatusCode::NO_CONTENT
    } else {
        tracing::debug!("Created value for key: {}", key);
        StatusCode::CREATED
    };
    Ok(response)
}

pub async fn delete_value(
    State(client): State<KeyValueServiceClient<Channel>>,
    Path(key): Path<String>,
) -> Result<StatusCode, ServiceError> {
    tracing::debug!("Deleting value for key: {}", key);
    let deleted = key_value_service::delete_value(client, &key).await?;
    let response = if deleted {
        tracing::debug!("Deleted value for key: {}", key);
        StatusCode::OK
    } else {
        tracing::debug!("Value for key not found: {}", key);
        StatusCode::NOT_FOUND
    };
    Ok(response)
}
