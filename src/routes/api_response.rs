use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError};
use mongodb_cursor_pagination::CursorDirections;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::string::ParseError;

#[derive(Debug)]
pub enum TNRAppErrorType {
    // DbError,
    MessageError,
    // ForbiddenError,
    // ParseError,
    NotFoundError,
    // ValidationError,
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
        match self {
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
            message: Some(error),
            error_type: TNRAppErrorType::MessageError,
        }
    }
}

impl ResponseError for TNRAppError {
    fn status_code(&self) -> StatusCode {
        match self.error_type {
            // TNRAppErrorType::DbError => StatusCode::INTERNAL_SERVER_ERROR,
            TNRAppErrorType::NotFoundError => StatusCode::NOT_FOUND,
            // TNRAppErrorType::ForbiddenError => StatusCode::FORBIDDEN,
            // TNRAppErrorType::ParseError => StatusCode::INTERNAL_SERVER_ERROR,
            TNRAppErrorType::MessageError => StatusCode::INTERNAL_SERVER_ERROR,
            // TNRAppErrorType::ValidationError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(TNRAppErrorResponse { error: self.message() })
    }
}

#[derive(Serialize)]
pub struct TNRAppSuccessResponse<T> {
    pub data: T,
    pub pagination: Option<PaginationData>,
}

impl<T> TNRAppSuccessResponse<T> {
    pub fn new(data: T, pagination: Option<PaginationData>) -> Self {
        Self { data, pagination }
    }

    pub fn cursor(data: T, cursor: Option<String>, limit: u64, direction: Option<PaginationDirection>) -> Self {
        Self {
            data,
            pagination: Some(PaginationData {
                cursor,
                limit: Some(limit),
                direction: Some(direction.unwrap_or_default()),
                ..Default::default()
            }),
        }
    }

    pub fn offset(data: T, offset: u64, limit: u64, direction: Option<PaginationDirection>) -> Self {
        Self {
            data,
            pagination: Some(PaginationData {
                offset: Some(offset),
                limit: Some(limit),
                direction: Some(direction.unwrap_or_default()),
                ..Default::default()
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaginationDataQueryParams {
    pub cursor: Option<String>,
    pub offset: Option<String>,
    pub limit: Option<String>,
    pub direction: Option<PaginationDirection>,
}

impl From<PaginationDataQueryParams> for PaginationData {
    fn from(value: PaginationDataQueryParams) -> Self {
        Self {
            cursor: value.cursor,
            offset: value.offset.map(|o| o.parse::<u64>().ok()).unwrap_or(None),
            limit: value.limit.map(|o| o.parse::<u64>().ok()).unwrap_or(None),
            direction: value.direction,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaginationData {
    pub cursor: Option<String>,
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub direction: Option<PaginationDirection>,
}

impl From<PaginationDirection> for CursorDirections {
    fn from(value: PaginationDirection) -> Self {
        match value {
            PaginationDirection::Next => CursorDirections::Next,
            PaginationDirection::Prev => CursorDirections::Previous,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PaginationDirection {
    Next,
    Prev,
}

impl Default for PaginationDirection {
    fn default() -> Self {
        Self::Next
    }
}

impl Default for PaginationData {
    fn default() -> Self {
        Self {
            cursor: None,
            offset: None,
            limit: Some(50),
            direction: Some(PaginationDirection::Next),
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
