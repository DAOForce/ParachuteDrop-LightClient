use actix_web::{get, web, HttpResponse, Responder};
use reqwest::Client;
use crate::http::error::HTTPError;
use crate::http::response::HealthResponse;

#[get("/osmosis")]
pub async fn osmosis_health(client: web::Data<Client>) -> Result<HttpResponse, HTTPError> {
    let res = client
        .get("https://osmosis-mainnet-rpc.allthatnode.com:1317/node_info")
        .send()
        .await
        .map_err(|_| HTTPError::Timeout)?;

    let data = if res.status().is_success() {
        let body = res.text().await.map_err(|_| HTTPError::Timeout)?;
        let data: serde_json::Value = serde_json::from_str(&body).map_err(|_| HTTPError::BadRequest)?;
        HealthResponse {
            status: 200,
            message: "now working".to_string(),
            data: Some(data),
        }
    } else {
        HealthResponse {
            status: 503,
            message: "service is not working well".to_string(),
            data: None,
        }
    };
    Ok(HttpResponse::Ok().json(data))
}
