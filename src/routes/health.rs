use crate::client::factory::{
    build_request_by_chain_name, build_request_with_body_and_chain_name, get_bank_grpc_client,
};
use crate::http::error::HTTPError;
use crate::http::method::HTTPRequestMethod;
use crate::http::response;
use crate::http::response::HealthResponse;
use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder};
use ibc_proto::cosmos::bank::v1beta1::{
    query_client::QueryClient, QueryAllBalancesRequest, QueryBalanceRequest, QueryBalanceResponse,
};
use ibc_proto::ibc::core::channel::v1::acknowledgement::Response::Error;
use reqwest::{Client, Response, Version};
use tonic::codegen::Body;

#[get("/evmos")]
pub async fn evmos_health() -> Result<HttpResponse, HTTPError> {
    let builder = build_request_by_chain_name("evmos", HTTPRequestMethod::GET).await;
    let res = builder.send().await.map_err(|_| HTTPError::Timeout)?;

    response::build_health_response(Some(res), serde_json::Value::Null).await
}

#[get("/polygon")]
pub async fn polygon_health(client: web::Data<Client>) -> Result<HttpResponse, HTTPError> {
    let body = &serde_json::json!({
        "jsonrpc": "2.0",
        "method": "net_version",
        "params": [],
        "id": 1
    });
    let res = build_request_with_body_and_chain_name("polygon", HTTPRequestMethod::POST, body)
        .await
        .send()
        .await
        .map_err(|_| HTTPError::Timeout)?;

    response::build_health_response(Option::from(res), serde_json::Value::Null).await
}

#[get("/osmosis")]
pub async fn osmosis_health() -> Result<HttpResponse, HTTPError> {
    let mut client = get_bank_grpc_client("osmosis").await;

    let request = tonic::Request::new(QueryBalanceRequest {
        address: "osmo18tduqdmp2hrk4avkyuu8eyl8uuq4vgjrc02rfq".to_string(),
        denom: "uosmo".to_string(),
    });

    let response = client
        .balance(request)
        .await
        .map(|r| r.into_inner())
        .map_err(|e| HTTPError::Timeout)?;

    // // Querying for a balance might fail, i.e. if the account doesn't actually exist
    let balance = response.balance.ok_or_else(|| HTTPError::BadRequest)?;
    // balance to Value
    let balance_value = serde_json::to_value(balance).unwrap();

    response::build_health_response(None, balance_value).await
}

#[cfg(test)]
mod tests {
    use crate::http::response::HealthResponse;
    use actix_web::web::Data;
    use actix_web::{body::to_bytes, dev::Service, http, middleware, test, web, App, Error};

    use super::*;

    #[actix_web::test]
    async fn test_polygon_health() -> Result<(), Error> {
        let polygon_controller = web::scope("/health")
            .app_data(Data::new(reqwest::Client::new()))
            .service(polygon_health);
        let app = App::new()
            .wrap(middleware::Logger::default())
            .service(polygon_controller);
        let app = test::init_service(app).await;
        let req = test::TestRequest::get().uri("/health/polygon").to_request();
        let resp = app.call(req).await?;

        assert_eq!(resp.status(), StatusCode::OK);

        let response_body = resp.into_body();
        let data: HealthResponse = serde_json::from_slice(&to_bytes(response_body).await?)?;
        assert_eq!(data.status, StatusCode::OK);
        assert_eq!(data.message, "now working");
        assert!(data.data.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn test_osmosis_health() -> Result<(), Error> {
        let health_controller = web::scope("/health")
            .app_data(Data::new(reqwest::Client::new()))
            .service(osmosis_health);
        let app = App::new()
            .wrap(middleware::Logger::default())
            .service(health_controller);
        let app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/health/osmosis").to_request();
        let resp = app.call(req).await?;

        assert_eq!(resp.status(), StatusCode::OK);

        let response_body = resp.into_body();
        let data: HealthResponse = serde_json::from_slice(&to_bytes(response_body).await?).unwrap();
        assert_eq!(data.status, 200);
        assert_eq!(data.message, "now working");
        assert!(data.data.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn test_evmos_health() -> Result<(), Error> {
        let evmos_controller = web::scope("/health")
            .app_data(Data::new(Client::new()))
            .service(evmos_health);
        let app = App::new()
            .wrap(middleware::Logger::default())
            .service(evmos_controller);
        let app = test::init_service(app).await;
        let req = test::TestRequest::get().uri("/health/evmos").to_request();
        let resp = app.call(req).await?;

        assert_eq!(resp.status(), StatusCode::OK);

        let response_body = resp.into_body();
        let data: HealthResponse = serde_json::from_slice(&to_bytes(response_body).await?).unwrap();
        assert_eq!(data.status, 200);
        assert_eq!(data.message, "now working");
        assert!(data.data.is_some());

        Ok(())
    }
}
