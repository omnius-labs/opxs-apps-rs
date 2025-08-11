#[allow(unused)]
pub type Result<T> = std::result::Result<T, crate::error::Error>;

#[allow(unused)]
pub type ApiResult<T> = std::result::Result<T, crate::error::ApiErrorCode>;
