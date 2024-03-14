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
    if body.0.is_null() {
        return Ok(StatusCode::BAD_REQUEST);
    }
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

    #[tokio::test]
    async fn test_put_value() {
        let key = "key".to_string();
        let value = Value::String("value".to_string());

        let mut key_value_service = MockKeyValueService::new();
        key_value_service
            .expect_put_value()
            .with(eq(key.clone()), eq(value.clone()))
            .returning(move |_, _| Ok(true));

        let state = AppState {
            key_value_service: Arc::new(key_value_service),
        };

        let status = put_value(State(state), Path(key), Json(value))
            .await
            .unwrap();
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_delete_value() {
        let key = "key".to_string();

        let mut key_value_service = MockKeyValueService::new();
        key_value_service
            .expect_delete_value()
            .with(eq(key.clone()))
            .returning(move |_| Ok(true));

        let state = AppState {
            key_value_service: Arc::new(key_value_service),
        };

        let status = delete_value(State(state), Path(key)).await.unwrap();
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_value_not_found() {
        let key = "key".to_string();

        let mut key_value_service = MockKeyValueService::new();
        key_value_service
            .expect_get_value()
            .with(eq(key.clone()))
            .returning(move |_| Ok(None));

        let state = AppState {
            key_value_service: Arc::new(key_value_service),
        };

        let (status, response) = get_value(State(state), Path(key)).await.unwrap();
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(response.0, None);
    }

    #[tokio::test]
    async fn test_delete_value_not_found() {
        let key = "key".to_string();

        let mut key_value_service = MockKeyValueService::new();
        key_value_service
            .expect_delete_value()
            .with(eq(key.clone()))
            .returning(move |_| Ok(false));

        let state = AppState {
            key_value_service: Arc::new(key_value_service),
        };

        let status = delete_value(State(state), Path(key)).await.unwrap();
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_put_value_created() {
        let key = "key".to_string();
        let value = Value::String("value".to_string());

        let mut key_value_service = MockKeyValueService::new();
        key_value_service
            .expect_put_value()
            .with(eq(key.clone()), eq(value.clone()))
            .returning(move |_, _| Ok(false));

        let state = AppState {
            key_value_service: Arc::new(key_value_service),
        };

        let status = put_value(State(state), Path(key), Json(value))
            .await
            .unwrap();
        assert_eq!(status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_put_value_null() {
        let key = "key".to_string();
        let value = Value::Null;

        let state = AppState {
            key_value_service: Arc::new(MockKeyValueService::new()),
        };

        let status = put_value(State(state), Path(key), Json(value))
            .await
            .unwrap();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}
