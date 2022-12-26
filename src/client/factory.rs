use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use ibc_proto::cosmos::bank::v1beta1::{query_client::QueryClient, QueryAllBalancesRequest, QueryBalanceRequest, QueryBalanceResponse};
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::process::Output;
use actix_web::web;
use ibc_proto::ibc::core::channel::v1::Channel;
use reqwest::{Client, RequestBuilder, Response};
use tokio::task::JoinSet;
use crate::http::error::HTTPError;

#[derive(Serialize,Deserialize, Debug, Clone, PartialEq)]
pub struct SupportedBlockchain {
    pub name: String,
    pub prefix: String,
    pub rest_url: Option<String>,
    pub grpc_url: Option<String>,
}

impl SupportedBlockchain {
    pub async fn get_bank_grpc_client(&self) -> QueryClient<tonic::transport::Channel> {
        match &self.grpc_url {
            None => panic!("Error: {:?} is not a supported grpc cosmos blockchain!", self.name),
            Some(grpc_url) => QueryClient::connect(grpc_url.to_owned()).await.unwrap(),
        }
    }

    pub async fn get_lcd_post_request_builder_with_json(
        &self, client: web::Data<Client>, body: serde_json::Value
    ) -> RequestBuilder {
        match &self.rest_url {
            None => panic!("Error: {:?} is not a supported lcd cosmos blockchain!", self.name),
            Some(rest_url) => client.post(rest_url.to_owned()).json(&body),
        }
    }

    pub async fn get_lcd_get_request_builder(&self, client: web::Data<Client>) -> RequestBuilder {
        match &self.rest_url {
            None => panic!("Error: {:?} is not a supported lcd cosmos blockchain!", self.name),
            Some(rest_url) => client.get(rest_url.to_owned()),
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
            rest_url: Some("https://rest.bd.evmos.org:1317/node_info".to_string()),
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
        }
    );
    supported_blockchains
}

pub async fn get_bank_grpc_client(name: &str) -> QueryClient<tonic::transport::Channel> {
    let supported_blockchains = get_supported_blockchains();
    let blockchain = supported_blockchains.get(name).unwrap();
    blockchain.get_bank_grpc_client().await
}

pub async fn get_lcd_request_builder_by_chain_name(chain_name: &str) -> RequestBuilder {
    let supported_blockchains = get_supported_blockchains();
    let blockchain = supported_blockchains.get(chain_name).unwrap();
    blockchain.get_lcd_get_request_builder(web::Data::new(Client::new())).await
}
