use crate::client::factory::{
    build_request_by_chain_name, build_request_to_explorer_api_by_chain_name_with_query_parameters,
    build_request_with_body_and_chain_name, get_bank_grpc_client, get_supported_blockchains,
    get_tx_grpc_client, SearchType,
};
use crate::http::error::HTTPError;
use crate::http::method::HTTPRequestMethod;
use crate::http::response;
use crate::http::response::HealthResponse;
use crate::routes;
use crate::routes::message::{
    DumpMessageType, EventType, HodlMessageType, IndetermineMessageType, MessageType,
};
use crate::routes::sonar;
use crate::routes::sonar::{CustomTxResponse, SonarOsmosisResponse};
use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder};
use base64::decode;
use ibc_proto::cosmos::bank::v1beta1::{
    query_client::QueryClient, QueryAllBalancesRequest, QueryBalanceRequest, QueryBalanceResponse,
};
use ibc_proto::cosmos::base::abci::v1beta1::TxResponse;
use ibc_proto::cosmos::tx::v1beta1::{GetTxRequest, GetTxResponse, Tx, TxBody};
use ibc_proto::ibc::core::channel::v1::acknowledgement::Response::Error;
use json::JsonValue;
use prost::Message;
use reqwest::{Client, Response, Version};
use serde::{Deserialize, Serialize};
use serde_json::from_value;
use serde_json::{from_str, Value};
use std::any::Any;
use std::borrow::Borrow;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use tonic::codegen::Body;

// HashMap<String, Vec<&parachutedrop_rust_server::routes::sonar::Tx>
pub type processed_message_type = HashMap<String, Vec<sonar::Tx>>;

#[derive(Deserialize)]
pub struct QueryChainTokenBalance {
    target_chain: String,
    token_denom: String,
    address: String,
}

