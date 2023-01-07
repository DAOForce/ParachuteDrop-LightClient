use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
struct Fee {
    amount: String,
    denom: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    Module: String,
    pub(crate) Type: String,
    TxHash: String,
    Json: String,
    Success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SonarOsmosisResponse {
    Page: u64,
    PerPage: u64,
    pub(crate) Txs: Vec<Tx>,
}
