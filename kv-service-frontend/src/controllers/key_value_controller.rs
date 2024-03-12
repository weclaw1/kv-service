use axum::{extract::Path, http::StatusCode, Json};
use serde_json::Value;

use crate::{error::ServiceError, services::key_value_service};

pub async fn get_value(Path(key): Path<String>) -> Result<Json<Value>, ServiceError> {
    tracing::debug!("Getting value for key: {}", key);
    let value = key_value_service::get_value(&key).await?;
    tracing::debug!("Got value: {:?}", value);
    Ok(Json(value))
}

pub async fn put_value(
    Path(key): Path<String>,
    body: Json<Value>,
) -> Result<StatusCode, ServiceError> {
    tracing::debug!("Putting value {} for key {}", body.0, key);
    let updated = key_value_service::put_value(&key, body.0).await?;
    let response = if updated {
        tracing::debug!("Updated value for key: {}", key);
        StatusCode::NO_CONTENT
    } else {
        tracing::debug!("Created value for key: {}", key);
        StatusCode::CREATED
    };
    Ok(response)
}

pub async fn delete_value(Path(key): Path<String>) -> Result<StatusCode, ServiceError> {
    tracing::debug!("Deleting value for key: {}", key);
    let deleted = key_value_service::delete_value(&key).await?;
    let response = if deleted {
        tracing::debug!("Deleted value for key: {}", key);
        StatusCode::OK
    } else {
        tracing::debug!("Value for key not found: {}", key);
        StatusCode::NOT_FOUND
    };
    Ok(response)
}
