use std::backtrace::Backtrace;

use axum::http::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

use omnius_core_base::error::{OmniError, OmniErrorBuilder};

pub struct Error {
    kind: ErrorKind,
    message: Option<String>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
    backtrace: Option<Backtrace>,
}

pub struct ErrorBuilder {
    inner: Error,
}

impl Error {
    pub fn builder() -> ErrorBuilder {
        ErrorBuilder {
            inner: Self {
                kind: ErrorKind::Unknown,
                message: None,
                source: None,
                backtrace: None,
            },
        }
    }
}

impl OmniError for Error {
    type ErrorKind = ErrorKind;

    fn kind(&self) -> &Self::ErrorKind {
        &self.kind
    }

    fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.backtrace.as_ref()
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|s| &**s as &(dyn std::error::Error + 'static))
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        OmniError::fmt(self, f)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        OmniError::fmt(self, f)
    }
}

impl OmniErrorBuilder<Error> for ErrorBuilder {
    type ErrorKind = ErrorKind;

    fn kind(mut self, kind: Self::ErrorKind) -> Self {
        self.inner.kind = kind;
        self
    }

    fn message<S: Into<String>>(mut self, message: S) -> Self {
        self.inner.message = Some(message.into());
        self
    }

    fn source<E: Into<Box<dyn std::error::Error + Send + Sync>>>(mut self, source: E) -> Self {
        self.inner.source = Some(source.into());
        self
    }

    fn backtrace(mut self) -> Self {
        self.inner.backtrace = Some(Backtrace::capture());
        self
    }

