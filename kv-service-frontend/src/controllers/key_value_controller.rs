use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;

use crate::error::ServiceError;

use super::AppState;

pub async fn get_value(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<(StatusCode, Json<Option<Value>>), ServiceError> {
    tracing::debug!("Getting value for key: {}", key);
    let value = state.key_value_service.get_value(&key).await?;
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
    State(state): State<AppState>,
    Path(key): Path<String>,
    body: Json<Value>,
) -> Result<StatusCode, ServiceError> {
    tracing::debug!("Putting value {} for key {}", body.0, key);
    let updated = state.key_value_service.put_value(&key, body.0).await?;
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
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<StatusCode, ServiceError> {
    tracing::debug!("Deleting value for key: {}", key);
    let deleted = state.key_value_service.delete_value(&key).await?;
    let response = if deleted {
        tracing::debug!("Deleted value for key: {}", key);
        StatusCode::OK
    } else {
        tracing::debug!("Value for key not found: {}", key);
        StatusCode::NOT_FOUND
    };
    Ok(response)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::eq;

    use crate::services::key_value_service::MockKeyValueService;

    use super::*;

    #[tokio::test]
    async fn test_get_value() {
        let key = "key".to_string();
        let value = Value::String("value".to_string());

        let mut key_value_service = MockKeyValueService::new();
        let cloned_value = value.clone();
        key_value_service
            .expect_get_value()
            .with(eq(key.clone()))
            .returning(move |_| Ok(Some(cloned_value.clone())));

        let state = AppState {
            key_value_service: Arc::new(key_value_service),
        };

        let (status, response) = get_value(State(state), Path(key)).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response.0, Some(value));
    }
}
