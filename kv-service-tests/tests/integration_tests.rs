use std::net::TcpListener;

use either::Either;
use kv_service_backend;
use kv_service_frontend;
use reqwest::StatusCode;
use serde_json::Value;

fn get_available_port() -> Option<u16> {
    (10000..20000).find(|port| port_is_available(*port))
}

fn port_is_available(port: u16) -> bool {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

async fn spawn_services() -> String {
    let grpc_server_address = format!("127.0.0.1:{}", get_available_port().unwrap());
    let cloned_grpc_server_address = grpc_server_address.clone();
    tokio::spawn(async move {
        let grpc_server = kv_service_backend::create_grpc_server(None).unwrap();
        grpc_server
            .serve(cloned_grpc_server_address.parse().unwrap())
            .await
            .unwrap();
    });
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let cloned_grpc_server_address = grpc_server_address.clone();
    let http_server_address = format!("127.0.0.1:{}", get_available_port().unwrap());
    let cloned_http_server_address = http_server_address.clone();
    tokio::spawn(async move {
        let grpc_client =
            kv_service_frontend::create_grpc_client(&cloned_grpc_server_address, None)
                .await
                .unwrap();
        let (server, router) = kv_service_frontend::create_http_server(
            cloned_http_server_address.parse().unwrap(),
            None,
            grpc_client,
        )
        .unwrap();
        match server {
            Either::Left(https_server) => https_server
                .serve(router.into_make_service())
                .await
                .unwrap(),
            Either::Right(http_server) => {
                http_server.serve(router.into_make_service()).await.unwrap()
            }
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    format!("http://{}/api", http_server_address)
}

#[tokio::test]
#[ignore]
async fn test_kv_services() {
    let api_address = spawn_services().await;
    let client = reqwest::Client::new();
    let response_put = client
        .put(format!("{}/test", api_address))
        .json(&"value")
        .send()
        .await
        .unwrap();
    assert_eq!(response_put.status(), StatusCode::CREATED);
    let response_get = client
        .get(format!("{}/test", api_address))
        .send()
        .await
        .unwrap();
    assert_eq!(response_get.status(), StatusCode::OK);
    assert_eq!(response_get.json::<Value>().await.unwrap(), "value");
}

#[tokio::test]
#[ignore]
async fn test_kv_services_get_nonexistent_key() {
    let api_address = spawn_services().await;
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/nonexistent", api_address))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore]
async fn test_kv_services_delete_nonexistent_key() {
    let api_address = spawn_services().await;
    let client = reqwest::Client::new();
    let response = client
        .delete(format!("{}/nonexistent", api_address))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore]
async fn test_kv_services_delete_existing_key() {
    let api_address = spawn_services().await;
    let client = reqwest::Client::new();
    let response_put = client
        .put(format!("{}/test", api_address))
        .json(&"value")
        .send()
        .await
        .unwrap();
    assert_eq!(response_put.status(), StatusCode::CREATED);
    let response_delete = client
        .delete(format!("{}/test", api_address))
        .send()
        .await
        .unwrap();
    assert_eq!(response_delete.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore]
async fn test_kv_services_put_existing_key_different_value() {
    let api_address = spawn_services().await;
    let client = reqwest::Client::new();
    let response_put = client
        .put(format!("{}/test", api_address))
        .json(&"value")
        .send()
        .await
        .unwrap();
    assert_eq!(response_put.status(), StatusCode::CREATED);
    let response_put = client
        .put(format!("{}/test", api_address))
        .json(&"value2")
        .send()
        .await
        .unwrap();
    assert_eq!(response_put.status(), StatusCode::NO_CONTENT);
    let response_get = client
        .get(format!("{}/test", api_address))
        .send()
        .await
        .unwrap();
    assert_eq!(response_get.status(), StatusCode::OK);
    assert_eq!(response_get.json::<Value>().await.unwrap(), "value2");
}

#[tokio::test]
#[ignore]
async fn kest_kv_services_put_null_value() {
    let api_address = spawn_services().await;
    let client = reqwest::Client::new();
    let response_put = client
        .put(format!("{}/test", api_address))
        .json(&Value::Null)
        .send()
        .await
        .unwrap();
    assert_eq!(response_put.status(), StatusCode::BAD_REQUEST);
}
