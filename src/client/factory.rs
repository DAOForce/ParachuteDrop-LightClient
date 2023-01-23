use crate::http::error::HTTPError;
use crate::http::method::HTTPRequestMethod;
use actix_web::web;
use ibc_proto::cosmos::bank::v1beta1::{
    query_client::QueryClient, QueryAllBalancesRequest, QueryBalanceRequest, QueryBalanceResponse,
};
use ibc_proto::ibc::core::channel::v1::Channel;
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::process::Output;
use tokio::task::JoinSet;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SupportedBlockchain {
    pub name: String,
    pub prefix: String,
    pub rest_url: Option<String>,
    pub grpc_url: Option<String>,
}

impl SupportedBlockchain {
    pub async fn get_bank_grpc_client(&self) -> QueryClient<tonic::transport::Channel> {
        match &self.grpc_url {
            None => panic!(
                "Error: {:?} is not a supported grpc cosmos blockchain!",
                self.name
            ),
            Some(grpc_url) => QueryClient::connect(grpc_url.to_owned()).await.unwrap(),
        }
    }

    pub async fn get_tx_grpc_client(
        &self,
    ) -> ibc_proto::cosmos::tx::v1beta1::service_client::ServiceClient<tonic::transport::Channel>
    {
        match &self.grpc_url {
            None => panic!(
                "Error: {:?} is not a supported grpc cosmos blockchain!",
                self.name
            ),
            Some(grpc_url) => {
                ibc_proto::cosmos::tx::v1beta1::service_client::ServiceClient::connect(
                    grpc_url.to_owned(),
                )
                .await
                .unwrap()
            }
        }
    }

    pub async fn get_lcd_request_builder_by_chain_name(
        &self,
        request_type: HTTPRequestMethod,
        client: web::Data<Client>,
    ) -> RequestBuilder {
        match &self.rest_url {
            None => panic!(
                "Error: {:?} is not a supported lcd cosmos blockchain!",
                self.name
            ),
            Some(rest_url) => {
                if request_type == HTTPRequestMethod::GET {
                    client.get(rest_url.to_owned())
                } else if request_type == HTTPRequestMethod::POST {
                    client.post(rest_url.to_owned())
                } else if request_type == HTTPRequestMethod::PUT {
                    client.put(rest_url.to_owned())
                } else if request_type == HTTPRequestMethod::DELETE {
                    client.delete(rest_url.to_owned())
                } else {
                    panic!("Error is not a supported http request type!")
                }
            }
        }
    }
}

pub fn get_supported_blockchains() -> HashMap<String, SupportedBlockchain> {
    let mut supported_blockchains: HashMap<String, SupportedBlockchain> = HashMap::new();
    supported_blockchains.insert(
        "evmos".to_string(),
        SupportedBlockchain {
            name: "Evmos".to_string(),
            prefix: "evmos".to_string(),
            rest_url: Some("https://rest.bd.evmos.org:1317".to_string()),
            grpc_url: None,
        },
    );
    supported_blockchains.insert(
        "polygon".to_string(),
        SupportedBlockchain {
            name: "Polygon".to_string(),
            prefix: "polygon".to_string(),
            rest_url: Some("https://polygon-mainnet-rpc.allthatnode.com:8545/".to_string()),
            grpc_url: None,
        },
    );
    supported_blockchains.insert(
        "osmosis".to_string(),
        SupportedBlockchain {
            name: "Osmosis".to_string(),
            prefix: "osmosis".to_string(),
            rest_url: None,
            grpc_url: Some("https://grpc.osmosis.zone:9090/".to_string()),
        },
    );
    supported_blockchains
}

pub async fn get_bank_grpc_client(name: &str) -> QueryClient<tonic::transport::Channel> {
    let supported_blockchains = get_supported_blockchains();
    let blockchain = supported_blockchains.get(name).unwrap();
    blockchain.get_bank_grpc_client().await
}

pub async fn get_tx_grpc_client(
    name: &str,
) -> ibc_proto::cosmos::tx::v1beta1::service_client::ServiceClient<tonic::transport::Channel> {
    let supported_blockchains = get_supported_blockchains();
    let blockchain = supported_blockchains.get(name).unwrap();
    blockchain.get_tx_grpc_client().await
}

pub async fn build_request_by_chain_name(
    chain_name: &str,
    method: HTTPRequestMethod,
) -> RequestBuilder {
    let supported_blockchains = get_supported_blockchains();
    let blockchain = supported_blockchains.get(chain_name).unwrap();
    blockchain
        .get_lcd_request_builder_by_chain_name(method, web::Data::new(Client::new()))
        .await
}

pub async fn build_request_with_body_and_chain_name(
    chain_name: &str,
    method: HTTPRequestMethod,
    body: &Value,
) -> RequestBuilder {
    let supported_blockchains = get_supported_blockchains();
    let blockchain = supported_blockchains.get(chain_name).unwrap();
    blockchain
        .get_lcd_request_builder_by_chain_name(method, web::Data::new(Client::new()))
        .await
        .json(body)
}
