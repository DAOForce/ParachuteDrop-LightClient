use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Tx {
    TxHash: String,
    Success: bool,
    Height: String,
    Timestamp: String,
    Sender: String,
    MessageCount: u64,
    UsedGas: String,
    WantedGas: String,
    Fee: Vec<Fee>,
    Memo: String,
    Messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Fee {
    amount: String,
    denom: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    Module: String,
    Type: String,
    TxHash: String,
    Json: String,
    Success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SonarOsmosisResponse {
    TotalTxs: u64,
    TotalSuccessTxs: u64,
    TotalFailureTxs: u64,
    Page: u64,
    PerPage: u64,
    SuccessOnly: bool,
    FailureOnly: bool,
    Txs: Vec<Tx>,
}
