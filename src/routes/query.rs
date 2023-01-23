use crate::client::factory::{
    build_request_by_chain_name, build_request_with_body_and_chain_name, get_bank_grpc_client,
    get_tx_grpc_client,
};
use crate::http::error::HTTPError;
use crate::http::method::HTTPRequestMethod;
use crate::http::response;
use crate::http::response::HealthResponse;
use crate::routes::message::{
    DumpMessageType, HodlMessageType, IndetermineMessageType, MessageType,
};
use crate::routes::sonar;
use crate::routes::sonar::SonarOsmosisResponse;
use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Responder};
use ibc_proto::cosmos::bank::v1beta1::{
    query_client::QueryClient, QueryAllBalancesRequest, QueryBalanceRequest, QueryBalanceResponse,
};
use ibc_proto::cosmos::tx::v1beta1::{GetTxRequest, GetTxResponse, Tx, TxBody};
use ibc_proto::ibc::core::channel::v1::acknowledgement::Response::Error;
use reqwest::{Client, Response, Version};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::borrow::Borrow;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use tonic::codegen::Body;

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
    // then filter & create new map which contains the type of the message contains the enum message type of `DumpMessageType` and `HodlMessageType`
    let mut dump_messages: HashMap<String, Vec<&sonar::Tx>> = HashMap::new();
    let mut hodl_messages: HashMap<String, Vec<&sonar::Tx>> = HashMap::new();
    let mut indetermine_messages: HashMap<String, Vec<&sonar::Tx>> = HashMap::new();
    // sonar_response Txs :: order by time desc
    for tx in &sonar_response.Txs {
        // iterate each Message struct in the tx struct
        // if given message type name includes the enum message type of `DumpMessageType` and `HodlMessageType`, then create new map which contains the type of the message contains the enum message type of `DumpMessageType` and `HodlMessageType`
        // iterate tx.Messages iter()
        // let txHash = tx.TxHash.to_string();
        for im in IndetermineMessageType::iter() {
            if doesContainMessageType(&tx, &im) {
                indetermine_messages
                    .entry(im.to_string())
                    .or_insert(vec![])
                    .push(tx);
                let raw_tx_response = getTxRawByHash(&tx.TxHash.to_string(), &target_chain).await;
                if raw_tx_response.tx_response.is_none() {
                    break;
                }
                if !isSucceedTransaction(&raw_tx_response) {
                    continue;
                }
                let tx = raw_tx_response.tx.unwrap();
                let tx_body = tx.body.unwrap();
                if !isMsgSwapExactAmountIn(&tx_body) {
                    continue;
                }
                let msg = &tx_body.messages[0].value;
                // msg to utf8 with lossy
                // token in denom: 52F0A20F1C1801A248333B13DFC7C54CF4E0BF8E6E6180D6204E6A9495B64057
                // token out denom: 163ED10C9238616CFEDF905EDCB848203405876CDB221045A4BC0FD4BE9907F4
                let msg_str = String::from_utf8_lossy(&msg);
                if msg_str.contains(
                    "ibc/6AE98883D4D5D5FF9E50D7130F1305DA2FFA0C652D1DD9C123657C6B4EB2DF8A",
                ) {
                    println!("dump message: {}", msg_str);
                }
                println!("msg_str: {}", msg_str);
            }
        }

        for hm in HodlMessageType::iter() {
            if doesContainMessageType(tx, &hm) {
                hodl_messages
                    .entry(hm.to_string())
                    .or_insert(vec![])
                    .push(tx);
            }
        }

        for dm in DumpMessageType::iter() {
            if doesContainMessageType(tx, &dm) {
                dump_messages
                    .entry(dm.to_string())
                    .or_insert(vec![])
                    .push(tx);
            }
        }
    }

    Ok(HttpResponse::Ok().json(sonar_response))
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

fn isSucceedTransaction(raw_tx_response: &GetTxResponse) -> bool {
    let tx_response = raw_tx_response.clone().tx_response.unwrap();
    if tx_response.code != 0 {
        return false;
    }
    return true;
}

// getTxRawByHash(&tx.TxHash.to_string(), &target_chain).await;
// 4EC9A84419A45FE9379B4406B822B8563C16D1EBA53B3C0639A68A5E550797A9
async fn getTxRawByHash(tx_hash: &str, target_chain: &str) -> GetTxResponse {
    // grpcurl -plaintext -d '{"hash": "4EC9A84419A45FE9379B4406B822B8563C16D1EBA53B3C0639A68A5E550797A9"}' grpc.osmosis.zone:9090 cosmos.tx.v1beta1.Service/GetTx
    let mut client = get_tx_grpc_client(target_chain).await;
    let request = tonic::Request::new(GetTxRequest {
        hash: tx_hash.to_string(),
    });

    let response = client.get_tx(request).await;
    return match response {
        Ok(response) => {
            let tx_response = response.into_inner();
            tx_response
        }
        Err(e) => {
            println!("Error getting transaction: {}", e);
            GetTxResponse {
                tx_response: None,
                tx: None,
            }
        }
    };
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
