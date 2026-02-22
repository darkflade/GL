use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    Conflict(String),
    Internal {
        public_message: String,
        context: String,
    },
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    pub fn internal(context: impl Into<String>) -> Self {
        Self::Internal {
            public_message: "Internal server error".to_string(),
            context: context.into(),
        }
    }

    fn public_message(&self) -> &str {
        match self {
            AppError::BadRequest(message) => message.as_str(),
            AppError::Unauthorized(message) => message.as_str(),
            AppError::NotFound(message) => message.as_str(),
            AppError::Conflict(message) => message.as_str(),
            AppError::Internal { public_message, .. } => public_message.as_str(),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.public_message())
    }
}

impl std::error::Error for AppError {}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        if let AppError::Internal { context, .. } = self {
            log::error!("{context}");
        }

        HttpResponse::build(self.status_code()).json(ErrorBody {
            error: self.public_message(),
        })
    }
}
