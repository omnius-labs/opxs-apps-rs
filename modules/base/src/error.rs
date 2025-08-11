use std::backtrace::Backtrace;

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
    UnexpectedError,

    AwsError,
    GcpError,

    InvalidFormat,
    NotFound,
    Mismatch,
    Duplicated,
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
            ErrorKind::UnexpectedError => write!(fmt, "unexpected error"),

            ErrorKind::AwsError => write!(fmt, "aws error"),
            ErrorKind::GcpError => write!(fmt, "gcp error"),

            ErrorKind::InvalidFormat => write!(fmt, "invalid format"),
            ErrorKind::NotFound => write!(fmt, "not found"),
            ErrorKind::Mismatch => write!(fmt, "mismatch"),
            ErrorKind::Duplicated => write!(fmt, "duplicated"),
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
        match &e {
            sqlx::Error::Database(db_err) => {
                // PostgreSQLの一意性制約違反エラーコード: 23505
                if db_err.code().as_deref() == Some("23505") {
                    return Error::builder()
                        .kind(ErrorKind::Duplicated)
                        .message("unique constraint violation")
                        .source(e)
                        .build();
                }
                Error::builder()
                    .kind(ErrorKind::DatabaseError)
                    .message("database operation failed")
                    .source(e)
                    .build()
            }
            _ => Error::builder()
                .kind(ErrorKind::DatabaseError)
                .message("database operation failed")
                .source(e)
                .build(),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::builder()
            .kind(ErrorKind::HttpClientError)
            .message("http client error")
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