    fn build(self) -> Error {
        self.inner
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Unknown,
    IoError,
    TimeError,
    SerdeError,
    DatabaseError,
    HttpClientError,
    CryptoError,
    UnexpectedError,

    AwsError,
    GcpError,

    InvalidFormat,
    InvalidRequest,
    TokenExpired,
    NotFound,
    Unauthorized,
    Duplicated,
    UnsupportedType,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Unknown => write!(fmt, "unknown"),
            ErrorKind::IoError => write!(fmt, "io error"),
            ErrorKind::TimeError => write!(fmt, "time conversion error"),
            ErrorKind::SerdeError => write!(fmt, "serde error"),
            ErrorKind::DatabaseError => write!(fmt, "database error"),
            ErrorKind::HttpClientError => write!(fmt, "http client error"),
            ErrorKind::CryptoError => write!(fmt, "crypto error"),
            ErrorKind::UnexpectedError => write!(fmt, "unexpected error"),

            ErrorKind::AwsError => write!(fmt, "aws error"),
            ErrorKind::GcpError => write!(fmt, "gcp error"),

            ErrorKind::InvalidFormat => write!(fmt, "invalid format"),
            ErrorKind::InvalidRequest => write!(fmt, "invalid request"),
            ErrorKind::TokenExpired => write!(fmt, "token expired"),
            ErrorKind::NotFound => write!(fmt, "not found"),
            ErrorKind::Unauthorized => write!(fmt, "unauthorized"),
            ErrorKind::Duplicated => write!(fmt, "duplicated"),
            ErrorKind::UnsupportedType => write!(fmt, "unsupported type"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::builder().kind(ErrorKind::IoError).message("io error").source(e).build()
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Error {
        Error::builder()
            .kind(ErrorKind::InvalidFormat)
            .message("int parse error")
            .source(e)
            .build()
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::builder().kind(ErrorKind::SerdeError).message("serde json error").source(e).build()
    }
}

impl From<std::env::VarError> for Error {
    fn from(e: std::env::VarError) -> Self {
        match e {
            std::env::VarError::NotPresent => Error::builder().kind(ErrorKind::NotFound).message("not found env var").build(),
            std::env::VarError::NotUnicode(_) => Error::builder().kind(ErrorKind::InvalidFormat).message("invalid utf-8").build(),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Error::builder()
            .kind(ErrorKind::DatabaseError)
            .message("database operation failed")
            .source(e)
            .build()
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::builder()
            .kind(ErrorKind::HttpClientError)
            .message("Http client error")
            .source(e)
            .build()
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Error::builder().kind(ErrorKind::InvalidFormat).message("invalid jwt").source(e).build()
    }
}

impl From<ring::error::Unspecified> for Error {
    fn from(e: ring::error::Unspecified) -> Self {
        Error::builder().kind(ErrorKind::CryptoError).message("ring error").source(e).build()
    }
}

impl From<hex::FromHexError> for Error {
    fn from(e: hex::FromHexError) -> Self {
        Error::builder()
            .kind(ErrorKind::InvalidFormat)
            .message("hex decode error")
            .source(e)
            .build()
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Error::builder()
            .kind(ErrorKind::InvalidFormat)
            .message("base64 decode error")
            .source(e)
            .build()
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::builder()
            .kind(ErrorKind::InvalidFormat)
            .message("utf-8 decode error")
            .source(e)
            .build()
    }
}

impl From<chrono::OutOfRangeError> for Error {
    fn from(e: chrono::OutOfRangeError) -> Self {
        Error::builder()
            .kind(ErrorKind::TimeError)
            .message("Time conversion failed")
            .source(e)
            .build()
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error::builder()
            .kind(ErrorKind::InvalidFormat)
            .message("URL parse error")
            .source(e)
            .build()
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(e: std::net::AddrParseError) -> Self {
        Error::builder()
            .kind(ErrorKind::InvalidFormat)
            .message("Network address parse error")
            .source(e)
            .build()
    }
}

impl From<axum_extra::typed_header::TypedHeaderRejection> for Error {
    fn from(e: axum_extra::typed_header::TypedHeaderRejection) -> Self {
        Error::builder()
            .kind(ErrorKind::InvalidRequest)
            .message("invalid header")
            .source(e)
            .build()
    }
}

impl From<validator::ValidationErrors> for Error {
    fn from(e: validator::ValidationErrors) -> Self {
        Error::builder()
            .kind(ErrorKind::InvalidRequest)
            .message("Validation failed")
            .source(e)
            .build()
    }
}

impl From<axum::extract::rejection::JsonRejection> for Error {
    fn from(e: axum::extract::rejection::JsonRejection) -> Self {
        Error::builder()
            .kind(ErrorKind::InvalidFormat)
            .message("JSON parsing failed")
            .source(e)
            .build()
    }
}

impl From<omnius_core_cloud::Error> for Error {
    fn from(e: omnius_core_cloud::Error) -> Self {
        match e.kind() {
            omnius_core_cloud::ErrorKind::Unknown => Error::builder().kind(ErrorKind::Unknown).source(e).build(),
            omnius_core_cloud::ErrorKind::IoError => Error::builder().kind(ErrorKind::IoError).source(e).build(),
            omnius_core_cloud::ErrorKind::TimeError => Error::builder().kind(ErrorKind::TimeError).source(e).build(),
            omnius_core_cloud::ErrorKind::AwsError => Error::builder().kind(ErrorKind::AwsError).source(e).build(),
            omnius_core_cloud::ErrorKind::GcpError => Error::builder().kind(ErrorKind::GcpError).source(e).build(),
            omnius_core_cloud::ErrorKind::InvalidFormat => Error::builder().kind(ErrorKind::InvalidFormat).source(e).build(),
            omnius_core_cloud::ErrorKind::NotFound => Error::builder().kind(ErrorKind::NotFound).source(e).build(),
        }
    }
}

impl From<omnius_opxs_auth::Error> for Error {
    fn from(e: omnius_opxs_auth::Error) -> Self {
        match e.kind() {
            omnius_opxs_auth::ErrorKind::Unknown => Error::builder().kind(ErrorKind::Unknown).source(e).build(),
            omnius_opxs_auth::ErrorKind::IoError => Error::builder().kind(ErrorKind::IoError).source(e).build(),
            omnius_opxs_auth::ErrorKind::TimeError => Error::builder().kind(ErrorKind::TimeError).source(e).build(),
            omnius_opxs_auth::ErrorKind::SerdeError => Error::builder().kind(ErrorKind::SerdeError).source(e).build(),
            omnius_opxs_auth::ErrorKind::DatabaseError => Error::builder().kind(ErrorKind::DatabaseError).source(e).build(),
            omnius_opxs_auth::ErrorKind::HttpClientError => Error::builder().kind(ErrorKind::HttpClientError).source(e).build(),
            omnius_opxs_auth::ErrorKind::CryptoError => Error::builder().kind(ErrorKind::CryptoError).source(e).build(),
            omnius_opxs_auth::ErrorKind::UnexpectedError => Error::builder().kind(ErrorKind::UnexpectedError).source(e).build(),
            omnius_opxs_auth::ErrorKind::AwsError => Error::builder().kind(ErrorKind::AwsError).source(e).build(),
            omnius_opxs_auth::ErrorKind::GcpError => Error::builder().kind(ErrorKind::GcpError).source(e).build(),
            omnius_opxs_auth::ErrorKind::InvalidFormat => Error::builder().kind(ErrorKind::InvalidFormat).source(e).build(),
            omnius_opxs_auth::ErrorKind::NotFound => Error::builder().kind(ErrorKind::NotFound).source(e).build(),
            omnius_opxs_auth::ErrorKind::TokenExpired => Error::builder().kind(ErrorKind::TokenExpired).source(e).build(),
            omnius_opxs_auth::ErrorKind::Unauthorized => Error::builder().kind(ErrorKind::Unauthorized).source(e).build(),
            omnius_opxs_auth::ErrorKind::Duplicated => Error::builder().kind(ErrorKind::Duplicated).source(e).build(),
        }
    }
}

impl From<omnius_opxs_email_send::Error> for Error {
    fn from(e: omnius_opxs_email_send::Error) -> Self {
        match e.kind() {
            omnius_opxs_email_send::ErrorKind::Unknown => Error::builder().kind(ErrorKind::Unknown).source(e).build(),
            omnius_opxs_email_send::ErrorKind::IoError => Error::builder().kind(ErrorKind::IoError).source(e).build(),
            omnius_opxs_email_send::ErrorKind::TimeError => Error::builder().kind(ErrorKind::TimeError).source(e).build(),
            omnius_opxs_email_send::ErrorKind::SerdeError => Error::builder().kind(ErrorKind::SerdeError).source(e).build(),
            omnius_opxs_email_send::ErrorKind::DatabaseError => Error::builder().kind(ErrorKind::DatabaseError).source(e).build(),
            omnius_opxs_email_send::ErrorKind::HttpClientError => Error::builder().kind(ErrorKind::HttpClientError).source(e).build(),
            omnius_opxs_email_send::ErrorKind::CryptoError => Error::builder().kind(ErrorKind::CryptoError).source(e).build(),
            omnius_opxs_email_send::ErrorKind::UnexpectedError => Error::builder().kind(ErrorKind::UnexpectedError).source(e).build(),
            omnius_opxs_email_send::ErrorKind::AwsError => Error::builder().kind(ErrorKind::AwsError).source(e).build(),
            omnius_opxs_email_send::ErrorKind::GcpError => Error::builder().kind(ErrorKind::GcpError).source(e).build(),
            omnius_opxs_email_send::ErrorKind::InvalidFormat => Error::builder().kind(ErrorKind::InvalidFormat).source(e).build(),
            omnius_opxs_email_send::ErrorKind::TokenExpired => Error::builder().kind(ErrorKind::TokenExpired).source(e).build(),
            omnius_opxs_email_send::ErrorKind::NotFound => Error::builder().kind(ErrorKind::NotFound).source(e).build(),
            omnius_opxs_email_send::ErrorKind::Unauthorized => Error::builder().kind(ErrorKind::Unauthorized).source(e).build(),
            omnius_opxs_email_send::ErrorKind::Duplicated => Error::builder().kind(ErrorKind::Duplicated).source(e).build(),
            omnius_opxs_email_send::ErrorKind::UnsupportedType => Error::builder().kind(ErrorKind::UnsupportedType).source(e).build(),
        }
    }
}

impl From<omnius_opxs_file_convert::Error> for Error {
    fn from(e: omnius_opxs_file_convert::Error) -> Self {
        match e.kind() {
            omnius_opxs_file_convert::ErrorKind::Unknown => Error::builder().kind(ErrorKind::Unknown).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::IoError => Error::builder().kind(ErrorKind::IoError).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::TimeError => Error::builder().kind(ErrorKind::TimeError).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::SerdeError => Error::builder().kind(ErrorKind::SerdeError).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::DatabaseError => Error::builder().kind(ErrorKind::DatabaseError).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::HttpClientError => Error::builder().kind(ErrorKind::HttpClientError).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::CryptoError => Error::builder().kind(ErrorKind::CryptoError).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::TaskError => Error::builder().kind(ErrorKind::UnexpectedError).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::UnexpectedError => Error::builder().kind(ErrorKind::UnexpectedError).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::AwsError => Error::builder().kind(ErrorKind::AwsError).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::GcpError => Error::builder().kind(ErrorKind::GcpError).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::ProcessFailed => Error::builder().kind(ErrorKind::UnexpectedError).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::InvalidFormat => Error::builder().kind(ErrorKind::InvalidFormat).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::NotFound => Error::builder().kind(ErrorKind::NotFound).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::TokenExpired => Error::builder().kind(ErrorKind::TokenExpired).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::Unauthorized => Error::builder().kind(ErrorKind::Unauthorized).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::Duplicated => Error::builder().kind(ErrorKind::Duplicated).source(e).build(),
            omnius_opxs_file_convert::ErrorKind::UnsupportedType => Error::builder().kind(ErrorKind::UnsupportedType).source(e).build(),
        }
    }
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub enum ApiErrorCode {
    InternalServerError,
    InvalidRequest,
    NotFound,
    TokenExpired,
    Unauthorized,
    Duplicated,
}

impl axum::response::IntoResponse for ApiErrorCode {
    fn into_response(self) -> axum::response::Response {
        use axum::Json;

        let status_code = match &self {
            ApiErrorCode::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::InvalidRequest => StatusCode::BAD_REQUEST,
            ApiErrorCode::NotFound => StatusCode::NOT_FOUND,
            ApiErrorCode::TokenExpired => StatusCode::UNAUTHORIZED,
            ApiErrorCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiErrorCode::Duplicated => StatusCode::CONFLICT,
        };

        let message = ApiErrorMessage { error_code: self.clone() };
        (status_code, Json(message)).into_response()
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiErrorMessage {
    pub error_code: ApiErrorCode,
}
