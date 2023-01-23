use actix_web::{
    error, get,
    http::{header::ContentType, StatusCode},
    App, HttpResponse,
};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum HTTPError {
    InternalError,
    BadRequest,
    Timeout,
}

impl Display for HTTPError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            HTTPError::InternalError => write!(f, "internal error"),
            HTTPError::BadRequest => write!(f, "bad request"),
            HTTPError::Timeout => write!(f, "timeout"),
        }
    }
}

impl error::ResponseError for HTTPError {
    fn status_code(&self) -> StatusCode {
        match *self {
            HTTPError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            HTTPError::BadRequest => StatusCode::BAD_REQUEST,
            HTTPError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}
