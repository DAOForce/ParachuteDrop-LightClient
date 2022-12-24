use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct HealthResponse {
    pub(crate) status: u16,
    pub(crate) message: String,
    pub(crate) data: Option<serde_json::Value>,
}