#[derive(Serialize)]
pub struct ChainTokenBalanceResponse {
    address: String,
    token_name: String,
    amount: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HodlDumpMessageResponse {
    queried_lastest_tx_amount: u32,
    hodl_message: processed_message_type,
    dump_message: processed_message_type,
}

#[get("/message")]
pub async fn track_messages(
    info: web::Query<QueryChainTokenBalance>,
) -> Result<HttpResponse, HTTPError> {
    let target_chain = info.target_chain.clone();
    let token_denom = info.token_denom.clone();
    let address = info.address.clone();

    let http_client = reqwest::Client::new();
    let sonar_api = format!(
        "https://api.sonarpod.com/osmosis/account/{}/transactions?per_page=20&page=1",
        address
    );
    let sonar_request = http_client
        .get(sonar_api)
        .send()
        .await
        .map_err(|_| HTTPError::Timeout)?;
    let sonar_response_body = sonar_request.text().await.map_err(|_| HTTPError::Timeout)?;
    let sonar_response: SonarOsmosisResponse =
        from_str(&sonar_response_body).map_err(|_| HTTPError::InternalError)?;

    // iterate each Tx struct in the sonar_response.
    // then filter & create new map which contains the type of the message
    // contains the enum message type of `DumpMessageType` and `HodlMessageType`
    let mut dump_messages: HashMap<String, Vec<sonar::Tx>> = HashMap::new();
    let mut hodl_messages: HashMap<String, Vec<sonar::Tx>> = HashMap::new();
    let mut indetermine_messages: HashMap<String, Vec<&sonar::Tx>> = HashMap::new();
    // sonar_response Txs :: order by time desc
    for tx in &sonar_response.Txs {
        // iterate each Message struct in the tx struct
        // if given message type name includes the enum message type of `DumpMessageType` and `HodlMessageType`,
        // then create new map which contains the type of the message
        // contains the enum message type of `DumpMessageType` and `HodlMessageType`
        for im in IndetermineMessageType::iter() {
            if doesContainMessageType(&tx, &im) {
                indetermine_messages
                    .entry(im.to_string())
                    .or_insert(vec![])
                    .push(tx);

                let response = getTxRawByHash(&tx.TxHash.to_string(), &target_chain).await;
                let unwrapped_response = response.clone();
                let unwrapped_response = match unwrapped_response {
                    Ok(response) => response,
                    Err(_) => continue,
                };
                if unwrapped_response.tx_response.is_none() {
                    continue;
                }

                let tx_response = response.clone().unwrap().tx_response.unwrap();
                if !isSucceedTransaction(&tx_response) {
                    continue;
                }

                let supported_blockchains = get_supported_blockchains();
                let cosmos_blockchains = supported_blockchains.get(&token_denom).unwrap();
                let cosmos_token_denom = cosmos_blockchains.clone().cosmos_token_denom;
                let cosmos_blockchain_denom = cosmos_token_denom.clone().unwrap_or_default();

                let events = tx_response.clone().events;
                for event in events {
                    if event.r#type == EventType::TokenSwapped.to_string() {
                        for attribute in &event.attributes {
                            if attribute.key == "pool_id" && attribute.value == "722" {
                                for attribute in &event.attributes {
                                    if attribute.value.contains(&cosmos_blockchain_denom) {
                                        if attribute.key == "tokens_in" {
                                            dump_messages
                                                .entry(im.to_string())
                                                .or_insert(vec![])
                                                .push(tx.clone());
                                        } else if attribute.key == "tokens_out" {
                                            hodl_messages
                                                .entry(im.to_string())
                                                .or_insert(vec![])
                                                .push(tx.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for hm in HodlMessageType::iter() {
            if doesContainMessageType(tx, &hm) {
                let response = getTxRawByHash(&tx.TxHash.to_string(), &target_chain).await;
                let unwrapped_response = response.clone();
                let unwrapped_response = match unwrapped_response {
                    Ok(response) => response,
                    Err(_) => continue,
                };
                if unwrapped_response.tx_response.is_none() {
                    continue;
                }

                let tx_response = response.clone().unwrap().tx_response.unwrap();
                if !isSucceedTransaction(&tx_response) {
                    continue;
                }

                let supported_blockchains = get_supported_blockchains();
                let cosmos_blockchains = supported_blockchains.get(&token_denom).unwrap();
                let cosmos_token_denom = cosmos_blockchains.clone().cosmos_token_denom;
                let cosmos_blockchain_denom = cosmos_token_denom.clone().unwrap_or_default();

                let events = tx_response.clone().events;
                for event in events {
                    if event.r#type == EventType::PoolJoined.to_string() {
                        for attribute in &event.attributes {
                            if attribute.key == "pool_id" && attribute.value == "722" {
                                for attribute in &event.attributes {
                                    if attribute.value.contains(&cosmos_blockchain_denom) {
                                        hodl_messages
                                            .entry(hm.to_string())
                                            .or_insert(vec![])
                                            .push(tx.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for dm in DumpMessageType::iter() {
            if doesContainMessageType(tx, &dm) {
                let response = getTxRawByHash(&tx.TxHash.to_string(), &target_chain).await;
                let unwrapped_response = response.clone();
                let unwrapped_response = match unwrapped_response {
                    Ok(response) => response,
                    Err(_) => continue,
                };
                if unwrapped_response.tx_response.is_none() {
                    continue;
                }

                let tx_response = response.clone().unwrap().tx_response.unwrap();
                if !isSucceedTransaction(&tx_response) {
                    continue;
                }

                let supported_blockchains = get_supported_blockchains();
                let cosmos_blockchains = supported_blockchains.get(&token_denom).unwrap();
                let cosmos_token_denom = cosmos_blockchains.clone().cosmos_token_denom;
                let cosmos_blockchain_denom = cosmos_token_denom.clone().unwrap_or_default();

                let events = tx_response.clone().events;
                for event in events {
                    if event.r#type == EventType::PoolExited.to_string() {
                        for attribute in &event.attributes {
                            if attribute.key == "pool_id" && attribute.value == "722" {
                                for attribute in &event.attributes {
                                    if attribute.value.contains(&cosmos_blockchain_denom) {
                                        dump_messages
                                            .entry(dm.to_string())
                                            .or_insert(vec![])
                                            .push(tx.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("dump_messages: {:?}", dump_messages);
    println!("hodl_messages: {:?}", hodl_messages);

    let response = HodlDumpMessageResponse {
        queried_lastest_tx_amount: sonar_response.Txs.len() as u32,
        hodl_message: hodl_messages.clone(),
        dump_message: dump_messages.clone(),
    };

    Ok(HttpResponse::Ok().json(response))
}

fn isMsgSwapExactAmountIn(tx_body: &TxBody) -> bool {
    let tx_messages = tx_body.messages.get(0);
    let tx_message = tx_messages.unwrap();
    let typeUrl = tx_message.type_url.clone();
    return if !typeUrl.contains("SwapExactAmountIn") {
        false
    } else {
        true
    };
}

fn isSucceedTransaction(raw_tx_response: &CustomTxResponse) -> bool {
    let tx_response = raw_tx_response.clone();
    if tx_response.code != 0 {
        return false;
    }
    return true;
}

#[derive(Deserialize, Debug)]
struct IntermediateGetTxResponse {
    tx_response: Option<Value>,
    tx: Option<Value>,
}

#[derive(Clone, PartialEq)]
pub struct CustomGetTxResponse {
    pub tx_response: Option<CustomTxResponse>,
}

// getTxRawByHash(&tx.TxHash.to_string(), &target_chain).await;
// 4EC9A84419A45FE9379B4406B822B8563C16D1EBA53B3C0639A68A5E550797A9
async fn getTxRawByHash(tx_hash: &str, target_chain: &str) -> Result<CustomGetTxResponse, String> {
    let supported_blockchains = get_supported_blockchains();
    let osmosis = supported_blockchains.get("osmosis").unwrap();
    let search_type = SearchType::new(osmosis, &tx_hash);
    let response = build_request_to_explorer_api_by_chain_name_with_query_parameters(
        HTTPRequestMethod::GET,
        &search_type,
        &tx_hash,
    )
    .await;

    let json_str = match response {
        Ok(response) => response.text().await.map_err(|e| e.to_string())?,
        Err(e) => return Err(e.to_string()),
    };

    let deserialized: IntermediateGetTxResponse =
        serde_json::from_str(&*json_str).map_err(|e| {
            println!("json_str: {}", json_str);
            println!("deserialized error: {}", e.to_string());
            e.to_string()
        })?;

    let tx_response = match deserialized.tx_response {
        Some(s) => {
            let json_str = serde_json::to_string(&s).unwrap();
            let mut custom_tx_response = serde_json::from_str(&json_str);
            let mut custom_tx_response: CustomTxResponse = match custom_tx_response {
                Ok(custom_tx_response) => custom_tx_response,
                Err(e) => {
                    println!("json_str: {}", json_str);
                    println!("deserialized error: {}", e.to_string());
                    return Err(e.to_string());
                }
            };
            for event in &mut custom_tx_response.events {
                for attribute in &mut event.attributes {
                    let decoded_key = decode(&attribute.key).unwrap();
                    let decoded_value = decode(&attribute.value).unwrap();
                    let decoded_key_string = String::from_utf8(decoded_key).unwrap();
                    let decoded_value_string = String::from_utf8(decoded_value).unwrap();
                    attribute.key = decoded_key_string;
                    attribute.value = decoded_value_string;
                }
            }
            Ok(custom_tx_response)
        }
        None => {
            println!("tx_response is none");
            Err("tx_response is none".to_string())
        }
    };

    // tx_response from Result<TxResponse, String> to Option<TxResponse>
    let tx_response: Option<CustomTxResponse> = match tx_response {
        Ok(tx_response) => Some(tx_response),
        Err(e) => return Err(e.to_string()),
    };

    Ok(CustomGetTxResponse { tx_response })
}

fn doesContainMessageType(tx: &sonar::Tx, msg: &dyn MessageType) -> bool {
    tx.Messages.iter().any(|m| m.Type == msg.to_string())
}

// check the balance of evmos in given account address
#[get("/balance")]
pub async fn query_balance(
    info: web::Query<QueryChainTokenBalance>,
) -> Result<HttpResponse, HTTPError> {
    // TODO: search appropriate lcd or grpc request by searching chain name
    let mut client = get_bank_grpc_client(&info.target_chain).await;
    let target_chain = info.target_chain.clone();
    let token_denom = info.token_denom.clone();
    let address = info.address.clone();

    let denom = match token_denom.as_str() {
        "evmos" => {
            "ibc/6AE98883D4D5D5FF9E50D7130F1305DA2FFA0C652D1DD9C123657C6B4EB2DF8A".to_string()
        }
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

    // convert response to ChainTokenBalanceResponse
    let balance = response.balance.ok_or_else(|| HTTPError::BadRequest)?;
    let amount = balance.amount.parse::<f64>().unwrap() / f64::powf(10.0, 18.0);

    let chain_token_balance_response = ChainTokenBalanceResponse {
        address: address.clone(),
        token_name: token_denom,
        amount,
    };

    // return with the response
    Ok(HttpResponse::Ok().json(chain_token_balance_response))
}

#[cfg(test)]
mod tests {
    use crate::http::response::HealthResponse;
    use actix_web::web::Data;
    use actix_web::{body::to_bytes, dev::Service, http, middleware, test, web, App, Error};

    use super::*;

    #[actix_web::test]
    async fn test_query_balance() -> Result<(), Error> {
        // given: http://localhost:8080/query/balance?target_chain=osmosis&token_denom=evmos&address=osmo1083svrca4t350mphfv9x45wq9asrs60cq5yv9n
        let query_controller = web::scope("/query").service(query_balance);
        let mut app = test::init_service(
            App::new()
                .service(query_controller)
                .wrap(middleware::Logger::default()),
        )
        .await;

        // when: make sure that the endpoint calls query_balance
        let req = test::TestRequest::get()
           .uri("/query/balance?target_chain=osmosis&token_denom=evmos&address=osmo1083svrca4t350mphfv9x45wq9asrs60cq5yv9n")
              .to_request();
        let resp = app.call(req).await.unwrap();

        // then: the response is the following
        // {
        //     "address": "osmo1083svrca4t350mphfv9x45wq9asrs60cq5yv9n",
        //     "token_name": "evmos",
        //     "amount": 6.856597348646046
        // }
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body = to_bytes(resp.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("{}", body_str);

        assert_eq!(
            body_str,
            r#"{"address":"osmo1083svrca4t350mphfv9x45wq9asrs60cq5yv9n","token_name":"evmos","amount":6.856597348646046}"#
        );
        Ok(())
    }
}
