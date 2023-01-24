use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Tx {
    pub(crate) TxHash: String,
    Success: bool,
    Height: String,
    Timestamp: String,
    Sender: String,
    MessageCount: u64,
    UsedGas: String,
    WantedGas: String,
    Fee: Vec<Fee>,
    Memo: String,
    pub(crate) Messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Fee {
    amount: String,
    denom: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Message {
    Module: String,
    pub(crate) Type: String,
    TxHash: String,
    Json: String,
    Success: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SonarOsmosisResponse {
    Page: u64,
    PerPage: u64,
    pub(crate) Txs: Vec<Tx>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CustomTxResponse {
    /// The block height
    pub height: String,
    /// The transaction hash.
    pub txhash: String,
    /// Namespace for the Code
    pub codespace: String,
    /// Response code.
    pub code: u32,
    /// Result bytes, if any.
    pub data: String,
    /// The output of the application's logger (raw string). May be
    /// non-deterministic.
    pub raw_log: String,
    /// The output of the application's logger (typed). May be non-deterministic.
    // #[prost(message, repeated, tag="7")]
    // pub logs: ::prost::alloc::vec::Vec<AbciMessageLog>,
    /// Additional information. May be non-deterministic.
    pub info: String,
    /// Amount of gas requested for transaction.
    pub gas_wanted: String,
    /// Amount of gas consumed by transaction.
    pub gas_used: String,
    /// The request transaction bytes.
    // #[prost(message, optional, tag="11")]
    // pub tx: ::core::option::Option<super::super::super::super::google::protobuf::Any>,
    /// Time of the previous block. For heights > 1, it's the weighted median of
    /// the timestamps of the valid votes in the block.LastCommit. For height == 1,
    /// it's genesis time.
    pub timestamp: String,
    /// Events defines all the events emitted by processing a transaction. Note,
    /// these events include those emitted by processing all the messages and those
    /// emitted from the ante. Whereas Logs contains the events, with
    /// additional metadata, emitted only by processing the messages.
    ///
    /// Since: cosmos-sdk 0.42.11, 0.44.5, 0.45
    pub events: Vec<CustomEvent>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct CustomEvent {
    pub r#type: String,
    pub attributes: Vec<CustomEventAttribute>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct CustomEventAttribute {
    pub key: String,
    pub value: String,
    pub index: bool,
}
