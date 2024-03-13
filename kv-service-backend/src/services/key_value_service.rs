use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use crate::{
    key_value_service::{
        key_value_service_server::KeyValueService as KeyValueServiceTrait, DeleteResponse,
        GetResponse, KeyRequest, KeyValueRequest, SetResponse,
    },
    utils::{prost_to_serde_json, serde_json_to_prost},
};

#[derive(Debug)]
pub struct KeyValueService {
    storage: RwLock<HashMap<String, Value>>,
}

impl KeyValueService {
    pub fn new(storage: HashMap<String, Value>) -> Self {
        Self {
            storage: RwLock::new(storage),
        }
    }
}

#[tonic::async_trait]
impl KeyValueServiceTrait for KeyValueService {
    async fn get(&self, request: Request<KeyRequest>) -> Result<Response<GetResponse>, Status> {
        tracing::info!("Received get request: {:?}", request.get_ref());
        let value = {
            tracing::info!("Reading from storage");
            let storage = self.storage.read().await;
            storage.get(request.into_inner().key.as_str()).cloned()
        };
        tracing::info!("Read from storage");
        let response = GetResponse {
            value: value.map(serde_json_to_prost),
        };
        tracing::info!("Sending get response: {:?}", response);
        Ok(Response::new(response))
    }

    async fn set(
        &self,
        request: Request<KeyValueRequest>,
    ) -> Result<Response<SetResponse>, Status> {
        tracing::info!("Received set request: {:?}", request.get_ref());
        let KeyValueRequest { key, value } = request.into_inner();
        let Some(value) = value else {
            return Err(Status::invalid_argument("value must be set"));
        };
        let previous_value = {
            tracing::info!("Writing to storage");
            let mut storage = self.storage.write().await;
            storage.insert(key, prost_to_serde_json(value))
        };
        tracing::info!("Wrote to storage");
        let response = match previous_value {
            Some(_) => SetResponse { updated: true },
            None => SetResponse { updated: false },
        };
        tracing::info!("Sending set response: {:?}", response);
        Ok(Response::new(response))
    }

    async fn delete(
        &self,
        request: Request<KeyRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        tracing::info!("Received delete request: {:?}", request.get_ref());
        let removed_value = {
            tracing::info!("Deleting from storage");
            let mut storage = self.storage.write().await;
            storage.remove(request.into_inner().key.as_str())
        };
        tracing::info!("Deleted from storage");
        let response = match removed_value {
            Some(_) => DeleteResponse { deleted: true },
            None => DeleteResponse { deleted: false },
        };
        tracing::info!("Sending delete response: {:?}", response);
        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get() {
        let mut storage = HashMap::new();
        storage.insert("key".to_string(), serde_json::json!("value"));
        let service = KeyValueService::new(storage);
        let request = Request::new(KeyRequest {
            key: "key".to_string(),
        });
        let response = service.get(request).await.unwrap().into_inner();
        assert_eq!(
            response.value,
            Some(serde_json_to_prost(serde_json::json!("value")))
        );
    }

    #[tokio::test]
    async fn test_set() {
        let storage = HashMap::new();
        let service = KeyValueService::new(storage);
        let request = Request::new(KeyValueRequest {
            key: "key".to_string(),
            value: Some(serde_json_to_prost(serde_json::json!("value"))),
        });
        let response = service.set(request).await.unwrap().into_inner();
        assert_eq!(response.updated, false);
        assert_eq!(
            service.storage.read().await.get("key"),
            Some(&serde_json::json!("value"))
        );
    }

    #[tokio::test]
    async fn test_delete() {
        let mut storage = HashMap::new();
        storage.insert("key".to_string(), serde_json::json!("value"));
        let service = KeyValueService::new(storage);
        let request = Request::new(KeyRequest {
            key: "key".to_string(),
        });
        let response = service.delete(request).await.unwrap().into_inner();
        assert_eq!(response.deleted, true);
        assert_eq!(service.storage.read().await.get("key"), None);
    }
}
