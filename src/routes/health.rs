use actix_web::{get, web, HttpResponse, Responder};
use actix_web::http::StatusCode;
use cosmos_sdk_proto::cosmos::bank::v1beta1::query_client::QueryClient;
use cosmos_sdk_proto::cosmos::bank::v1beta1::QueryAllBalancesRequest;
use cosmos_sdk_proto::cosmos::base::query::v1beta1::PageRequest;
use reqwest::{Client, Response, Version};
use crate::http::error::HTTPError;
use crate::http::response::HealthResponse;

#[get("/evmos")]
pub async fn evmos_health() -> Result<HttpResponse, HTTPError> {
    // grpcurl -d '{"'address'": "'$EVMOS_ADDR_3'"}' $NODE cosmos.bank.v1beta1.Query/AllBalances
    let endpoint = tonic::transport::Endpoint::new("https://grpc.evmos.nodestake.top"
            .parse::<tonic::transport::Uri>().unwrap())
            .unwrap();
    let channel = endpoint.connect().await;
    let mut connected_client = QueryClient::new(channel.unwrap());
    let request = QueryAllBalancesRequest {
        address: "evmos1f57x5pm9wpvlu4qldy864j0hdnq025287r65z2".to_string(),
        pagination: Option::from(PageRequest {
            key: Vec::from("".to_string()),
            offset: 0,
            limit: 0,
            count_total: false,
            reverse: false
        }),
    };
    let response = connected_client.all_balances(request);
    let response2 = response.await;

    Ok(HttpResponse::Ok().json(HealthResponse {
        status: StatusCode::OK.as_u16(),
        message: "OK".to_string(),
        data: None,
    }))
}

#[get("/polygon")]
pub async fn polygon_health(client: web::Data<Client>) -> Result<HttpResponse, HTTPError> {
    let res = client
        .post("https://polygon-mainnet-rpc.allthatnode.com:8545/")
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1
        }))
        .send()
        .await
        .map_err(|e| HTTPError::Timeout)?;

    build_health_response(res).await
}

async fn build_health_response(res: Response) -> Result<HttpResponse, HTTPError> {
    if res.status().is_success() {
        let body = res.text().await.map_err(|_| HTTPError::Timeout)?;
        let data: serde_json::Value = serde_json::from_str(&body).map_err(|_| HTTPError::BadRequest)?;
        let ok_response = HealthResponse {
            status: 200,
            message: "now working".to_string(),
            data: Some(data),
        };
        Ok(HttpResponse::Ok().json(ok_response))
    } else {
        let err_response = HealthResponse {
            status: res.status().as_u16(),
            message: "service is not working well".to_string(),
            data: None,
        };

        let response = HttpResponse::build(
            StatusCode::from_u16(err_response.status).unwrap()
        ).json(err_response);

        Ok(response)
    }
}

#[get("/osmosis")]
pub async fn osmosis_health(client: web::Data<Client>) -> Result<HttpResponse, HTTPError> {
    let res = client
        .get("https://osmosis-mainnet-rpc.allthatnode.com:1317/node_info")
        .send()
        .await
        .map_err(|_| HTTPError::Timeout)?;

    build_health_response(res).await
}

#[cfg(test)]
mod tests {
    use actix_web::{body::to_bytes, dev::Service, http, test, web, App, Error, middleware};
    use actix_web::web::Data;
    use crate::http::response::HealthResponse;

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

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = resp.into_body();
        let data: HealthResponse = serde_json::from_slice(&to_bytes(response_body).await?).unwrap();
        assert_eq!(data.status, 200);
        assert_eq!(data.message, "now working");
        assert!(data.data.is_some());

        Ok(())
    }

}
