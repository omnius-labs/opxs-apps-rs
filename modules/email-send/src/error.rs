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
    NotFound,
    TokenExpired,
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
            ErrorKind::NotFound => write!(fmt, "not found"),
            ErrorKind::TokenExpired => write!(fmt, "token expired"),
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
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            message: None,
            source: None,
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
        write!(fmt, "{}", self)
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
        Error::new(ErrorKind::IoError).message("I/O error").source(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Error {
        Error::new(ErrorKind::InvalidFormat).message("Int parse error").source(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::new(ErrorKind::SerdeError).message("Serde json error").source(e)
    }
}

impl From<std::env::VarError> for Error {
    fn from(e: std::env::VarError) -> Self {
        match e {
            std::env::VarError::NotPresent => Error::new(ErrorKind::NotFound).message("Not found env var"),
            std::env::VarError::NotUnicode(_) => Error::new(ErrorKind::InvalidFormat).message("Invalid utf-8"),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match &e {
            sqlx::Error::Database(db_err) => {
                // PostgreSQLの一意性制約違反エラーコード: 23505
                if db_err.code().as_deref() == Some("23505") {
                    return Error::new(ErrorKind::Duplicated).message("Unique constraint violation").source(e);
                }
                Error::new(ErrorKind::DatabaseError).message("Database operation failed").source(e)
            }
            _ => Error::new(ErrorKind::DatabaseError).message("Database operation failed").source(e),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::new(ErrorKind::HttpClientError).message("HTTP client error").source(e)
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Error::new(ErrorKind::InvalidFormat).message("Invalid JWT").source(e)
    }
}

impl From<ring::error::Unspecified> for Error {
    fn from(e: ring::error::Unspecified) -> Self {
        Error::new(ErrorKind::CryptoError).message("Ring error").source(e)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(e: hex::FromHexError) -> Self {
        Error::new(ErrorKind::InvalidFormat).message("Hex decode error").source(e)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Error::new(ErrorKind::InvalidFormat).message("Base64 decode error").source(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::new(ErrorKind::InvalidFormat).message("UTF-8 decode error").source(e)
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
