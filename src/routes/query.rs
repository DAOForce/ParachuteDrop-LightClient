use actix_web::{get, HttpResponse, Responder, web};
use actix_web::http::StatusCode;
use ibc_proto::cosmos::bank::v1beta1::{query_client::QueryClient, QueryAllBalancesRequest, QueryBalanceRequest, QueryBalanceResponse};
use ibc_proto::ibc::core::channel::v1::acknowledgement::Response::Error;
use reqwest::{Client, Response, Version};
use tonic::codegen::Body;
use crate::client::factory::{build_request_by_chain_name, build_request_with_body_and_chain_name, get_bank_grpc_client};
use crate::http::error::HTTPError;
use crate::http::method::HTTPRequestMethod;
use crate::http::response;
use crate::http::response::HealthResponse;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueryChainTokenBalance {
    target_chain: String,
    token_denom: String,
    address: String,
}

// check the balance of evmos in given account address
#[get("/balance")]
pub async fn query_balance(info: web::Query<QueryChainTokenBalance>) -> Result<HttpResponse, HTTPError> {
    // this function checks the balance of the token_denom that address holds in the target_chain
    // using the grpc client or http client
    let mut client = get_bank_grpc_client(&info.target_chain).await;
    let target_chain = info.target_chain.clone();
    let token_denom = info.token_denom.clone();
    let address = info.address.clone();

    println!("target_chain: {}", target_chain);
    println!("token_denom: {}", token_denom);
    println!("address: {}", address);

    let request = build_request_with_body_and_chain_name(&target_chain, HTTPRequestMethod::POST, &serde_json::json!({
        "jsonrpc": "2.0",
        "method": "net_version",
        "params": [],
        "id": 1
    })).await.send().await.map_err(|_| HTTPError::Timeout)?;

    // return the response
    response::build_health_response(Option::from(request), serde_json::Value::Null).await
}