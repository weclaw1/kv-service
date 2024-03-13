use axum::async_trait;
use serde_json::Value;
use tokio::sync::Mutex;
use tonic::{transport::Channel, Request};

use crate::{
    error::ServiceError,
    key_value_service::{
        key_value_service_client::KeyValueServiceClient, DeleteResponse, GetResponse, KeyRequest,
        KeyValueRequest, SetResponse,
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

pub struct KeyValueServiceGrpcClient(pub KeyValueServiceClient<Channel>);

#[cfg_attr(test, automock)]
#[async_trait]
pub trait KeyValueServiceClientTrait: Send + Sync {
    async fn get(
        &mut self,
        request: Request<KeyRequest>,
    ) -> Result<tonic::Response<GetResponse>, tonic::Status>;
    async fn set(
        &mut self,
        request: Request<KeyValueRequest>,
    ) -> Result<tonic::Response<SetResponse>, tonic::Status>;
    async fn delete(
        &mut self,
        request: Request<KeyRequest>,
    ) -> Result<tonic::Response<DeleteResponse>, tonic::Status>;
}

#[async_trait]
impl KeyValueServiceClientTrait for KeyValueServiceGrpcClient {
    async fn get(
        &mut self,
        request: Request<KeyRequest>,
    ) -> Result<tonic::Response<GetResponse>, tonic::Status> {
        self.0.get(request).await
    }

    async fn set(
        &mut self,
        request: Request<KeyValueRequest>,
    ) -> Result<tonic::Response<SetResponse>, tonic::Status> {
        self.0.set(request).await
    }

    async fn delete(
        &mut self,
        request: Request<KeyRequest>,
    ) -> Result<tonic::Response<DeleteResponse>, tonic::Status> {
        self.0.delete(request).await
    }
}

pub struct GrpcKeyValueService<T: KeyValueServiceClientTrait + Send> {
    client: Mutex<T>,
}

impl<T: KeyValueServiceClientTrait + Send> GrpcKeyValueService<T> {
    pub fn new(client: T) -> Self {
        Self {
            client: Mutex::new(client),
        }
    }
}

#[async_trait]
impl<T: KeyValueServiceClientTrait + Send> KeyValueService for GrpcKeyValueService<T> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_value() {
        let mut mock = MockKeyValueServiceClientTrait::new();
        mock.expect_get()
            .withf(|request| request.get_ref().key == "key")
            .times(1)
            .returning(|_| {
                Ok(tonic::Response::new(GetResponse {
                    value: Some(serde_json_to_prost(serde_json::json!("value"))),
                }))
            });

        let service = GrpcKeyValueService::new(mock);
        let result = service.get_value("key").await.unwrap();
        assert_eq!(result, Some(serde_json::json!("value")));
    }

    #[tokio::test]
    async fn test_put_value() {
        let mut mock = MockKeyValueServiceClientTrait::new();
        mock.expect_set()
            .withf(|request| {
                request.get_ref().key == "key"
                    && request.get_ref().value
                        == Some(serde_json_to_prost(serde_json::json!("value")))
            })
            .times(1)
            .returning(|_| Ok(tonic::Response::new(SetResponse { updated: true })));

        let service = GrpcKeyValueService::new(mock);
        let result = service
            .put_value("key", serde_json::json!("value"))
            .await
            .unwrap();
        assert_eq!(result, true);
    }

    #[tokio::test]
    async fn test_delete_value() {
        let mut mock = MockKeyValueServiceClientTrait::new();
        mock.expect_delete()
            .withf(|request| request.get_ref().key == "key")
            .times(1)
            .returning(|_| Ok(tonic::Response::new(DeleteResponse { deleted: true })));

        let service = GrpcKeyValueService::new(mock);
        let result = service.delete_value("key").await.unwrap();
        assert_eq!(result, true);
    }

    #[tokio::test]
    async fn test_get_value_error() {
        let mut mock = MockKeyValueServiceClientTrait::new();
        mock.expect_get()
            .withf(|request| request.get_ref().key == "key")
            .times(1)
            .returning(|_| Err(tonic::Status::new(tonic::Code::Internal, "Internal error")));

        let service = GrpcKeyValueService::new(mock);
        let result = service.get_value("key").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_put_value_error() {
        let mut mock = MockKeyValueServiceClientTrait::new();
        mock.expect_set()
            .withf(|request| {
                request.get_ref().key == "key"
                    && request.get_ref().value
                        == Some(serde_json_to_prost(serde_json::json!("value")))
            })
            .times(1)
            .returning(|_| Err(tonic::Status::new(tonic::Code::Internal, "Internal error")));

        let service = GrpcKeyValueService::new(mock);
        let result = service.put_value("key", serde_json::json!("value")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_value_error() {
        let mut mock = MockKeyValueServiceClientTrait::new();
        mock.expect_delete()
            .withf(|request| request.get_ref().key == "key")
            .times(1)
            .returning(|_| Err(tonic::Status::new(tonic::Code::Internal, "Internal error")));

        let service = GrpcKeyValueService::new(mock);
        let result = service.delete_value("key").await;
        assert!(result.is_err());
    }
}
