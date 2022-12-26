use serde::{Deserialize, Serialize};
use ibc_proto::cosmos::bank::v1beta1::QueryBalanceResponse;
use actix_web::HttpResponse;
use reqwest::Response;
use actix_web::http::StatusCode;
use serde_json::map::Values;
use serde_json::Value;
use crate::http::error::HTTPError;

#[derive(Serialize, Deserialize)]
pub(crate) struct HealthResponse {
    pub(crate) status: u16,
    pub(crate) message: String,
    pub(crate) data: Option<Value>,
}

pub async fn build_health_response(res: Option<Response>, val: Value) -> Result<HttpResponse, HTTPError> {
    // TODO: refactor - make sedre_json to be possible at the method level, not from outside
    if !val.is_null() {
        let response = HealthResponse {
            status: 200,
            message: "now working".to_string(),
            data: Some(val)
        };

        Ok(HttpResponse::Ok().json(response))
    } else if let Some(res) = res {
        if res.status().is_success() {
            let body = res.text().await.map_err(|_| HTTPError::Timeout)?;
            let data: serde_json::Value = serde_json::from_str(&body).map_err(|_| HTTPError::BadRequest)?;
            let ok_response = HealthResponse {
                status: 200,
                message: "now working".to_string(),
                data: Some(data),
            };
            Ok(HttpResponse::Ok().json(ok_response))
        } else {
            let err_response = HealthResponse {
                status: res.status().as_u16(),
                message: "service is not working well".to_string(),
                data: None,
            };

            let response = HttpResponse::build(
                StatusCode::from_u16(err_response.status).unwrap()
            ).json(err_response);

            Ok(response)
        }
    } else {
        let err_response = HealthResponse {
            status: 500,
            message: "service is not working well".to_string(),
            data: None,
        };

        let response = HttpResponse::build(
            StatusCode::from_u16(err_response.status).unwrap()
        ).json(err_response);

        Ok(response)
    }
}
