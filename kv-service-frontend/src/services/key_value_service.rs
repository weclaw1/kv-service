use serde_json::{json, Value};

use crate::error::ServiceError;

pub async fn get_value(_: &str) -> Result<Option<Value>, ServiceError> {
    Ok(Some(json!("placeholder")))
}

pub async fn put_value(_: &str, _: Value) -> Result<bool, ServiceError> {
    Ok(true)
}

pub async fn delete_value(_: &str) -> Result<bool, ServiceError> {
    Ok(true)
}
