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
    // TODO: search appropriate lcd or grpc request by searching chain name
    let mut client = get_bank_grpc_client(&info.target_chain).await;
    let target_chain = info.target_chain.clone();
    let token_denom = info.token_denom.clone();
    let address = info.address.clone();

    let denom = match token_denom.as_str() {
        "evmos" => "ibc/6AE98883D4D5D5FF9E50D7130F1305DA2FFA0C652D1DD9C123657C6B4EB2DF8A".to_string(),
        _ => return Err(HTTPError::BadRequest),
    };

    let request = tonic::Request::new(QueryBalanceRequest {
        address: address.clone(),
        denom: denom.clone(),
    });

    println!("target_chain: {}", &target_chain);
    println!("token_denom: {}", &token_denom);
    println!("address: {}", &address);

    let response = client
        .balance(request)
        .await
        .map(|r| r.into_inner())
        .map_err(|e| HTTPError::Timeout)?;

    // return with the response
    Ok(HttpResponse::Ok().json(response))
}