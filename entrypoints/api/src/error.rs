use axum::{Json, http::StatusCode, response::IntoResponse};
use backtrace::Backtrace;
use serde::Serialize;
use serde_json::json;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
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
            ErrorKind::IoError => write!(fmt, "I/O error"),
            ErrorKind::TimeError => write!(fmt, "time conversion error"),
            ErrorKind::SerdeError => write!(fmt, "serde error"),
            ErrorKind::DatabaseError => write!(fmt, "database error"),
            ErrorKind::HttpClientError => write!(fmt, "http client error"),
            ErrorKind::CryptoError => write!(fmt, "crypto error"),
            ErrorKind::UnexpectedError => write!(fmt, "unexpected error"),

            ErrorKind::AwsError => write!(fmt, "AWS error"),
            ErrorKind::GcpError => write!(fmt, "GCP error"),

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

pub struct Error {
    kind: ErrorKind,
    message: Option<String>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
    backtrace: Backtrace,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            message: None,
            source: None,
            backtrace: Backtrace::new(),
        }
    }

    pub fn message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn source<E: Into<Box<dyn std::error::Error + Send + Sync>>>(mut self, source: E) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = fmt.debug_struct("Error");

        debug.field("kind", &self.kind);

        if let Some(message) = &self.message {
            debug.field("message", message);
        }

        if let Some(source) = &self.source {
            debug.field("source", source);
        }

        debug.field("backtrace", &format_args!("{:?}", self.backtrace));

        debug.finish()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(message) = &self.message {
            write!(fmt, "{}: {}", self.kind, message)
        } else {
            write!(fmt, "{}", self.kind)
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|s| &**s as &(dyn std::error::Error + 'static))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::new(ErrorKind::IoError).message("io error").source(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Error {
        Error::new(ErrorKind::InvalidFormat).message("int parse error").source(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::new(ErrorKind::SerdeError).message("serde json error").source(e)
    }
}

impl From<std::env::VarError> for Error {
    fn from(e: std::env::VarError) -> Self {
        match e {
            std::env::VarError::NotPresent => Error::new(ErrorKind::NotFound).message("not found env var"),
            std::env::VarError::NotUnicode(_) => Error::new(ErrorKind::InvalidFormat).message("invalid utf-8"),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Error::new(ErrorKind::DatabaseError).message("Database operation failed").source(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::new(ErrorKind::HttpClientError).message("Http client error").source(e)
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Error::new(ErrorKind::InvalidFormat).message("Invalid jwt").source(e)
    }
}

impl From<ring::error::Unspecified> for Error {
    fn from(e: ring::error::Unspecified) -> Self {
        Error::new(ErrorKind::CryptoError).message("ring error").source(e)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(e: hex::FromHexError) -> Self {
        Error::new(ErrorKind::InvalidFormat).message("hex decode error").source(e)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Error::new(ErrorKind::InvalidFormat).message("base64 decode error").source(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::new(ErrorKind::InvalidFormat).message("utf-8 decode error").source(e)
    }
}

impl From<chrono::OutOfRangeError> for Error {
    fn from(e: chrono::OutOfRangeError) -> Self {
        Error::new(ErrorKind::TimeError).message("Time conversion failed").source(e)
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error::new(ErrorKind::InvalidFormat).message("URL parse error").source(e)
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(e: std::net::AddrParseError) -> Self {
        Error::new(ErrorKind::InvalidFormat).message("Network address parse error").source(e)
    }
}

impl From<axum_extra::typed_header::TypedHeaderRejection> for Error {
    fn from(e: axum_extra::typed_header::TypedHeaderRejection) -> Self {
        Error::new(ErrorKind::InvalidRequest).message("Invalid header").source(e)
    }
}

impl From<validator::ValidationErrors> for Error {
    fn from(e: validator::ValidationErrors) -> Self {
        Error::new(ErrorKind::InvalidRequest).message("Validation failed").source(e)
    }
}

impl From<axum::extract::rejection::JsonRejection> for Error {
    fn from(e: axum::extract::rejection::JsonRejection) -> Self {
        Error::new(ErrorKind::InvalidFormat).message("JSON parsing failed").source(e)
    }
}

impl From<omnius_core_cloud::Error> for Error {
    fn from(e: omnius_core_cloud::Error) -> Self {
        match e.kind() {
            omnius_core_cloud::ErrorKind::IoError => Error::new(ErrorKind::IoError).message("I/O operation failed").source(e),
            omnius_core_cloud::ErrorKind::TimeError => Error::new(ErrorKind::TimeError).message("Time conversion failed").source(e),

            omnius_core_cloud::ErrorKind::AwsError => Error::new(ErrorKind::AwsError).message("AWS operation failed").source(e),
            omnius_core_cloud::ErrorKind::GcpError => Error::new(ErrorKind::GcpError).message("GCP operation failed").source(e),

            omnius_core_cloud::ErrorKind::InvalidFormat => Error::new(ErrorKind::InvalidFormat).message("Invalid format error").source(e),
            omnius_core_cloud::ErrorKind::NotFound => Error::new(ErrorKind::NotFound).message("Resource not found").source(e),
        }
    }
}

impl From<omnius_opxs_auth::Error> for Error {
    fn from(e: omnius_opxs_auth::Error) -> Self {
        match e.kind() {
            omnius_opxs_auth::ErrorKind::IoError => Error::new(ErrorKind::IoError).message("I/O operation failed").source(e),
            omnius_opxs_auth::ErrorKind::TimeError => Error::new(ErrorKind::TimeError).message("Time conversion failed").source(e),
            omnius_opxs_auth::ErrorKind::SerdeError => Error::new(ErrorKind::SerdeError)
                .message("Serialization/deserialization failed")
                .source(e),
            omnius_opxs_auth::ErrorKind::DatabaseError => Error::new(ErrorKind::DatabaseError).message("Database operation failed").source(e),
            omnius_opxs_auth::ErrorKind::HttpClientError => Error::new(ErrorKind::HttpClientError).message("HTTP client error").source(e),
            omnius_opxs_auth::ErrorKind::CryptoError => Error::new(ErrorKind::CryptoError).message("Cryptographic operation failed").source(e),
            omnius_opxs_auth::ErrorKind::UnexpectedError => Error::new(ErrorKind::UnexpectedError).message("Unexpected error occurred").source(e),

            omnius_opxs_auth::ErrorKind::AwsError => Error::new(ErrorKind::AwsError).message("AWS operation failed").source(e),
            omnius_opxs_auth::ErrorKind::GcpError => Error::new(ErrorKind::GcpError).message("GCP operation failed").source(e),

            omnius_opxs_auth::ErrorKind::InvalidFormat => Error::new(ErrorKind::InvalidFormat).message("Invalid format error").source(e),
            omnius_opxs_auth::ErrorKind::NotFound => Error::new(ErrorKind::NotFound).message("Resource not found").source(e),
            omnius_opxs_auth::ErrorKind::TokenExpired => Error::new(ErrorKind::TokenExpired).message("Authentication token expired").source(e),
            omnius_opxs_auth::ErrorKind::Unauthorized => Error::new(ErrorKind::Unauthorized).message("Unauthorized access").source(e),
            omnius_opxs_auth::ErrorKind::Duplicated => Error::new(ErrorKind::Duplicated).message("Resource already exists").source(e),
        }
    }
}

impl From<omnius_opxs_email_send::Error> for Error {
    fn from(e: omnius_opxs_email_send::Error) -> Self {
        match e.kind() {
            omnius_opxs_email_send::ErrorKind::IoError => Error::new(ErrorKind::IoError).message("I/O operation failed").source(e),
            omnius_opxs_email_send::ErrorKind::TimeError => Error::new(ErrorKind::TimeError).message("Time conversion failed").source(e),
            omnius_opxs_email_send::ErrorKind::SerdeError => Error::new(ErrorKind::SerdeError)
                .message("Serialization/deserialization failed")
                .source(e),
            omnius_opxs_email_send::ErrorKind::DatabaseError => Error::new(ErrorKind::DatabaseError).message("Database operation failed").source(e),
            omnius_opxs_email_send::ErrorKind::HttpClientError => Error::new(ErrorKind::HttpClientError).message("HTTP client error").source(e),
            omnius_opxs_email_send::ErrorKind::CryptoError => Error::new(ErrorKind::CryptoError).message("Cryptographic operation failed").source(e),
            omnius_opxs_email_send::ErrorKind::UnexpectedError => {
                Error::new(ErrorKind::UnexpectedError).message("Unexpected error occurred").source(e)
            }

            omnius_opxs_email_send::ErrorKind::AwsError => Error::new(ErrorKind::AwsError).message("AWS operation failed").source(e),
            omnius_opxs_email_send::ErrorKind::GcpError => Error::new(ErrorKind::GcpError).message("GCP operation failed").source(e),

            omnius_opxs_email_send::ErrorKind::InvalidFormat => Error::new(ErrorKind::InvalidFormat).message("Invalid format error").source(e),
            omnius_opxs_email_send::ErrorKind::TokenExpired => Error::new(ErrorKind::TokenExpired).message("Authentication token expired").source(e),
            omnius_opxs_email_send::ErrorKind::NotFound => Error::new(ErrorKind::NotFound).message("Resource not found").source(e),
            omnius_opxs_email_send::ErrorKind::Unauthorized => Error::new(ErrorKind::Unauthorized).message("Unauthorized access").source(e),
            omnius_opxs_email_send::ErrorKind::Duplicated => Error::new(ErrorKind::Duplicated).message("Resource already exists").source(e),
            omnius_opxs_email_send::ErrorKind::UnsupportedType => Error::new(ErrorKind::UnsupportedType).message("Unsupported type").source(e),
        }
    }
}

impl From<omnius_opxs_file_convert::Error> for Error {
    fn from(e: omnius_opxs_file_convert::Error) -> Self {
        match e.kind() {
            omnius_opxs_file_convert::ErrorKind::IoError => Error::new(ErrorKind::IoError).message("I/O operation failed").source(e),
            omnius_opxs_file_convert::ErrorKind::TimeError => Error::new(ErrorKind::TimeError).message("Time conversion failed").source(e),
            omnius_opxs_file_convert::ErrorKind::SerdeError => Error::new(ErrorKind::SerdeError)
                .message("Serialization/deserialization failed")
                .source(e),
            omnius_opxs_file_convert::ErrorKind::DatabaseError => Error::new(ErrorKind::DatabaseError).message("Database operation failed").source(e),
            omnius_opxs_file_convert::ErrorKind::HttpClientError => Error::new(ErrorKind::HttpClientError).message("HTTP client error").source(e),
            omnius_opxs_file_convert::ErrorKind::CryptoError => {
                Error::new(ErrorKind::CryptoError).message("Cryptographic operation failed").source(e)
            }
            omnius_opxs_file_convert::ErrorKind::TaskError => Error::new(ErrorKind::UnexpectedError).message("Task execution failed").source(e),
            omnius_opxs_file_convert::ErrorKind::UnexpectedError => {
                Error::new(ErrorKind::UnexpectedError).message("Unexpected error occurred").source(e)
            }

            omnius_opxs_file_convert::ErrorKind::AwsError => Error::new(ErrorKind::AwsError).message("AWS operation failed").source(e),
            omnius_opxs_file_convert::ErrorKind::GcpError => Error::new(ErrorKind::GcpError).message("GCP operation failed").source(e),

            omnius_opxs_file_convert::ErrorKind::ProcessFailed => {
                Error::new(ErrorKind::UnexpectedError).message("Process execution failed").source(e)
            }
            omnius_opxs_file_convert::ErrorKind::InvalidFormat => Error::new(ErrorKind::InvalidFormat).message("Invalid format error").source(e),
            omnius_opxs_file_convert::ErrorKind::NotFound => Error::new(ErrorKind::NotFound).message("Resource not found").source(e),
            omnius_opxs_file_convert::ErrorKind::TokenExpired => {
                Error::new(ErrorKind::TokenExpired).message("Authentication token expired").source(e)
            }
            omnius_opxs_file_convert::ErrorKind::Unauthorized => Error::new(ErrorKind::Unauthorized).message("Unauthorized access").source(e),
            omnius_opxs_file_convert::ErrorKind::Duplicated => Error::new(ErrorKind::Duplicated).message("Resource already exists").source(e),
            omnius_opxs_file_convert::ErrorKind::UnsupportedType => Error::new(ErrorKind::UnsupportedType).message("Unsupported type").source(e),
        }
    }
}

#[derive(Debug, Serialize)]
enum ErrorCode {
    InternalServerError,
    InvalidRequest,
    NotFound,
    TokenExpired,
    Unauthorized,
    Duplicated,
    UnsupportedType,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::InternalServerError => write!(f, "InternalServerError"),
            ErrorCode::InvalidRequest => write!(f, "InvalidRequest"),
            ErrorCode::NotFound => write!(f, "NotFound"),
            ErrorCode::TokenExpired => write!(f, "TokenExpired"),
            ErrorCode::Unauthorized => write!(f, "Unauthorized"),
            ErrorCode::Duplicated => write!(f, "Duplicated"),
            ErrorCode::UnsupportedType => write!(f, "DuplicatedType"),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status_code, error_code) = match self.kind {
            ErrorKind::IoError => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalServerError),
            ErrorKind::TimeError => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalServerError),
            ErrorKind::SerdeError => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalServerError),
            ErrorKind::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalServerError),
            ErrorKind::HttpClientError => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalServerError),
            ErrorKind::CryptoError => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalServerError),
            ErrorKind::UnexpectedError => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalServerError),

            ErrorKind::AwsError => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalServerError),
            ErrorKind::GcpError => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalServerError),

            ErrorKind::InvalidFormat => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalServerError),
            ErrorKind::InvalidRequest => (StatusCode::BAD_REQUEST, ErrorCode::InvalidRequest),
            ErrorKind::NotFound => (StatusCode::NOT_FOUND, ErrorCode::NotFound),
            ErrorKind::TokenExpired => (StatusCode::UNAUTHORIZED, ErrorCode::TokenExpired),
            ErrorKind::Unauthorized => (StatusCode::UNAUTHORIZED, ErrorCode::Unauthorized),
            ErrorKind::Duplicated => (StatusCode::CONFLICT, ErrorCode::Duplicated),
            ErrorKind::UnsupportedType => (StatusCode::BAD_REQUEST, ErrorCode::UnsupportedType),
        };

        let payload = json!({"error_code": error_code.to_string()});
        (status_code, Json(payload)).into_response()
    }
}
