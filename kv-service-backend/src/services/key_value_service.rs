use serde_json::Value;
use std::{collections::HashMap, sync::RwLock};
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
    pub fn new(storage: RwLock<HashMap<String, Value>>) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl KeyValueServiceTrait for KeyValueService {
    async fn get(&self, request: Request<KeyRequest>) -> Result<Response<GetResponse>, Status> {
        tracing::info!("Received get request: {:?}", request.get_ref());
        let value = {
            tracing::info!("Reading from storage");
            let storage = self.storage.read().unwrap();
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
            let mut storage = self.storage.write().unwrap();
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
            let mut storage = self.storage.write().unwrap();
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
