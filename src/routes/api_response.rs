use std::fmt;
use std::string::ParseError;
use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError};
use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum TNRAppErrorType {
    DbError,
    MessageError,
    ForbiddenError,
    ParseError,
    NotFoundError,
    ValidationError,
}

#[derive(Debug)]
pub struct TNRAppError {
    pub message: Option<String>,
    pub error_type: TNRAppErrorType,
}

#[derive(Serialize)]
pub struct TNRAppErrorResponse {
    pub error: String,
}

impl TNRAppError {
    fn message(&self) -> String {
        match &*self {
            TNRAppError {
                message: Some(message),
                error_type: _,
            } => message.clone(),
            TNRAppError {
                message: None,
                error_type: TNRAppErrorType::NotFoundError,
            } => "The requested item was not found".to_string(),
            _ => "An unexpected error has occured".to_string(),
        }
    }
}

impl fmt::Display for TNRAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for TNRAppErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<ParseError> for TNRAppErrorType {
    fn from(_error: ParseError) -> TNRAppErrorType {
        TNRAppErrorType::NotFoundError
    }
}

impl From<String> for TNRAppError {
    fn from(error: String) -> TNRAppError {
        TNRAppError {
            message: Some(error.to_string()),
            error_type: TNRAppErrorType::MessageError,
        }
    }
}

impl ResponseError for TNRAppError {
    fn status_code(&self) -> StatusCode {
        match self.error_type {
            TNRAppErrorType::DbError => { StatusCode::INTERNAL_SERVER_ERROR }
            TNRAppErrorType::NotFoundError => { StatusCode::NOT_FOUND }
            TNRAppErrorType::ForbiddenError => { StatusCode::FORBIDDEN }
            TNRAppErrorType::ParseError => { StatusCode::INTERNAL_SERVER_ERROR }
            TNRAppErrorType::MessageError => { StatusCode::INTERNAL_SERVER_ERROR }
            TNRAppErrorType::ValidationError => { StatusCode::INTERNAL_SERVER_ERROR }
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(TNRAppErrorResponse {
            error: self.message()
        })
    }
}


#[derive(Serialize)]
pub struct TNRAppSuccessResponse<T> {
    pub data: T,
}

impl<T> TNRAppSuccessResponse<T> {
    pub fn new(data: T) -> Self <> {
        Self {
            data
        }
    }
}

impl<T: Serialize> Responder for TNRAppSuccessResponse<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();
        HttpResponse::Ok().insert_header(ContentType::json()).body(body)
    }
}
